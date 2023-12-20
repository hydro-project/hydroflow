use std::cell::RefCell;
use std::hash::Hash;
use std::io;
use std::marker::PhantomData;

use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::futures::Sink;
use hydroflow_lang::parse::Pipeline;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::de::DeserializeOwned;
use serde::Serialize;
use stageleft::{IntoQuotedMut, Quoted};
use syn::parse_quote;

use crate::builder::Builders;
use crate::node::{
    HfCluster, HfNode, HfSendManyToMany, HfSendManyToOne, HfSendOneToMany, HfSendOneToOne,
};

pub struct Async {}
pub struct Windowed {}

pub struct HfStream<'a, T, W, N: HfNode<'a>> {
    pub(crate) ident: syn::Ident,
    pub(crate) node: N,
    pub(crate) next_id: &'a RefCell<usize>,
    pub(crate) builders: &'a Builders,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), T, W)>,
}

impl<'a, T, W, N: HfNode<'a>> HfStream<'a, T, W, N> {
    fn pipeline_op<U, W2>(&self, pipeline: Pipeline) -> HfStream<'a, U, W2, N> {
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
                #ident = #self_ident -> #pipeline -> tee();
            });

        HfStream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn map<U, F: Fn(T) -> U + 'a>(
        &self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> HfStream<'a, U, W, N> {
        let f = f.splice();
        self.pipeline_op(parse_quote!(map(#f)))
    }

    pub fn enumerate(&self) -> HfStream<'a, (usize, T), W, N> {
        self.pipeline_op(parse_quote!(enumerate()))
    }

    pub fn inspect<F: Fn(&T) + 'a>(&self, f: impl IntoQuotedMut<'a, F>) -> HfStream<'a, T, W, N> {
        let f = f.splice();
        self.pipeline_op(parse_quote!(inspect(#f)))
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        &self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> HfStream<'a, T, W, N> {
        let f = f.splice();
        self.pipeline_op(parse_quote!(filter(#f)))
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        &self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> HfStream<'a, U, W, N> {
        let f = f.splice();
        self.pipeline_op(parse_quote!(filter_map(#f)))
    }

    pub fn persist(&self) -> HfStream<'a, T, Windowed, N> {
        self.pipeline_op(parse_quote!(persist()))
    }

    // TODO(shadaj): should allow for differing windows, using strongest one
    pub fn cross_product<O>(&self, other: &HfStream<'a, O, W, N>) -> HfStream<'a, (T, O), W, N> {
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

    pub fn union(&self, other: &HfStream<'a, T, W, N>) -> HfStream<'a, T, W, N> {
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

    pub fn assume_windowed(&self) -> HfStream<'a, T, Windowed, N> {
        HfStream {
            ident: self.ident.clone(),
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, N: HfNode<'a>> HfStream<'a, T, Async, N> {
    pub fn batched(&self) -> HfStream<'a, T, Windowed, N> {
        HfStream {
            ident: self.ident.clone(),
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, N: HfNode<'a>> HfStream<'a, T, Windowed, N> {
    pub fn fold<A, I: Fn() -> A + 'a, C: Fn(&mut A, T)>(
        &self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> HfStream<'a, A, Windowed, N> {
        let init = init.splice();
        let comb = comb.splice();
        self.pipeline_op(parse_quote!(fold::<'tick>(#init, #comb)))
    }

    pub fn delta(&self) -> HfStream<'a, T, Windowed, N> {
        self.pipeline_op(parse_quote!(multiset_delta()))
    }

    pub fn unique(&self) -> HfStream<'a, T, Windowed, N>
    where
        T: Eq + Hash,
    {
        self.pipeline_op(parse_quote!(unique::<'tick>()))
    }
}

impl<'a, T: Clone, W, N: HfNode<'a>> HfStream<'a, &T, W, N> {
    pub fn cloned(&self) -> HfStream<'a, T, W, N> {
        self.pipeline_op(parse_quote!(map(|d| d.clone())))
    }
}

impl<'a, K, V1, W, N: HfNode<'a>> HfStream<'a, (K, V1), W, N> {
    // TODO(shadaj): figure out window semantics
    pub fn join<W2, V2>(
        &self,
        n: &HfStream<'a, (K, V2), W2, N>,
    ) -> HfStream<'a, (K, (V1, V2)), W, N>
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

fn get_this_crate() -> TokenStream {
    let hydroflow_crate = proc_macro_crate::crate_name("hydroflow_plus")
        .expect("hydroflow_plus should be present in `Cargo.toml`");
    match hydroflow_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    }
}

fn node_send_direct<'a, T, W, N: HfNode<'a>>(me: &HfStream<'a, T, W, N>, sink: Pipeline) {
    let self_ident = &me.ident;

    let mut builders_borrowed = me.builders.borrow_mut();
    let builders = builders_borrowed.as_mut().unwrap();

    builders
        .entry(me.node.id())
        .or_default()
        .add_statement(parse_quote! {
            #self_ident -> #sink;
        });
}

fn node_send_bincode<'a, T: Serialize, W, N: HfNode<'a>>(
    me: &HfStream<'a, T, W, N>,
    sink: Pipeline,
) {
    let self_ident = &me.ident;

    let mut builders_borrowed = me.builders.borrow_mut();
    let builders = builders_borrowed.as_mut().unwrap();

    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let t_type: syn::Type = syn::parse_str(std::any::type_name::<T>()).unwrap();

    builders
        .entry(me.node.id())
        .or_default()
        .add_statement(parse_quote! {
            #self_ident -> map(|data| {
                #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into()
            }) -> #sink;
        });
}

fn cluster_demux_bincode<'a, T, W, N: HfNode<'a>>(
    me: &HfStream<'a, (u32, T), W, N>,
    sink: Pipeline,
) {
    let self_ident = &me.ident;

    let mut builders_borrowed = me.builders.borrow_mut();
    let builders = builders_borrowed.as_mut().unwrap();

    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let t_type: syn::Type = syn::parse_str(std::any::type_name::<T>()).unwrap();

    builders
        .entry(me.node.id())
        .or_default()
        .add_statement(parse_quote! {
            #self_ident -> map(|(id, data)| {
                (id, #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into())
            }) -> #sink;
        });
}

fn node_recv_direct<'a, T, W, N: HfNode<'a>, N2: HfNode<'a>>(
    me: &HfStream<'a, T, W, N>,
    other: &N2,
    source: Pipeline,
) -> syn::Ident {
    let recipient_next_id = {
        let mut next_id = me.next_id.borrow_mut();
        let id = *next_id;
        *next_id += 1;
        id
    };

    let ident = syn::Ident::new(&format!("stream_{}", recipient_next_id), Span::call_site());

    let mut builders_borrowed = me.builders.borrow_mut();
    let builders = builders_borrowed.as_mut().unwrap();

    builders
        .entry(other.id())
        .or_default()
        .add_statement(parse_quote! {
            #ident = #source -> tee();
        });

    ident
}

fn node_recv_bincode<'a, T1, T2: DeserializeOwned, W, N: HfNode<'a>, N2: HfNode<'a>>(
    me: &HfStream<'a, T1, W, N>,
    other: &N2,
    source: Pipeline,
    tagged: bool,
) -> syn::Ident {
    let recipient_next_id = {
        let mut next_id = me.next_id.borrow_mut();
        let id = *next_id;
        *next_id += 1;
        id
    };

    let ident = syn::Ident::new(&format!("stream_{}", recipient_next_id), Span::call_site());

    let mut builders_borrowed = me.builders.borrow_mut();
    let builders = builders_borrowed.as_mut().unwrap();

    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let t_type: syn::Type = syn::parse_str(std::any::type_name::<T2>()).unwrap();

    builders.entry(other.id()).or_default().add_statement({
        if tagged {
            parse_quote! {
                #ident = #source -> map(|res| {
                    let (id, b) = res.unwrap();
                    (id, #root::runtime_support::bincode::deserialize::<#t_type>(&b).unwrap())
                }) -> tee();
            }
        } else {
            parse_quote! {
                #ident = #source -> map(|res| {
                    #root::runtime_support::bincode::deserialize::<#t_type>(&res.unwrap()).unwrap()
                }) -> tee();
            }
        }
    });

    ident
}

impl<'a, W, N: HfNode<'a>> HfStream<'a, Bytes, W, N> {
    pub fn send_bytes<N2: HfNode<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_direct(self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn send_bytes_tagged<N2: HfNode<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_direct(self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn broadcast_bytes<N2: HfNode<'a> + HfCluster<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned();
        other_ids
            .cross_product(&self.assume_windowed())
            .demux_bytes(other)
    }

    pub fn broadcast_bytes_tagged<N2: HfNode<'a> + HfCluster<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned();
        other_ids
            .cross_product(&self.assume_windowed())
            .demux_bytes_tagged(other)
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: HfNode<'a>> HfStream<'a, T, W, N> {
    pub fn send_bincode<N2: HfNode<'a>>(&self, other: &N2) -> HfStream<'a, T, Async, N2>
    where
        N: HfSendOneToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_bincode(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_bincode::<_, T, _, _, _>(
            self,
            other,
            N::gen_source_statement(other, &recv_port),
            false,
        );

        self.node.connect(other, &send_port, &recv_port);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn send_bincode_tagged<N2: HfNode<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_bincode(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_bincode::<_, T, _, _, _>(
            self,
            other,
            N::gen_source_statement(other, &recv_port),
            true,
        );

        self.node.connect(other, &send_port, &recv_port);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }

    pub fn broadcast_bincode<N2: HfNode<'a> + HfCluster<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, T, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned();
        other_ids
            .cross_product(&self.assume_windowed())
            .demux_bincode(other)
    }

    pub fn broadcast_bincode_tagged<N2: HfNode<'a> + HfCluster<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned();
        other_ids
            .cross_product(&self.assume_windowed())
            .demux_bincode_tagged(other)
    }
}

impl<'a, W, N: HfNode<'a>> HfStream<'a, (u32, Bytes), W, N> {
    pub fn demux_bytes<N2: HfNode<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_direct(self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: HfNode<'a>> HfStream<'a, (u32, T), W, N> {
    pub fn demux_bincode<N2: HfNode<'a>>(&self, other: &N2) -> HfStream<'a, T, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        cluster_demux_bincode(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_bincode::<_, T, _, _, _>(
            self,
            other,
            N::gen_source_statement(other, &recv_port),
            false,
        );

        self.node.connect(other, &send_port, &recv_port);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }
}

impl<'a, W, N: HfNode<'a>> HfStream<'a, (u32, Bytes), W, N> {
    pub fn demux_bytes_tagged<N2: HfNode<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_direct(self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: HfNode<'a>> HfStream<'a, (u32, T), W, N> {
    pub fn demux_bincode_tagged<N2: HfNode<'a>>(
        &self,
        other: &N2,
    ) -> HfStream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        cluster_demux_bincode(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_bincode::<_, T, _, _, _>(
            self,
            other,
            N::gen_source_statement(other, &recv_port),
            true,
        );

        self.node.connect(other, &send_port, &recv_port);

        HfStream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            _phantom: PhantomData,
        }
    }
}
