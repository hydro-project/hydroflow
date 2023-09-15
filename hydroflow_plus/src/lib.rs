use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;

use hydroflow_lang::graph::{partition_graph, propegate_flow_props, FlatGraphBuilder};
pub use hydroflow_plus_macro::{flow, q, quse};
use proc_macro2::{Span, TokenStream};
pub use quote::quote;
use quote::ToTokens;
use syn::parse_quote;
pub use {hydroflow, syn};

pub fn lol() {
    println!("lol");
}

pub trait ParseFromLiteral {
    fn parse_from_literal(literal: &syn::Expr) -> Self;
}

impl ParseFromLiteral for u32 {
    fn parse_from_literal(literal: &syn::Expr) -> Self {
        match literal {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) => lit_int.base10_parse().unwrap(),
            _ => panic!("Expected literal"),
        }
    }
}

pub trait FreeVariable<O> {
    fn to_tokens(&self) -> TokenStream;
    fn uninitialized(&self) -> O {
        #[allow(clippy::uninit_assumed_init)]
        unsafe {
            MaybeUninit::uninit().assume_init()
        }
    }
}

impl FreeVariable<u32> for u32 {
    fn to_tokens(&self) -> TokenStream {
        quote!(#self)
    }
}

pub struct RuntimeData<T> {
    ident: String,
    _phantom: PhantomData<T>,
}

impl<T> RuntimeData<T> {
    pub fn new(ident: &str) -> RuntimeData<T> {
        RuntimeData {
            ident: ident.to_string(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Deref for RuntimeData<T> {
    type Target = T;
    fn deref(&self) -> &T {
        panic!("RuntimeData should not be dereferenced.")
    }
}

impl<T> FreeVariable<T> for RuntimeData<T> {
    fn to_tokens(&self) -> TokenStream {
        let ident = syn::Ident::new(&self.ident, Span::call_site());
        quote!(#ident)
    }
}

pub struct Import<T> {
    path: String,
    _phantom: PhantomData<T>
}

impl <T> Import<T> {
    pub fn create(
        path: &str,
        _unused_type_check: T
    ) -> Import<T> {
        Import {
            path: path.to_string(),
            _phantom: PhantomData
        }
    }
}

impl<T> FreeVariable<T> for Import<T> {
    fn to_tokens(&self) -> TokenStream {
        let parsed = syn::parse_str::<syn::Path>(&self.path).unwrap();
        quote!(::#parsed)
    }
}

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

pub struct QuotedExpr<T> {
    expr: syn::Expr,
    free_variables: Vec<(String, TokenStream)>,
    _phantom: PhantomData<T>,
}

impl<T> QuotedExpr<T> {
    pub fn create(
        expr: &str,
        free_variables: Vec<(String, TokenStream)>,
        _unused_type_check: T,
    ) -> QuotedExpr<T> {
        let expr = syn::parse_str(expr).unwrap();
        QuotedExpr {
            expr,
            free_variables,
            _phantom: PhantomData,
        }
    }
}

impl<T> ToTokens for QuotedExpr<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let instantiated_free_variables = self.free_variables.iter().map(|(ident, value)| {
            let ident = syn::Ident::new(ident, Span::call_site());
            quote!(let #ident = #value;)
        });

        let expr = &self.expr;
        tokens.extend(quote!({
            #(#instantiated_free_variables)*
            #expr
        }));
    }
}

pub struct HydroflowNode<T> {
    ident: syn::Ident,
    _phantom: PhantomData<T>,
}

impl<T> HydroflowNode<T> {
    pub fn source_iter(e: QuotedExpr<impl IntoIterator<Item = T>>) -> HydroflowNode<T> {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let ident = syn::Ident::new(&format!("source_{}", next_id), Span::call_site());

        HYDROFLOW_BUILDER.with(|builder| {
            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #ident = source_iter(#e) -> tee();
                });
        });

        HydroflowNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn map<U>(&self, f: QuotedExpr<impl Fn(T) -> U>) -> HydroflowNode<U> {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("map_{}", next_id), Span::call_site());

        HYDROFLOW_BUILDER.with(|builder| {
            builder
                .borrow_mut()
                .as_mut()
                .unwrap()
                .add_statement(parse_quote! {
                    #ident = #self_ident -> map(#f) -> tee();
                });
        });

        HydroflowNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn for_each(&self, f: QuotedExpr<impl Fn(T)>) {
        let next_id = HYDROFLOW_NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        });

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("for_each_{}", next_id), Span::call_site());

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

impl <K, V1> HydroflowNode<(K, V1)> {
    pub fn join<V2>(&self, n: &HydroflowNode<(K, V2)>) -> HydroflowNode<(K, (V1, V2))> {
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

        HydroflowNode {
            ident,
            _phantom: PhantomData,
        }
    }
}
