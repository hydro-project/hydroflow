use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use hydroflow_lang::graph::{
    eliminate_extra_unions_tees, partition_graph, propagate_flow_props, FlatGraphBuilder,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use stageleft::{Quoted, QuotedContext};
use syn::parse_quote;

use crate::location::{ClusterSpec, LocalDeploy, Location, ProcessSpec};
use crate::{HfBuilt, RuntimeContext};

pub type Builders = RefCell<Option<BTreeMap<usize, FlatGraphBuilder>>>;

pub struct FlowBuilder<'a, D: LocalDeploy<'a> + ?Sized> {
    pub(crate) next_id: RefCell<usize>,
    pub(crate) builders: Builders,
    nodes: RefCell<Vec<D::Process>>,
    clusters: RefCell<Vec<D::Cluster>>,

    /// Tracks metadata about concrete deployments in this graph, such
    /// as the IDs of each node in a cluster. This is written to
    /// by `ProcessSpec` and `ClusterSpec` and is written to
    /// each instantiated node and cluster via `HfNode::update_meta`.
    meta: RefCell<D::Meta>,

    next_node_id: RefCell<usize>,
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a, D: LocalDeploy<'a>> QuotedContext for FlowBuilder<'a, D> {
    fn create() -> Self {
        FlowBuilder::new()
    }
}

impl<'a, D: LocalDeploy<'a>> FlowBuilder<'a, D> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> FlowBuilder<'a, D> {
        FlowBuilder {
            next_id: RefCell::new(0),
            builders: RefCell::new(Some(Default::default())),
            nodes: RefCell::new(Vec::new()),
            clusters: RefCell::new(Vec::new()),
            meta: RefCell::new(Default::default()),
            next_node_id: RefCell::new(0),
            _phantom: PhantomData,
        }
    }

    pub fn builder_components(&self) -> (&RefCell<usize>, &Builders) {
        (&self.next_id, &self.builders)
    }

    pub fn process(&'a self, builder: &impl ProcessSpec<'a, D>) -> D::Process {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let node = builder.build(id, self, &mut self.meta.borrow_mut());
        self.nodes.borrow_mut().push(node.clone());

        self.update_metas();

        node
    }

    pub fn cluster(&'a self, builder: &impl ClusterSpec<'a, D>) -> D::Cluster {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let cluster = builder.build(id, self, &mut self.meta.borrow_mut());
        self.clusters.borrow_mut().push(cluster.clone());

        self.update_metas();

        cluster
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext {
            _phantom: PhantomData,
        }
    }

    fn update_metas(&self) {
        self.nodes
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.update_meta(&self.meta.borrow()));
        self.clusters
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.update_meta(&self.meta.borrow()));
    }
}

fn build_inner<'a, D: LocalDeploy<'a>>(me: &FlowBuilder<'a, D>, id: TokenStream) -> HfBuilt<'a> {
    let builders = me.builders.borrow_mut().take().unwrap();

    let mut conditioned_tokens = None;
    for (subgraph_id, builder) in builders {
        let (mut flat_graph, _, _) = builder.build();
        eliminate_extra_unions_tees(&mut flat_graph);
        let mut partitioned_graph =
            partition_graph(flat_graph).expect("Failed to partition (cycle detected).");

        let hydroflow_crate = proc_macro_crate::crate_name("hydroflow_plus")
            .expect("hydroflow_plus should be present in `Cargo.toml`");
        let root = match hydroflow_crate {
            proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
            proc_macro_crate::FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, Span::call_site());
                quote! { #ident }
            }
        };

        let mut diagnostics = Vec::new();
        // Propagate flow properties throughout the graph.
        // TODO(mingwei): Should this be done at a flat graph stage instead?
        let _ =
            propagate_flow_props::propagate_flow_props(&mut partitioned_graph, &mut diagnostics);

        let tokens = partitioned_graph.as_code(&root, true, quote::quote!(), &mut diagnostics);

        if let Some(conditioned_tokens) = conditioned_tokens.as_mut() {
            *conditioned_tokens = parse_quote! {
                #conditioned_tokens else if __given_id == #subgraph_id {
                    #tokens
                }
            };
        } else {
            conditioned_tokens = Some(parse_quote! {
                if __given_id == #subgraph_id {
                    #tokens
                }
            });
        }
    }

    let conditioned_tokens: TokenStream = conditioned_tokens.unwrap();

    HfBuilt {
        tokens: parse_quote! {
            let __given_id = #id;
            #conditioned_tokens else {
                panic!("Invalid node id: {}", __given_id);
            }
        },
        _phantom: PhantomData,
    }
}

impl<'a, D: LocalDeploy<'a, RuntimeID = usize>> FlowBuilder<'a, D> {
    pub fn build(&self, id: impl Quoted<'a, usize>) -> HfBuilt<'a> {
        build_inner(self, id.splice())
    }
}

impl<'a, D: LocalDeploy<'a, RuntimeID = ()>> FlowBuilder<'a, D> {
    pub fn build_single(&self) -> HfBuilt<'a> {
        if self.builders.borrow().as_ref().unwrap().len() != 1 {
            panic!("Expected exactly one node in the graph.");
        }

        build_inner(self, quote!(0))
    }
}
