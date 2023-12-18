use std::cell::RefCell;
use std::hash::Hash;
use std::io;
use std::marker::PhantomData;

use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::futures::Sink;
use proc_macro2::Span;
use quote::quote;
use serde::de::DeserializeOwned;
use serde::Serialize;
use stageleft::{IntoQuotedMut, Quoted};
use syn::parse_quote;

use crate::builder::Builders;
use crate::node::{HfNode, HfSendTo};

pub struct HfStream<'a, T, N: HfNode<'a>> {
    pub(crate) ident: syn::Ident,
    pub(crate) node: N,
    pub(crate) next_id: &'a RefCell<usize>,
    pub(crate) builders: &'a Builders,
    pub(crate) _phantom: PhantomData<&'a mut &'a T>,
}

impl<'a, T, N: HfNode<'a>> HfStream<'a, T, N> {
    pub fn map<U, F: Fn(T) -> U + 'a>(&self, f: impl IntoQuotedMut<'a, F>) -> HfStream<'a, U, N> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let f = f.splice();

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> map(#f) -> tee();
            });

        HfStream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn inspect<F: Fn(&T) + 'a>(&self, f: impl IntoQuotedMut<'a, F>) -> HfStream<'a, T, N> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };
        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let f = f.splice();

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> inspect(#f) -> tee();
            });

        HfStream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        &self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> HfStream<'a, T, N> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let f = f.splice();

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> filter(#f) -> tee();
            });

        HfStream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        &self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> HfStream<'a, U, N> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let f = f.splice();

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> filter_map(#f) -> tee();
            });

        HfStream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn fold<A, I: Fn() -> A + 'a, C: Fn(&mut A, T)>(
        &self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> HfStream<'a, A, N> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };
        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let init = init.splice();
        let comb = comb.splice();

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> fold(#init, #comb) -> tee();
            });

        HfStream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn persist(&self) -> HfStream<'a, T, N> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> persist() -> tee();
            });

        HfStream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn delta(&self) -> HfStream<'a, T, N> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> multiset_delta() -> tee();
            });

        HfStream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn unique(&self) -> HfStream<'a, T, N>
    where
        T: Eq + Hash,
    {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #self_ident -> unique::<'tick>() -> tee();
            });

        HfStream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn cross_product<O>(&self, other: &HfStream<'a, O, N>) -> HfStream<'a, (T, O), N> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let other_ident = &other.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        let mut builders = self.builders.borrow_mut();
        let builder = builders
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default();

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
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn union(&self, other: &HfStream<'a, T, N>) -> HfStream<'a, T, N> {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let other_ident = &other.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        let mut builders = self.builders.borrow_mut();
        let builder = builders
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default();

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
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn for_each<F: Fn(T) + 'a>(&self, f: impl IntoQuotedMut<'a, F>) {
        let self_ident = &self.ident;
        let f = f.splice();

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #self_ident -> for_each(#f);
            });
    }

    pub fn dest_sink<S: Unpin + Sink<T> + 'a>(&self, sink: impl Quoted<'a, S>) {
        let self_ident = &self.ident;
        let sink = sink.splice();

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #self_ident -> dest_sink(#sink);
            });
    }
}

impl<'a, N: HfNode<'a>> HfStream<'a, Bytes, N> {
    pub fn send_bytes<N2: HfNode<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, Result<BytesMut, io::Error>, N2>
    where
        N: HfSendTo<'a, N2>,
    {
        let self_ident = &self.ident;

        let mut builders_borrowed = self.builders.borrow_mut();
        let builders = builders_borrowed.as_mut().unwrap();

        let source_name = self.node.next_port();
        let self_sink = self.node.gen_sink_statement(&source_name);

        builders
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #self_ident -> #self_sink;
            });

        let recipient_next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", recipient_next_id), Span::call_site());

        let recipient_port_name = other.next_port();
        let recipient_source = N::gen_source_statement(other, &recipient_port_name);

        builders
            .entry(other.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #recipient_source -> tee();
            });

        self.node.send_to(other, &source_name, &recipient_port_name);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Serialize + DeserializeOwned, N: HfNode<'a>> HfStream<'a, T, N> {
    pub fn send_bincode<N2: HfNode<'a>>(&self, other: &N2) -> HfStream<'a, T, N2>
    where
        N: HfSendTo<'a, N2>,
    {
        let self_ident = &self.ident;

        let mut builders_borrowed = self.builders.borrow_mut();
        let builders = builders_borrowed.as_mut().unwrap();

        let source_name = self.node.next_port();
        let self_sink = self.node.gen_sink_statement(&source_name);

        let hydroflow_crate = proc_macro_crate::crate_name("hydroflow_plus")
            .expect("hydroflow_plus should be present in `Cargo.toml`");
        let root = match hydroflow_crate {
            proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
            proc_macro_crate::FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, Span::call_site());
                quote! { #ident }
            }
        };

        // TODO(shadaj): this may fail when instantiated in an environment with different deps
        let t_type: syn::Type = syn::parse_str(std::any::type_name::<T>()).unwrap();

        builders
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #self_ident -> map(|data| {
                    #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into()
                }) -> #self_sink;
            });

        let recipient_next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", recipient_next_id), Span::call_site());

        let recipient_port_name = other.next_port();
        let recipient_source = N::gen_source_statement(other, &recipient_port_name);

        builders
            .entry(other.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #recipient_source -> map(|b| {
                    #root::runtime_support::bincode::deserialize::<#t_type>(&b.unwrap()).unwrap()
                }) -> tee();
            });

        self.node.send_to(other, &source_name, &recipient_port_name);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }
}

impl<'a, K, V1, N: HfNode<'a>> HfStream<'a, (K, V1), N> {
    pub fn join<V2>(&self, n: &HfStream<'a, (K, V2), N>) -> HfStream<'a, (K, (V1, V2)), N>
    where
        K: Eq + Hash,
    {
        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ident;
        let other_ident = &n.ident;
        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        let mut builders = self.builders.borrow_mut();
        let builder = builders
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default();

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
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }
}
