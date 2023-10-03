use std::cell::RefCell;
use std::marker::PhantomData;

use hydroflow::futures::stream::Stream;
use hydroflow::scheduled::context::Context;
use hydroflow_lang::graph::{partition_graph, propegate_flow_props, FlatGraphBuilder};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quoting::{FreeVariable, IntoQuotedMut, IntoQuotedOnce, ToFreeVariableTokens};
use syn::parse_quote;

pub mod internal {
    pub use proc_macro2::TokenStream;
    pub use quote::quote;
    pub use syn;
}

pub mod quoting;

pub use hydroflow;
pub use hydroflow_plus_macro::{flow, q, qtype, quse, quse_type};
pub use quoting::RuntimeData;

thread_local! {
    static HYDROFLOW_NEXT_ID: RefCell<usize> = RefCell::new(0);
    static HYDROFLOW_BUILDER: RefCell<Option<FlatGraphBuilder>> = RefCell::new(None);
}

pub fn hydroflow_build(f: impl Fn()) -> TokenStream {
    let hydroflow_crate = proc_macro_crate::crate_name("hydroflow_plus")
        .expect("hydroflow_plus should be present in `Cargo.toml`");
    let root = match hydroflow_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus::hydroflow },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident::hydroflow }
        }
    };

    HYDROFLOW_NEXT_ID.with(|next_id| {
        *next_id.borrow_mut() = 0;
        HYDROFLOW_BUILDER.with(|builder| {
            *builder.borrow_mut() = Some(FlatGraphBuilder::new());
            f();

            let (flat_graph, _, _) = builder.borrow_mut().take().unwrap().build();
            let mut partitioned_graph =
                partition_graph(flat_graph).expect("Failed to partition (cycle detected).");

            let mut diagnostics = Vec::new();
            // Propgeate flow properties throughout the graph.
            // TODO(mingwei): Should this be done at a flat graph stage instead?
            let _ = propegate_flow_props::propegate_flow_props(
                &mut partitioned_graph,
                &mut diagnostics,
            );

            partitioned_graph.as_code(&root, true, quote::quote!(), &mut diagnostics)
        })
    })
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

pub struct HfGraph<'a> {
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a> HfGraph<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> HfGraph<'a> {
        HfGraph {
            _phantom: PhantomData,
        }
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext {
            _phantom: PhantomData,
        }
    }

    pub fn source_stream<T, E: Stream<Item = T> + Unpin>(
        &self,
        e: impl IntoQuotedOnce<'a, E>,
    ) -> HfNode<'a, T> {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let ident = syn::Ident::new(&format!("source_{}", next_id), Span::call_site());
        let e = e.to_quoted();

        HYDROFLOW_BUILDER.with(|builder| {
            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #ident = source_stream(#e) -> tee();
                });
        });

        HfNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn source_iter<T, E: IntoIterator<Item = T>>(
        &self,
        e: impl IntoQuotedOnce<'a, E>,
    ) -> HfNode<'a, T> {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let ident = syn::Ident::new(&format!("source_{}", next_id), Span::call_site());
        let e = e.to_quoted();

        HYDROFLOW_BUILDER.with(|builder| {
            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #ident = source_iter(#e) -> tee();
                });
        });

        HfNode {
            ident,
            _phantom: PhantomData,
        }
    }
}

pub struct HfNode<'a, T> {
    ident: syn::Ident,
    _phantom: PhantomData<&'a mut &'a T>,
}

impl<'a, T> HfNode<'a, T> {
    pub fn source_iter<E: IntoIterator<Item = T>>(e: impl IntoQuotedOnce<'a, E>) -> HfNode<'a, T> {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let ident = syn::Ident::new(&format!("source_{}", next_id), Span::call_site());
        let e = e.to_quoted();

        HYDROFLOW_BUILDER.with(|builder| {
            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #ident = source_iter(#e) -> tee();
                });
        });

        HfNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn map<U, F: Fn(T) -> U + 'a>(&self, f: impl IntoQuotedMut<'a, F>) -> HfNode<'a, U> {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("map_{}", next_id), Span::call_site());
        let f = f.to_quoted();

        HYDROFLOW_BUILDER.with(|builder| {
            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #ident = #self_ident -> map(#f) -> tee();
                });
        });

        HfNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(&self, f: impl IntoQuotedMut<'a, F>) -> HfNode<'a, T> {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("filter_{}", next_id), Span::call_site());
        let f = f.to_quoted();

        HYDROFLOW_BUILDER.with(|builder| {
            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #ident = #self_ident -> filter(#f) -> tee();
                });
        });

        HfNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn for_each<F: Fn(T) + 'a>(&self, f: impl IntoQuotedMut<'a, F>) {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("for_each_{}", next_id), Span::call_site());
        let f = f.to_quoted();

        HYDROFLOW_BUILDER.with(|builder| {
            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #ident = #self_ident -> for_each(#f);
                });
        });
    }
}

impl<'a, K, V1> HfNode<'a, (K, V1)> {
    pub fn join<V2>(&self, n: &HfNode<(K, V2)>) -> HfNode<(K, (V1, V2))> {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let self_ident = &self.ident;
        let other_ident = &n.ident;
        let ident = syn::Ident::new(&format!("for_each_{}", next_id), Span::call_site());

        HYDROFLOW_BUILDER.with(|builder| {
            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #ident = join() -> tee();
                });

            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #self_ident -> [0]#ident;
                });

            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #other_ident -> [1]#ident;
                });
        });

        HfNode {
            ident,
            _phantom: PhantomData,
        }
    }
}
