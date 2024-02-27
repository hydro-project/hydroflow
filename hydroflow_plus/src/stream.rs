use std::cell::RefCell;
use std::hash::Hash;
use std::io;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

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

use crate::ir::{DebugPipelineFn, HfPlusLeaf, HfPlusNode};
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
    node: N,

    pub(crate) ir_leaves: &'a RefCell<Vec<HfPlusLeaf>>,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    pub(crate) _phantom: PhantomData<(&'a mut &'a (), T, W)>,
}

impl<'a, T, W, N: Location<'a>> Stream<'a, T, W, N> {
    pub(crate) fn new(
        node: N,
        ir_leaves: &'a RefCell<Vec<HfPlusLeaf>>,
        ir_node: HfPlusNode,
    ) -> Self {
        Stream {
            node,
            ir_leaves,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone, W, N: Location<'a>> Clone for Stream<'a, T, W, N> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: Rc::new(RefCell::new(orig_ir_node)),
            };
        }

        Stream::new(
            self.node.clone(),
            self.ir_leaves,
            self.ir_node.borrow().clone(),
        )
    }
}

impl<'a, T, W, N: Location<'a>> Stream<'a, T, W, N> {
    fn pipeline_op<U, W2>(
        self,
        kind: &'static str,
        gen_pipeline: impl Fn(bool) -> Option<(Pipeline, bool)> + 'static,
    ) -> Stream<'a, U, W2, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::PipelineOp {
                kind,
                gen_pipeline: DebugPipelineFn(Rc::new(gen_pipeline)),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Stream<'a, U, W, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Map {
                f: f.splice().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn flat_map<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, N> {
        let f = f.splice();
        self.pipeline_op("flat_map", move |d| Some((parse_quote!(flat_map(#f)), d)))
    }

    pub fn enumerate(self) -> Stream<'a, (usize, T), W, N> {
        self.pipeline_op("enumerate", |d| {
            if d {
                None
            } else {
                Some((parse_quote!(enumerate()), false))
            }
        })
    }

    pub fn inspect<F: Fn(&T) + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Stream<'a, T, W, N> {
        let f = f.splice();
        self.pipeline_op("inspect", move |d| {
            if d {
                None
            } else {
                Some((parse_quote!(inspect(#f)), false))
            }
        })
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, T, W, N> {
        let f = f.splice();
        self.pipeline_op("filter", move |d| Some((parse_quote!(filter(#f)), d)))
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, N> {
        let f = f.splice();
        self.pipeline_op("filter_map", move |d| {
            Some((parse_quote!(filter_map(#f)), d))
        })
    }

    // TODO(shadaj): should allow for differing windows, using strongest one
    pub fn cross_product<O>(self, other: Stream<'a, O, W, N>) -> Stream<'a, (T, O), W, N> {
        if self.node.id() != other.node.id() {
            panic!("cross_product must be called on streams on the same node");
        }

        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::CrossProduct(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn union(self, other: Stream<'a, T, W, N>) -> Stream<'a, T, W, N> {
        if self.node.id() != other.node.id() {
            panic!("union must be called on streams on the same node");
        }

        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Union(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn for_each<F: Fn(T) + 'a>(self, f: impl IntoQuotedMut<'a, F>) {
        self.ir_leaves.borrow_mut().push(HfPlusLeaf::ForEach {
            input: Box::new(self.ir_node.into_inner()),
            f: f.splice().into(),
        });
    }

    pub fn dest_sink<S: Unpin + Sink<T> + 'a>(self, sink: impl Quoted<'a, S>) {
        self.ir_leaves.borrow_mut().push(HfPlusLeaf::DestSink {
            sink: sink.splice().into(),
            input: Box::new(self.ir_node.into_inner()),
        });
    }

    pub fn all_ticks(self) -> Stream<'a, T, Windowed, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn assume_windowed(self) -> Stream<'a, T, Windowed, N> {
        Stream::new(self.node, self.ir_leaves, self.ir_node.into_inner())
    }
}

impl<'a, T, N: Location<'a>> Stream<'a, T, Async, N> {
    pub fn tick_batch(self) -> Stream<'a, T, Windowed, N> {
        Stream::new(self.node, self.ir_leaves, self.ir_node.into_inner())
    }
}

impl<'a, T, N: Location<'a>> Stream<'a, T, Windowed, N> {
    pub fn fold<A, I: Fn() -> A + 'a, C: Fn(&mut A, T)>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, A, Windowed, N> {
        let init = init.splice();
        let comb = comb.splice();

        self.pipeline_op("fold", move |d| {
            if d {
                Some((parse_quote!(fold::<'static>(#init, #comb)), false))
            } else {
                Some((parse_quote!(fold::<'tick>(#init, #comb)), false))
            }
        })
    }

    pub fn reduce<C: Fn(&mut T, T) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, T, Windowed, N> {
        let comb = comb.splice();

        self.pipeline_op("reduce", move |d| {
            if d {
                Some((parse_quote!(reduce::<'static>(#comb)), false))
            } else {
                Some((parse_quote!(reduce::<'tick>(#comb)), false))
            }
        })
    }

    pub fn count(self) -> Stream<'a, usize, Windowed, N> {
        self.fold(q!(|| 0usize), q!(|count, _| *count += 1))
    }

    pub fn delta(self) -> Stream<'a, T, Windowed, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn unique(self) -> Stream<'a, T, Windowed, N>
    where
        T: Eq + Hash,
    {
        self.pipeline_op("unique", |d| {
            if d {
                None
            } else {
                Some((parse_quote!(unique::<'tick>()), false))
            }
        })
    }

    pub fn filter_not_in(self, other: Stream<'a, T, Windowed, N>) -> Stream<'a, T, Windowed, N>
    where
        T: Eq + Hash,
    {
        if self.node.id() != other.node.id() {
            panic!("union must be called on streams on the same node");
        }

        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Difference(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn sample_every(
        self,
        duration: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<'a, T, Windowed, N> {
        let samples = self.node.source_interval(duration).tick_batch();
        self.cross_product(samples).map(q!(|(a, _)| a))
    }
}

impl<'a, T: Clone, W, N: Location<'a>> Stream<'a, &T, W, N> {
    pub fn cloned(self) -> Stream<'a, T, W, N> {
        self.pipeline_op("cloned", |d| Some((parse_quote!(map(|d| d.clone())), d)))
    }
}

impl<'a, K, V1, W, N: Location<'a>> Stream<'a, (K, V1), W, N> {
    // TODO(shadaj): figure out window semantics
    pub fn join<W2, V2>(self, n: Stream<'a, (K, V2), W2, N>) -> Stream<'a, (K, (V1, V2)), W, N>
    where
        K: Eq + Hash,
    {
        if self.node.id() != n.node.id() {
            panic!("join must be called on streams on the same node");
        }

        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Join(
                Box::new(self.ir_node.into_inner()),
                Box::new(n.ir_node.into_inner()),
            ),
        )
    }

    pub fn anti_join<W2>(self, n: Stream<'a, K, W2, N>) -> Stream<'a, (K, V1), W, N>
    where
        K: Eq + Hash,
    {
        if self.node.id() != n.node.id() {
            panic!("anti_join must be called on streams on the same node");
        }

        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::AntiJoin(
                Box::new(self.ir_node.into_inner()),
                Box::new(n.ir_node.into_inner()),
            ),
        )
    }
}

impl<'a, K: Eq + Hash, V, N: Location<'a>> Stream<'a, (K, V), Windowed, N> {
    pub fn fold_keyed<A, I: Fn() -> A + 'a, C: Fn(&mut A, V) + 'a>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, (K, A), Windowed, N> {
        let init = init.splice();
        let comb = comb.splice();

        self.pipeline_op("fold_keyed", move |d| {
            if d {
                Some((parse_quote!(fold_keyed::<'static>(#init, #comb)), false))
            } else {
                Some((parse_quote!(fold_keyed::<'tick>(#init, #comb)), false))
            }
        })
    }

    pub fn reduce_keyed<F: Fn(&mut V, V) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, (K, V), Windowed, N> {
        let comb = comb.splice();

        self.pipeline_op("reduce_keyed", move |d| {
            if d {
                Some((parse_quote!(reduce_keyed::<'static>(#comb)), false))
            } else {
                Some((parse_quote!(reduce_keyed::<'tick>(#comb)), false))
            }
        })
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

fn serialize_bincode<T: Serialize>(is_demux: bool) -> Pipeline {
    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let mut t_type: syn::Type = syn::parse_str(std::any::type_name::<T>()).unwrap();
    RewriteAlloc {}.visit_type_mut(&mut t_type);

    if is_demux {
        parse_quote! {
            map(|(id, data)| {
                (id, #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into())
            })
        }
    } else {
        parse_quote! {
            map(|data| {
                #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into()
            })
        }
    }
}

fn deserialize_bincode<T2: DeserializeOwned>(tagged: bool) -> Pipeline {
    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let mut t_type: syn::Type = syn::parse_str(std::any::type_name::<T2>()).unwrap();
    RewriteAlloc {}.visit_type_mut(&mut t_type);

    if tagged {
        parse_quote! {
            map(|res| {
                let (id, b) = res.unwrap();
                (id, #root::runtime_support::bincode::deserialize::<#t_type>(&b).unwrap())
            })
        }
    } else {
        parse_quote! {
            map(|res| {
                #root::runtime_support::bincode::deserialize::<#t_type>(&res.unwrap()).unwrap()
            })
        }
    }
}

impl<'a, W, N: Location<'a>> Stream<'a, Bytes, W, N> {
    pub fn send_bytes<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        let sink_expr = self.node.gen_sink_statement(&send_port).into();

        let recv_port = other.next_port();
        let source_expr = N::gen_source_statement(other, &recv_port).into();

        self.node.connect(other, &send_port, &recv_port);

        Stream::new(
            other.clone(),
            self.ir_leaves,
            HfPlusNode::Network {
                to_location: other.id(),
                serialize_pipeline: None,
                sink_expr,
                source_expr,
                deserialize_pipeline: None,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn send_bytes_tagged<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        let sink_expr = self.node.gen_sink_statement(&send_port).into();

        let recv_port = other.next_port();
        let source_expr = N::gen_source_statement(other, &recv_port).into();

        self.node.connect(other, &send_port, &recv_port);

        Stream::new(
            other.clone(),
            self.ir_leaves,
            HfPlusNode::Network {
                to_location: other.id(),
                serialize_pipeline: None,
                sink_expr,
                source_expr,
                deserialize_pipeline: None,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn send_bytes_interleaved<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        self.send_bytes_tagged(other).map(q!(|r| r.map(|(_, b)| b)))
    }

    pub fn broadcast_bytes<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned().all_ticks();
        other_ids
            .cross_product(self.assume_windowed())
            .demux_bytes(other)
    }

    pub fn broadcast_bytes_tagged<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned().all_ticks();
        other_ids
            .cross_product(self.assume_windowed())
            .demux_bytes_tagged(other)
    }

    pub fn broadcast_bytes_interleaved<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        self.broadcast_bytes_tagged(other)
            .map(q!(|r| r.map(|(_, b)| b)))
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: Location<'a>> Stream<'a, T, W, N> {
    pub fn send_bincode<N2: Location<'a>>(self, other: &N2) -> Stream<'a, T, Async, N2>
    where
        N: HfSendOneToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        let serialize_pipeline = Some(serialize_bincode::<T>(false));
        let sink_expr = self.node.gen_sink_statement(&send_port).into();

        let recv_port = other.next_port();
        let deserialize_pipeline = Some(deserialize_bincode::<T>(false));
        let source_expr = N::gen_source_statement(other, &recv_port).into();

        self.node.connect(other, &send_port, &recv_port);

        Stream::new(
            other.clone(),
            self.ir_leaves,
            HfPlusNode::Network {
                to_location: other.id(),
                serialize_pipeline,
                sink_expr,
                source_expr,
                deserialize_pipeline,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn send_bincode_tagged<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        let serialize_pipeline = Some(serialize_bincode::<T>(false));
        let sink_expr = self.node.gen_sink_statement(&send_port).into();

        let recv_port = other.next_port();
        let deserialize_pipeline = Some(deserialize_bincode::<T>(true));
        let source_expr = N::gen_source_statement(other, &recv_port).into();

        self.node.connect(other, &send_port, &recv_port);

        Stream::new(
            other.clone(),
            self.ir_leaves,
            HfPlusNode::Network {
                to_location: other.id(),
                serialize_pipeline,
                sink_expr,
                source_expr,
                deserialize_pipeline,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn send_bincode_interleaved<N2: Location<'a>>(self, other: &N2) -> Stream<'a, T, Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        self.send_bincode_tagged(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bincode<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, T, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned().all_ticks();
        other_ids
            .cross_product(self.assume_windowed())
            .demux_bincode(other)
    }

    pub fn broadcast_bincode_tagged<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned().all_ticks();
        other_ids
            .cross_product(self.assume_windowed())
            .demux_bincode_tagged(other)
    }

    pub fn broadcast_bincode_interleaved<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, T, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        self.broadcast_bincode_tagged(other).map(q!(|(_, b)| b))
    }
}

impl<'a, W, N: Location<'a>> Stream<'a, (u32, Bytes), W, N> {
    pub fn demux_bytes<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        let sink_expr = self.node.gen_sink_statement(&send_port).into();

        let recv_port = other.next_port();
        let source_expr = N::gen_source_statement(other, &recv_port).into();

        self.node.connect(other, &send_port, &recv_port);

        Stream::new(
            other.clone(),
            self.ir_leaves,
            HfPlusNode::Network {
                to_location: other.id(),
                serialize_pipeline: None,
                sink_expr,
                source_expr,
                deserialize_pipeline: None,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: Location<'a>> Stream<'a, (u32, T), W, N> {
    pub fn demux_bincode<N2: Location<'a>>(self, other: &N2) -> Stream<'a, T, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        let serialize_pipeline = Some(serialize_bincode::<T>(true));
        let sink_expr = self.node.gen_sink_statement(&send_port).into();

        let recv_port = other.next_port();
        let deserialize_pipeline = Some(deserialize_bincode::<T>(false));
        let source_expr = N::gen_source_statement(other, &recv_port).into();

        self.node.connect(other, &send_port, &recv_port);

        Stream::new(
            other.clone(),
            self.ir_leaves,
            HfPlusNode::Network {
                to_location: other.id(),
                serialize_pipeline,
                sink_expr,
                source_expr,
                deserialize_pipeline,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, W, N: Location<'a>> Stream<'a, (u32, Bytes), W, N> {
    pub fn demux_bytes_tagged<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        let sink_expr = self.node.gen_sink_statement(&send_port).into();

        let recv_port = other.next_port();
        let source_expr = N::gen_source_statement(other, &recv_port).into();

        self.node.connect(other, &send_port, &recv_port);

        Stream::new(
            other.clone(),
            self.ir_leaves,
            HfPlusNode::Network {
                to_location: other.id(),
                serialize_pipeline: None,
                sink_expr,
                source_expr,
                deserialize_pipeline: None,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn demux_bytes_interleaved<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        self.demux_bytes_tagged(other)
            .map(q!(|r| r.map(|(_, b)| b)))
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: Location<'a>> Stream<'a, (u32, T), W, N> {
    pub fn demux_bincode_tagged<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        let serialize_pipeline = Some(serialize_bincode::<T>(true));
        let sink_expr = self.node.gen_sink_statement(&send_port).into();

        let recv_port = other.next_port();
        let deserialize_pipeline = Some(deserialize_bincode::<T>(true));
        let source_expr = N::gen_source_statement(other, &recv_port).into();

        self.node.connect(other, &send_port, &recv_port);

        Stream::new(
            other.clone(),
            self.ir_leaves,
            HfPlusNode::Network {
                to_location: other.id(),
                serialize_pipeline,
                sink_expr,
                source_expr,
                deserialize_pipeline,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn demux_bincode_interleaved<N2: Location<'a>>(self, other: &N2) -> Stream<'a, T, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        self.demux_bincode_tagged(other).map(q!(|(_, b)| b))
    }
}
