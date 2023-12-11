use std::hash::Hash;
use std::marker::PhantomData;

use proc_macro2::Span;
use stageleft::{IntoQuotedMut, Quoted};
use syn::parse_quote;

use crate::HfBuilder;

pub struct HfStream<'a, T> {
    pub(crate) ident: syn::Ident,
    pub(crate) node_id: usize,
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
            .builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node_id)
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> map(#f) -> tee();
            });

        HfStream {
            ident,
            node_id: self.node_id,
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
            .builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node_id)
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> filter(#f) -> tee();
            });

        HfStream {
            ident,
            node_id: self.node_id,
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
            .builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node_id)
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> filter_map(#f) -> tee();
            });

        HfStream {
            ident,
            node_id: self.node_id,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }

    pub fn fold<A, I: Fn() -> A + 'a, C: Fn(&mut A, T)>(
        &self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> HfStream<'a, A> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };
        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let init = init.splice();
        let comb = comb.splice();

        self.graph
            .builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node_id)
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> fold(#init, #comb) -> tee();
            });

        HfStream {
            ident,
            node_id: self.node_id,
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
            .builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node_id)
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> persist() -> tee();
            });

        HfStream {
            ident,
            node_id: self.node_id,
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
            .builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node_id)
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> multiset_delta() -> tee();
            });

        HfStream {
            ident,
            node_id: self.node_id,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }

    pub fn unique(&self) -> HfStream<'a, T>
    where
        T: Eq + Hash,
    {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        self.graph
            .builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node_id)
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> unique::<'tick>() -> tee();
            });

        HfStream {
            ident,
            node_id: self.node_id,
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

        let mut builders = self.graph.builders.borrow_mut();
        let builder = builders.as_mut().unwrap().entry(self.node_id).or_default();

        builder.add_statement(parse_quote! {
            #ident = cross_join::<'tick, 'tick>() -> tee();
        });

        builder.add_statement(parse_quote! {
            #self_ident -> [0]#ident;
        });

        builder.add_statement(parse_quote! {
            #other_ident -> [1]#ident;
        });

        HfStream {
            ident,
            node_id: self.node_id,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }

    pub fn union(&self, other: &HfStream<T>) -> HfStream<'a, T> {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let other_ident = &other.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        let mut builders = self.graph.builders.borrow_mut();
        let builder = builders.as_mut().unwrap().entry(self.node_id).or_default();

        builder.add_statement(parse_quote! {
            #ident = union() -> tee();
        });

        builder.add_statement(parse_quote! {
            #self_ident -> [0]#ident;
        });

        builder.add_statement(parse_quote! {
            #other_ident -> [1]#ident;
        });

        HfStream {
            ident,
            node_id: self.node_id,
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
            .builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node_id)
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> for_each(#f);
            });
    }
}

impl<'a, K, V1> HfStream<'a, (K, V1)> {
    pub fn join<V2>(&self, n: &HfStream<(K, V2)>) -> HfStream<'a, (K, (V1, V2))>
    where
        K: Eq + Hash,
    {
        let next_id = {
            let mut next_id = self.graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let other_ident = &n.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        let mut builders = self.graph.builders.borrow_mut();
        let builder = builders.as_mut().unwrap().entry(self.node_id).or_default();

        builder.add_statement(parse_quote! {
            #ident = join::<'tick, 'tick>() -> tee();
        });

        builder.add_statement(parse_quote! {
            #self_ident -> [0]#ident;
        });

        builder.add_statement(parse_quote! {
            #other_ident -> [1]#ident;
        });

        HfStream {
            ident,
            node_id: self.node_id,
            graph: self.graph,
            _phantom: PhantomData,
        }
    }
}
