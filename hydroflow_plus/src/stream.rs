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
use stageleft::{q, IntoQuotedMut, Quoted};
use syn::parse_quote;
use syn::visit_mut::VisitMut;

use crate::builder::Builders;
use crate::location::{
    Cluster, HfSendManyToMany, HfSendManyToOne, HfSendOneToMany, HfSendOneToOne, Location,
};

/// Marks the stream as being asynchronous, which means the presence
/// of all elements is directly influenced by the runtime's batching
/// behavior. Aggregation operations are not permitted on streams
/// with this tag because the developer has not explicitly specified
/// if they want to aggregate over the entire stream or just the
/// current batch.
pub struct Async {}

/// Marks the stream as being windowed, which means the developer has
/// opted-into either a batched or persistent windowing semantics.
/// Aggregation operations are permitted on streams with this tag.
pub struct Windowed {}

/// An infinite stream of elements of type `T`.
///
/// Type Parameters:
/// - `'a`: the lifetime of the final Hydroflow graph, which constraints
///   which values can be captured in closures passed to operators
/// - `T`: the type of elements in the stream
/// - `W`: the windowing semantics of the stream, which is either [`Async`]
///    or [`Windowed`]
/// - `N`: the type of the node that the stream is materialized on
pub struct Stream<'a, T, W, N: Location<'a>> {
    pub(crate) ident: syn::Ident,
    pub(crate) node: N,
    pub(crate) next_id: &'a RefCell<usize>,
    pub(crate) builders: &'a Builders,

    /// Does this stream represent a delta of the underlying data?
    /// i.e. is the true stream supposed to have a `persist()` at the end
    pub(crate) is_delta: bool,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), T, W)>,
}

impl<'a, T, W, N: Location<'a>> Stream<'a, T, W, N> {
    fn pipeline_op<U, W2>(&self, pipeline: Pipeline, produces_delta: bool) -> Stream<'a, U, W2, N> {
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

        Stream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: produces_delta,
            _phantom: PhantomData,
        }
    }

    /// Reifies the stream to ensure that it is not producing deltas.
    pub(crate) fn ensure_concrete(&self) -> Stream<'a, T, Windowed, N> {
        if self.is_delta {
            self.pipeline_op(parse_quote!(persist()), false)
        } else {
            self.assume_windowed()
        }
    }

    pub fn map<U, F: Fn(T) -> U + 'a>(&self, f: impl IntoQuotedMut<'a, F>) -> Stream<'a, U, W, N> {
        let f = f.splice();
        self.pipeline_op(parse_quote!(map(#f)), self.is_delta)
    }

    pub fn flat_map<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        &self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, N> {
        let f = f.splice();
        self.pipeline_op(parse_quote!(flat_map(#f)), self.is_delta)
    }

    pub fn enumerate(&self) -> Stream<'a, (usize, T), W, N> {
        self.ensure_concrete()
            .pipeline_op(parse_quote!(enumerate()), self.is_delta)
    }

    pub fn inspect<F: Fn(&T) + 'a>(&self, f: impl IntoQuotedMut<'a, F>) -> Stream<'a, T, W, N> {
        let f = f.splice();
        self.ensure_concrete()
            .pipeline_op(parse_quote!(inspect(#f)), self.is_delta)
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        &self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, T, W, N> {
        let f = f.splice();
        self.pipeline_op(parse_quote!(filter(#f)), self.is_delta)
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        &self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, N> {
        let f = f.splice();
        self.pipeline_op(parse_quote!(filter_map(#f)), self.is_delta)
    }

    // TODO(shadaj): should allow for differing windows, using strongest one
    pub fn cross_product<O>(&self, other: &Stream<'a, O, W, N>) -> Stream<'a, (T, O), W, N> {
        if self.node.id() != other.node.id() {
            panic!("cross_product must be called on streams on the same node");
        }

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

        let output_delta = match (self.is_delta, other.is_delta) {
            (true, true) => {
                builder.add_statement(parse_quote! {
                    #ident = cross_join::<'static, 'static>() -> tee();
                });

                false // TODO(shadaj): cross_join already replays?
            }
            (true, false) => {
                builder.add_statement(parse_quote! {
                    #ident = cross_join::<'static, 'tick>() -> tee();
                });

                false
            }
            (false, true) => {
                builder.add_statement(parse_quote! {
                    #ident = cross_join::<'tick, 'static>() -> tee();
                });

                false
            }
            (false, false) => {
                builder.add_statement(parse_quote! {
                    #ident = cross_join::<'tick, 'tick>() -> tee();
                });

                false
            }
        };

        builder.add_statement(parse_quote! {
            #self_ident -> [0]#ident;
        });

        builder.add_statement(parse_quote! {
            #other_ident -> [1]#ident;
        });

        Stream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: output_delta,
            _phantom: PhantomData,
        }
    }

    pub fn union(&self, other: &Stream<'a, T, W, N>) -> Stream<'a, T, W, N> {
        if self.node.id() != other.node.id() {
            panic!("union must be called on streams on the same node");
        }

        let next_id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let self_ident = &self.ensure_concrete().ident;
        let other_ident = &other.ensure_concrete().ident;
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

        Stream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: false,
            _phantom: PhantomData,
        }
    }

    pub fn for_each<F: Fn(T) + 'a>(&self, f: impl IntoQuotedMut<'a, F>) {
        let self_ident = &self.ensure_concrete().ident;
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
        let self_ident = &self.ensure_concrete().ident;
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

    pub fn all_ticks(&self) -> Stream<'a, T, Windowed, N> {
        if self.is_delta {
            self.ensure_concrete().all_ticks()
        } else {
            Stream {
                ident: self.ident.clone(),
                node: self.node.clone(),
                next_id: self.next_id,
                builders: self.builders,
                is_delta: true,
                _phantom: PhantomData,
            }
        }
    }

    pub fn assume_windowed(&self) -> Stream<'a, T, Windowed, N> {
        Stream {
            ident: self.ident.clone(),
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, N: Location<'a>> Stream<'a, T, Async, N> {
    pub fn tick_batch(&self) -> Stream<'a, T, Windowed, N> {
        Stream {
            ident: self.ident.clone(),
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, N: Location<'a>> Stream<'a, T, Windowed, N> {
    pub fn fold<A, I: Fn() -> A + 'a, C: Fn(&mut A, T)>(
        &self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, A, Windowed, N> {
        let init = init.splice();
        let comb = comb.splice();

        if self.is_delta {
            self.pipeline_op(parse_quote!(fold::<'static>(#init, #comb)), false)
        } else {
            self.pipeline_op(parse_quote!(fold::<'tick>(#init, #comb)), false)
        }
    }

    pub fn delta(&self) -> Stream<'a, T, Windowed, N> {
        if self.is_delta {
            Stream {
                ident: self.ident.clone(),
                node: self.node.clone(),
                next_id: self.next_id,
                builders: self.builders,
                is_delta: false,
                _phantom: PhantomData,
            }
        } else {
            self.pipeline_op(parse_quote!(multiset_delta()), false)
        }
    }

    pub fn unique(&self) -> Stream<'a, T, Windowed, N>
    where
        T: Eq + Hash,
    {
        self.ensure_concrete()
            .pipeline_op(parse_quote!(unique::<'tick>()), false)
    }

    pub fn sample_every(
        &self,
        duration: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<'a, T, Windowed, N> {
        self.cross_product(&self.node.source_interval(duration).tick_batch())
            .map(q!(|(a, _)| a))
    }
}

impl<'a, T: Clone, W, N: Location<'a>> Stream<'a, &T, W, N> {
    pub fn cloned(&self) -> Stream<'a, T, W, N> {
        self.pipeline_op(parse_quote!(map(|d| d.clone())), self.is_delta)
    }
}

impl<'a, K, V1, W, N: Location<'a>> Stream<'a, (K, V1), W, N> {
    // TODO(shadaj): figure out window semantics
    pub fn join<W2, V2>(&self, n: &Stream<'a, (K, V2), W2, N>) -> Stream<'a, (K, (V1, V2)), W, N>
    where
        K: Eq + Hash,
    {
        if self.node.id() != n.node.id() {
            panic!("join must be called on streams on the same node");
        }

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

        let output_delta = match (self.is_delta, n.is_delta) {
            (true, true) => {
                builder.add_statement(parse_quote! {
                    #ident = join::<'static, 'static>() -> tee();
                });

                false // TODO(shadaj): join already replays?
            }
            (true, false) => {
                builder.add_statement(parse_quote! {
                    #ident = join::<'static, 'tick>() -> tee();
                });

                false
            }
            (false, true) => {
                builder.add_statement(parse_quote! {
                    #ident = join::<'tick, 'static>() -> tee();
                });

                false
            }
            (false, false) => {
                builder.add_statement(parse_quote! {
                    #ident = join::<'tick, 'tick>() -> tee();
                });

                false
            }
        };

        builder.add_statement(parse_quote! {
            #self_ident -> [0]#ident;
        });

        builder.add_statement(parse_quote! {
            #other_ident -> [1]#ident;
        });

        Stream {
            ident,
            node: self.node.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: output_delta,
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

// TODO(shadaj): has to be public due to temporary stageleft limitations
/// Rewrites use of alloc::string::* to use std::string::*
pub struct RewriteAlloc {}
impl VisitMut for RewriteAlloc {
    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        if i.segments.iter().take(2).collect::<Vec<_>>()
            == vec![
                &syn::PathSegment::from(syn::Ident::new("alloc", Span::call_site())),
                &syn::PathSegment::from(syn::Ident::new("string", Span::call_site())),
            ]
        {
            *i.segments.first_mut().unwrap() =
                syn::PathSegment::from(syn::Ident::new("std", Span::call_site()));
        }
    }
}

fn node_send_direct<'a, T, W, N: Location<'a>>(me: &Stream<'a, T, W, N>, sink: Pipeline) {
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

fn node_send_bincode<'a, T: Serialize, W, N: Location<'a>>(
    me: &Stream<'a, T, W, N>,
    sink: Pipeline,
) {
    let self_ident = &me.ident;

    let mut builders_borrowed = me.builders.borrow_mut();
    let builders = builders_borrowed.as_mut().unwrap();

    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let mut t_type: syn::Type = syn::parse_str(std::any::type_name::<T>()).unwrap();
    RewriteAlloc {}.visit_type_mut(&mut t_type);

    builders
        .entry(me.node.id())
        .or_default()
        .add_statement(parse_quote! {
            #self_ident -> map(|data| {
                #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into()
            }) -> #sink;
        });
}

fn cluster_demux_bincode<'a, T, W, N: Location<'a>>(
    me: &Stream<'a, (u32, T), W, N>,
    sink: Pipeline,
) {
    let self_ident = &me.ident;

    let mut builders_borrowed = me.builders.borrow_mut();
    let builders = builders_borrowed.as_mut().unwrap();

    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let mut t_type: syn::Type = syn::parse_str(std::any::type_name::<T>()).unwrap();
    RewriteAlloc {}.visit_type_mut(&mut t_type);

    builders
        .entry(me.node.id())
        .or_default()
        .add_statement(parse_quote! {
            #self_ident -> map(|(id, data)| {
                (id, #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into())
            }) -> #sink;
        });
}

fn node_recv_direct<'a, T, W, N: Location<'a>, N2: Location<'a>>(
    me: &Stream<'a, T, W, N>,
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

fn node_recv_bincode<'a, T1, T2: DeserializeOwned, W, N: Location<'a>, N2: Location<'a>>(
    me: &Stream<'a, T1, W, N>,
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
    let mut t_type: syn::Type = syn::parse_str(std::any::type_name::<T2>()).unwrap();
    RewriteAlloc {}.visit_type_mut(&mut t_type);

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

impl<'a, W, N: Location<'a>> Stream<'a, Bytes, W, N> {
    pub fn send_bytes<N2: Location<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_direct(self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn send_bytes_tagged<N2: Location<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_direct(self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn broadcast_bytes<N2: Location<'a> + Cluster<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned();
        other_ids
            .cross_product(&self.assume_windowed())
            .demux_bytes(other)
    }

    pub fn broadcast_bytes_tagged<N2: Location<'a> + Cluster<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned();
        other_ids
            .cross_product(&self.assume_windowed())
            .demux_bytes_tagged(other)
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: Location<'a>> Stream<'a, T, W, N> {
    pub fn send_bincode<N2: Location<'a>>(&self, other: &N2) -> Stream<'a, T, Async, N2>
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

        Stream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn send_bincode_tagged<N2: Location<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, (u32, T), Async, N2>
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

        Stream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn broadcast_bincode<N2: Location<'a> + Cluster<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, T, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned();
        other_ids
            .cross_product(&self.assume_windowed())
            .demux_bincode(other)
    }

    pub fn broadcast_bincode_tagged<N2: Location<'a> + Cluster<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned();
        other_ids
            .cross_product(&self.assume_windowed())
            .demux_bincode_tagged(other)
    }
}

impl<'a, W, N: Location<'a>> Stream<'a, (u32, Bytes), W, N> {
    pub fn demux_bytes<N2: Location<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_direct(self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: Location<'a>> Stream<'a, (u32, T), W, N> {
    pub fn demux_bincode<N2: Location<'a>>(&self, other: &N2) -> Stream<'a, T, Async, N2>
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

        Stream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, W, N: Location<'a>> Stream<'a, (u32, Bytes), W, N> {
    pub fn demux_bytes_tagged<N2: Location<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let ident = node_recv_direct(self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: Location<'a>> Stream<'a, (u32, T), W, N> {
    pub fn demux_bincode_tagged<N2: Location<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, (u32, T), Async, N2>
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

        Stream {
            ident,
            node: other.clone(),
            next_id: self.next_id,
            builders: self.builders,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}
