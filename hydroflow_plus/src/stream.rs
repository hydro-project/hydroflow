use std::marker::PhantomData;

use proc_macro2::Span;
use stageleft::{IntoQuotedMut, Quoted};
use syn::parse_quote;

use crate::HfBuilder;

pub struct HfStream<'a, T> {
    pub(crate) ident: syn::Ident,
    pub(crate) graph: &'a HfBuilder<'a>,
    pub(crate) _phantom: PhantomData<&'a mut &'a T>,
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
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let f = f.splice();

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
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let f = f.splice();

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

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        &self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> HfStream<'a, U> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let f = f.splice();

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = #self_ident -> filter_map(#f) -> tee();
            });

        HfStream {
            ident,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }

    pub fn persist(&self) -> HfStream<'a, T> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = #self_ident -> persist() -> tee();
            });

        HfStream {
            ident,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }

    pub fn delta(&self) -> HfStream<'a, T> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = #self_ident -> multiset_delta() -> tee();
            });

        HfStream {
            ident,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }

    pub fn cross_product<O>(&self, other: &HfStream<O>) -> HfStream<'a, (T, O)> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let other_ident = &other.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = cross_join::<'tick, 'tick>() -> tee();
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

    pub fn for_each<F: Fn(T) + 'a>(&self, f: impl IntoQuotedMut<'a, F>) {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let f = f.splice();

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
    pub fn join<V2>(&self, n: &HfStream<(K, V2)>) -> HfStream<'a, (K, (V1, V2))> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let other_ident = &n.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        self.graph
            .builder
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add_statement(parse_quote! {
                #ident = join::<'tick, 'tick>() -> tee();
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
