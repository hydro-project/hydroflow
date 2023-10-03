use std::cell::RefCell;
use std::marker::PhantomData;

use hydroflow::futures::stream::Stream;
use hydroflow::scheduled::context::Context;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow_lang::graph::{partition_graph, propegate_flow_props, FlatGraphBuilder};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use quoting::{
    FreeVariable, IntoQuotedMut, IntoQuotedOnce, Quoted, QuotedContext, ToFreeVariableTokens,
};
use syn::parse_quote;

pub mod internal {
    pub use proc_macro2::TokenStream;
    pub use quote::quote;
    pub use syn;
}

pub mod quoting;

pub use hydroflow;
pub use hydroflow_plus_macro::{entry, q, qtype, quse, quse_type};
pub use quoting::RuntimeData;

pub fn hydroflow_build(f: impl FnOnce() -> FlatGraphBuilder) -> TokenStream {
    let hydroflow_crate = proc_macro_crate::crate_name("hydroflow_plus")
        .expect("hydroflow_plus should be present in `Cargo.toml`");
    let root = match hydroflow_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus::hydroflow },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident::hydroflow }
        }
    };

    let (flat_graph, _, _) = f().build();
    let mut partitioned_graph =
        partition_graph(flat_graph).expect("Failed to partition (cycle detected).");

    let mut diagnostics = Vec::new();
    // Propgeate flow properties throughout the graph.
    // TODO(mingwei): Should this be done at a flat graph stage instead?
    let _ = propegate_flow_props::propegate_flow_props(&mut partitioned_graph, &mut diagnostics);

    partitioned_graph.as_code(&root, true, quote::quote!(), &mut diagnostics)
}

#[derive(Clone)]
pub struct RuntimeContext<'a> {
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl Copy for RuntimeContext<'_> {}

impl<'a> ToFreeVariableTokens for RuntimeContext<'a> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(&context)))
    }
}

impl<'a> FreeVariable<&'a Context> for RuntimeContext<'a> {}

pub struct HfBuilt<'a> {
    tokens: TokenStream,
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a> Quoted<Hydroflow<'a>> for HfBuilt<'a> {}

impl<'a> ToTokens for HfBuilt<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens);
    }
}

pub struct HfBuilder<'a> {
    next_id: RefCell<usize>,
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
        HfBuilt {
            tokens: hydroflow_build(|| builder),
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
        e: impl IntoQuotedOnce<'a, E>,
    ) -> HfStream<'a, T> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("source_{}", next_id), Span::call_site());
        let e = e.to_quoted();

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

        let ident = syn::Ident::new(&format!("source_{}", next_id), Span::call_site());
        let e = e.to_quoted();

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
}

pub struct HfStream<'a, T> {
    ident: syn::Ident,
    graph: &'a HfBuilder<'a>,
    _phantom: PhantomData<&'a mut &'a T>,
}

impl<'a, T> HfStream<'a, T> {
    pub fn map<U, F: Fn(T) -> U + 'a>(&self, f: impl IntoQuotedMut<'a, F>) -> HfStream<'a, U> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("map_{}", next_id), Span::call_site());
        let f = f.to_quoted();

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = #self_ident -> map(#f) -> tee();
            });

        HfStream {
            ident,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(&self, f: impl IntoQuotedMut<'a, F>) -> HfStream<'a, T> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("filter_{}", next_id), Span::call_site());
        let f = f.to_quoted();

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = #self_ident -> filter(#f) -> tee();
            });

        HfStream {
            ident,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }

    pub fn for_each<F: Fn(T) + 'a>(&self, f: impl IntoQuotedMut<'a, F>) {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("for_each_{}", next_id), Span::call_site());
        let f = f.to_quoted();

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = #self_ident -> for_each(#f);
            });
    }
}

impl<'a, K, V1> HfStream<'a, (K, V1)> {
    pub fn join<V2>(&'a self, n: &HfStream<(K, V2)>) -> HfStream<(K, (V1, V2))> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let other_ident = &n.ident;
        let ident = syn::Ident::new(&format!("for_each_{}", next_id), Span::call_site());

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = join() -> tee();
            });

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #self_ident -> [0]#ident;
            });

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #other_ident -> [1]#ident;
            });

        HfStream {
            ident,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }
}
