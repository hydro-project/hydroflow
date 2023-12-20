use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ops::Deref;

use hydroflow_lang::graph::{
    eliminate_extra_unions_tees, partition_graph, propegate_flow_props, FlatGraphBuilder,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use stageleft::{Quoted, QuotedContext};
use syn::parse_quote;

use crate::node::{HfClusterBuilder, HfDeploy, HfNode, HfNodeBuilder};
use crate::{HfBuilt, RuntimeContext};

pub type Builders = RefCell<Option<BTreeMap<usize, FlatGraphBuilder>>>;

pub struct HfBuilder<'a, D: HfDeploy<'a> + ?Sized> {
    pub(crate) next_id: RefCell<usize>,
    pub(crate) builders: Builders,
    nodes: RefCell<Vec<D::Node>>,
    clusters: RefCell<Vec<D::Cluster>>,
    pub meta: RefCell<Option<D::Meta>>,
    next_node_id: RefCell<usize>,
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a, D: HfDeploy<'a>> QuotedContext for HfBuilder<'a, D> {
    fn create() -> Self {
        HfBuilder::new()
    }
}

impl<'a, D: HfDeploy<'a>> HfBuilder<'a, D> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> HfBuilder<'a, D> {
        HfBuilder {
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

    pub fn node(&'a self, builder: &impl HfNodeBuilder<'a, D>) -> D::Node {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let node = builder.build(id, self);
        self.nodes.borrow_mut().push(node.clone());
        node
    }

    pub fn cluster(&'a self, builder: &impl HfClusterBuilder<'a, D>) -> D::Cluster {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let cluster = builder.build(id, self);
        self.clusters.borrow_mut().push(cluster.clone());
        cluster
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext {
            _phantom: PhantomData,
        }
    }
}

impl<'a, D: HfDeploy<'a, RuntimeID = ()>> HfBuilder<'a, D> {
    pub fn wire(&self) {
        let meta_borrow = self.meta.borrow();
        let meta = meta_borrow.deref();
        self.nodes
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.build(meta));
        self.clusters
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.build(meta));
    }
}

impl<'a, D: HfDeploy<'a, RuntimeID = usize>> HfBuilder<'a, D> {
    pub fn build(&self, id: impl Quoted<'a, usize>) -> HfBuilt<'a> {
        let meta_borrow = self.meta.borrow();
        let meta = meta_borrow.deref();
        self.nodes
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.build(meta));
        self.clusters
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.build(meta));
        drop(meta_borrow);

        let builders = self.builders.borrow_mut().take().unwrap();

        let mut conditioned_tokens = None;
        for (node_id, builder) in builders {
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
            // Propgeate flow properties throughout the graph.
            // TODO(mingwei): Should this be done at a flat graph stage instead?
            let _ = propegate_flow_props::propegate_flow_props(
                &mut partitioned_graph,
                &mut diagnostics,
            );

            let tokens = partitioned_graph.as_code(&root, true, quote::quote!(), &mut diagnostics);

            if let Some(conditioned_tokens) = conditioned_tokens.as_mut() {
                *conditioned_tokens = parse_quote! {
                    #conditioned_tokens else if __given_id == #node_id {
                        #tokens
                    }
                };
            } else {
                conditioned_tokens = Some(parse_quote! {
                    if __given_id == #node_id {
                        #tokens
                    }
                });
            }
        }

        let id_spliced = id.splice();
        let conditioned_tokens: TokenStream = conditioned_tokens.unwrap();

        HfBuilt {
            tokens: parse_quote! {
                let __given_id = #id_spliced;
                #conditioned_tokens else {
                    panic!("Invalid node id: {}", __given_id);
                }
            },
            _phantom: PhantomData,
        }
    }
}
