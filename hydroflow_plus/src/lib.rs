use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use hydroflow::scheduled::context::Context;
use hydroflow_lang::graph::{partition_graph, propegate_flow_props, FlatGraphBuilder};
pub use hydroflow_plus_macro::{flow, q, qtype, quse};
use proc_macro2::Span;
pub use proc_macro2::TokenStream;
pub use quote::quote;
use quote::ToTokens;
use syn::parse_quote;
pub use {hydroflow, syn};

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

pub trait ToFreeVariableTokens {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>);
}

pub trait FreeVariable<O>
where
    Self: Sized,
{
    fn uninitialized(self) -> O {
        #[allow(clippy::uninit_assumed_init)]
        unsafe {
            MaybeUninit::uninit().assume_init()
        }
    }
}

impl ToFreeVariableTokens for u32 {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(#self)))
    }
}

impl FreeVariable<u32> for u32 {}

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

pub struct RuntimeData<T> {
    ident: &'static str,
    _phantom: PhantomData<T>,
}

impl<T: Copy> Copy for RuntimeData<T> {}

impl<T> Clone for RuntimeData<T> {
    fn clone(&self) -> Self {
        RuntimeData {
            ident: self.ident.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> RuntimeData<T> {
    pub fn new(ident: &'static str) -> RuntimeData<T> {
        RuntimeData {
            ident,
            _phantom: PhantomData,
        }
    }
}

impl<T> ToFreeVariableTokens for RuntimeData<T> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        let ident = syn::Ident::new(self.ident, Span::call_site());
        (None, Some(quote!(#ident)))
    }
}

impl<T> FreeVariable<T> for RuntimeData<T> {}

pub struct Import<T, P> {
    parent: Option<P>,
    path: &'static str,
    _phantom: PhantomData<T>,
}

impl<T, P: Copy> Copy for Import<T, P> {}
impl<T, P: Copy> Clone for Import<T, P> {
    fn clone(&self) -> Self {
        Import {
            parent: self.parent,
            path: self.path,
            _phantom: PhantomData,
        }
    }
}

pub fn create_import<T>(path: &'static str, _unused_type_check: T) -> Import<T, u32> {
    Import {
        parent: None,
        path,
        _phantom: PhantomData,
    }
}

impl<T, P: ToFreeVariableTokens + Copy> Import<T, P> {
    pub fn extend<T2>(
        &self,
        path: &'static str,
        _unused_type_check: T2,
    ) -> Import<T2, Import<T, P>> {
        Import {
            parent: Some(*self),
            path,
            _phantom: PhantomData,
        }
    }
}

impl<T, P: ToFreeVariableTokens> ToFreeVariableTokens for Import<T, P> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        if let Some(parent) = &self.parent {
            let (prelude, value) = parent.to_tokens();
            let parsed = syn::parse_str::<syn::Path>(self.path).unwrap();
            (prelude, Some(quote!(#value::#parsed)))
        } else {
            let parsed = syn::parse_str::<syn::Path>(self.path).unwrap();
            (Some(quote!(use ::#parsed;)), None)
        }
    }
}

impl<T, P: ToFreeVariableTokens> FreeVariable<T> for Import<T, P> {}

pub struct Type {
    definition: String,
}

impl Type {
    pub fn new(def: &str) -> Type {
        Type {
            definition: def.to_string(),
        }
    }
}

impl<F: Fn() -> Type + Clone> ToFreeVariableTokens for F {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        let parsed: syn::Item = syn::parse_str(&self().definition).unwrap();
        (Some(quote!(#parsed)), None)
    }
}

impl<F: Fn() -> Type + Clone> FreeVariable<()> for F {}

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

type FreeVariables = Vec<(String, (Option<TokenStream>, Option<TokenStream>))>;

pub trait IntoQuoted<'a, T>: FnOnce(&mut String, &mut FreeVariables, bool) -> T + 'a
where
    Self: Sized,
{
    fn to_quoted(self) -> QuotedExpr<T> {
        let mut str = String::new();
        let mut free_variables = Vec::new();
        // this is an uninit value so we can't drop it
        std::mem::forget(self(&mut str, &mut free_variables, false));
        QuotedExpr::create(&str, free_variables)
    }
}

impl<'a, T, F: FnOnce(&mut String, &mut FreeVariables, bool) -> T + 'a> IntoQuoted<'a, T> for F {}

pub trait IntoQuotedWithContext<'a, T>:
    FnMut(&mut String, &mut FreeVariables, bool) -> T + 'a
where
    Self: Sized,
{
    fn to_quoted(mut self) -> QuotedExpr<T> {
        let mut str = String::new();
        let mut free_variables = Vec::new();
        // this is an uninit value so we can't drop it
        std::mem::forget(self(&mut str, &mut free_variables, false));
        QuotedExpr::create(&str, free_variables)
    }
}

impl<'a, T, F: FnMut(&mut String, &mut FreeVariables, bool) -> T + 'a> IntoQuotedWithContext<'a, T>
    for F
{
}

pub struct QuotedExpr<T> {
    expr: syn::Expr,
    free_variables: FreeVariables,
    _phantom: PhantomData<T>,
}

impl<T> QuotedExpr<T> {
    pub fn create(expr: &str, free_variables: FreeVariables) -> QuotedExpr<T> {
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
        let instantiated_free_variables = self.free_variables.iter().flat_map(|(ident, value)| {
            let ident = syn::Ident::new(ident, Span::call_site());
            value.0.iter().map(|prelude| quote!(#prelude)).chain(
                value
                    .1
                    .iter()
                    .map(move |value| quote!(let #ident = #value;)),
            )
        });

        let expr = &self.expr;
        tokens.extend(quote!({
            #(#instantiated_free_variables)*
            #expr
        }));
    }
}

pub struct HydroflowContext<'a> {
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a> HydroflowContext<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> HydroflowContext<'a> {
        HydroflowContext {
            _phantom: PhantomData,
        }
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext {
            _phantom: PhantomData,
        }
    }

    pub fn source_stream<
        T,
        E: hydroflow::futures::stream::Stream<Item = T> + ::std::marker::Unpin,
    >(
        &self,
        e: impl IntoQuoted<'a, E>,
    ) -> HydroflowNode<'a, T> {
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

        HydroflowNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn source_iter<T, E: IntoIterator<Item = T>>(
        &self,
        e: impl IntoQuoted<'a, E>,
    ) -> HydroflowNode<'a, T> {
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

        HydroflowNode {
            ident,
            _phantom: PhantomData,
        }
    }
}

pub struct HydroflowNode<'a, T> {
    ident: syn::Ident,
    _phantom: PhantomData<&'a mut &'a T>,
}

impl<'a, T> HydroflowNode<'a, T> {
    pub fn source_iter<E: IntoIterator<Item = T>>(
        e: impl IntoQuoted<'a, E>,
    ) -> HydroflowNode<'a, T> {
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

        HydroflowNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn map<U, F: Fn(T) -> U + 'a>(
        &self,
        f: impl IntoQuotedWithContext<'a, F>,
    ) -> HydroflowNode<'a, U> {
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

        HydroflowNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        &self,
        f: impl IntoQuotedWithContext<'a, F>,
    ) -> HydroflowNode<'a, T> {
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

        HydroflowNode {
            ident,
            _phantom: PhantomData,
        }
    }

    pub fn for_each<F: Fn(T) + 'a>(&self, f: impl IntoQuotedWithContext<'a, F>) {
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

impl<'a, K, V1> HydroflowNode<'a, (K, V1)> {
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
