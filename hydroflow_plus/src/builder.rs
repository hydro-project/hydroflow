use std::cell::RefCell;
use std::marker::PhantomData;

use hydroflow::futures::stream::Stream;
use hydroflow_lang::graph::{partition_graph, propegate_flow_props, FlatGraphBuilder};
use proc_macro2::Span;
use quote::quote;
use stageleft::{IntoQuotedOnce, Quoted, QuotedContext};
use syn::parse_quote;

use crate::{HfBuilt, HfStream, RuntimeContext};

pub struct HfBuilder<'a> {
    pub(crate) next_id: RefCell<usize>,
    pub(crate) builder: RefCell<Option<FlatGraphBuilder>>,
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a> QuotedContext for HfBuilder<'a> {
    fn create() -> Self {
        HfBuilder::new()
    }
}

impl<'a> HfBuilder<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> HfBuilder<'a> {
        HfBuilder {
            next_id: RefCell::new(0),
            builder: RefCell::new(Some(FlatGraphBuilder::new())),
            _phantom: PhantomData,
        }
    }

    pub fn build(&self) -> HfBuilt<'a> {
        let builder = self.builder.borrow_mut().take().unwrap();

        let (flat_graph, _, _) = builder.build();
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
        let _ =
            propegate_flow_props::propegate_flow_props(&mut partitioned_graph, &mut diagnostics);

        let tokens = partitioned_graph.as_code(&root, true, quote::quote!(), &mut diagnostics);

        HfBuilt {
            tokens,
            _phantom: PhantomData,
        }
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext {
            _phantom: PhantomData,
        }
    }

    pub fn source_stream<T, E: Stream<Item = T> + Unpin>(
        &'a self,
        e: impl Quoted<'a, E>,
    ) -> HfStream<'a, T> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let e = e.splice();

        self.builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = source_stream(#e) -> tee();
            });

        HfStream {
            ident,
            graph: self,
            _phantom: PhantomData,
        }
    }

    pub fn source_iter<T, E: IntoIterator<Item = T>>(
        &'a self,
        e: impl IntoQuotedOnce<'a, E>,
    ) -> HfStream<'a, T> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let e = e.splice();

        self.builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = source_iter(#e) -> tee();
            });

        HfStream {
            ident,
            graph: self,
            _phantom: PhantomData,
        }
    }

    pub fn cycle<T>(&'a self) -> (HfCycle<'a, T>, HfStream<'a, T>) {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        self.builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = tee();
            });

        (
            HfCycle {
                ident: ident.clone(),
                graph: self,
                _phantom: PhantomData,
            },
            HfStream {
                ident,
                graph: self,
                _phantom: PhantomData,
            },
        )
    }
}

pub struct HfCycle<'a, T> {
    ident: syn::Ident,
    graph: &'a HfBuilder<'a>,
    _phantom: PhantomData<T>,
}

impl<'a, T> HfCycle<'a, T> {
    pub fn complete(self, stream: &HfStream<'a, T>) {
        let ident = self.ident;
        let stream_ident = stream.ident.clone();

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #stream_ident -> #ident;
            });
    }
}
