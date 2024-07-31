use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use hydroflow::bytes::Bytes;
use hydroflow::futures::Sink;
use hydroflow_lang::parse::Pipeline;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::de::DeserializeOwned;
use serde::Serialize;
use stageleft::{q, IntoQuotedMut, Quoted};
use syn::parse_quote;

use crate::builder::FlowLeaves;
use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource};
use crate::location::{Cluster, HfSend, Location};

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
pub struct Stream<'a, T, W, N: Location + Clone> {
    node: N,

    ir_leaves: FlowLeaves,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    _phantom: PhantomData<(&'a mut &'a (), T, W)>,
}

impl<'a, T, W, N: Location + Clone> Stream<'a, T, W, N> {
    pub(crate) fn new(node: N, ir_leaves: FlowLeaves, ir_node: HfPlusNode) -> Self {
        Stream {
            node,
            ir_leaves,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone, W, N: Location + Clone> Clone for Stream<'a, T, W, N> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: Rc::new(RefCell::new(orig_ir_node)),
            };
        }

        Stream::new(
            self.node.clone(),
            self.ir_leaves.clone(),
            self.ir_node.borrow().clone(),
        )
    }
}

impl<'a, T, W, N: Location + Clone> Stream<'a, T, W, N> {
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
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::FlatMap {
                f: f.splice().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn enumerate(self) -> Stream<'a, (usize, T), W, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Enumerate(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn inspect<F: Fn(&T) + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Stream<'a, T, W, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Inspect {
                f: f.splice().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, T, W, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Filter {
                f: f.splice().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::FilterMap {
                f: f.splice().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
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
        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::ForEach {
            input: Box::new(self.ir_node.into_inner()),
            f: f.splice().into(),
        });
    }

    pub fn dest_sink<S: Unpin + Sink<T> + 'a>(self, sink: impl Quoted<'a, S>) {
        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::DestSink {
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

impl<'a, T, N: Location + Clone> Stream<'a, T, Async, N> {
    pub fn tick_batch(self) -> Stream<'a, T, Windowed, N> {
        Stream::new(self.node, self.ir_leaves, self.ir_node.into_inner())
    }
}

impl<'a, T, N: Location + Clone> Stream<'a, T, Windowed, N> {
    pub fn fold<A, I: Fn() -> A + 'a, C: Fn(&mut A, T)>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, A, Windowed, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Fold {
                init: init.splice().into(),
                acc: comb.splice().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn reduce<C: Fn(&mut T, T) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, T, Windowed, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Reduce {
                f: comb.splice().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
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
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::Unique(Box::new(self.ir_node.into_inner())),
        )
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
        let interval = duration.splice();

        let samples = Stream::<'a, hydroflow::tokio::time::Instant, Windowed, N>::new(
            self.node.clone(),
            self.ir_leaves.clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Interval(interval.into()),
                location_id: self.node.id(),
            },
        );

        self.cross_product(samples).map(q!(|(a, _)| a))
    }
}

impl<'a, T: Clone, W, N: Location + Clone> Stream<'a, &T, W, N> {
    pub fn cloned(self) -> Stream<'a, T, W, N> {
        self.map(q!(|d| d.clone()))
    }
}

impl<'a, K, V1, W, N: Location + Clone> Stream<'a, (K, V1), W, N> {
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

impl<'a, K: Eq + Hash, V, N: Location + Clone> Stream<'a, (K, V), Windowed, N> {
    pub fn fold_keyed<A, I: Fn() -> A + 'a, C: Fn(&mut A, V) + 'a>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, (K, A), Windowed, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::FoldKeyed {
                init: init.splice().into(),
                acc: comb.splice().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn reduce_keyed<F: Fn(&mut V, V) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, (K, V), Windowed, N> {
        Stream::new(
            self.node,
            self.ir_leaves,
            HfPlusNode::ReduceKeyed {
                f: comb.splice().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
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

fn serialize_bincode<T: Serialize>(is_demux: bool) -> Pipeline {
    let root = get_this_crate();

    let t_type: syn::Type = stageleft::quote_type::<T>();

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

fn deserialize_bincode<T: DeserializeOwned>(tagged: bool) -> Pipeline {
    let root = get_this_crate();

    let t_type: syn::Type = stageleft::quote_type::<T>();

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

impl<'a, T, W, N: Location + Clone> Stream<'a, T, W, N> {
    pub fn send_bincode<N2: Location + Clone, V, CoreType>(
        self,
        other: &N2,
    ) -> Stream<'a, N::Out<CoreType>, Async, N2>
    where
        N: HfSend<N2, V, In<CoreType> = T>,
        CoreType: Serialize + DeserializeOwned,
    {
        let send_port = self.node.next_port();
        let serialize_pipeline = Some(serialize_bincode::<CoreType>(N::is_demux()));
        let sink_expr = self.node.gen_sink_statement(&send_port).into();

        let recv_port = other.next_port();
        let deserialize_pipeline = Some(deserialize_bincode::<CoreType>(N::is_tagged()));
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

    pub fn send_bytes<N2: Location + Clone, V>(
        self,
        other: &N2,
    ) -> Stream<'a, N::Out<Bytes>, Async, N2>
    where
        N: HfSend<N2, V, In<Bytes> = T>,
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
                deserialize_pipeline: if N::is_tagged() {
                    Some(parse_quote!(map(|(id, b)| (id, b.unwrap().freeze()))))
                } else {
                    Some(parse_quote!(map(|b| b.unwrap().freeze())))
                },
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn send_bincode_interleaved<N2: Location + Clone, Tag, CoreType, V>(
        self,
        other: &N2,
    ) -> Stream<'a, CoreType, Async, N2>
    where
        N: HfSend<N2, V, In<CoreType> = T, Out<CoreType> = (Tag, CoreType)>,
        CoreType: Serialize + DeserializeOwned,
    {
        self.send_bincode::<N2, V, CoreType>(other)
            .map(q!(|(_, b)| b))
    }

    pub fn send_bytes_interleaved<N2: Location + Clone, Tag, V>(
        self,
        other: &N2,
    ) -> Stream<'a, Bytes, Async, N2>
    where
        N: HfSend<N2, V, In<Bytes> = T, Out<Bytes> = (Tag, Bytes)>,
    {
        self.send_bytes::<N2, V>(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bincode<N2: Location + Cluster<'a> + Clone, V>(
        self,
        other: &N2,
    ) -> Stream<'a, N::Out<T>, Async, N2>
    where
        N: HfSend<N2, V, In<T> = (N2::Id, T)>,
        T: Serialize + DeserializeOwned,
        N2::Id: Clone,
    {
        let ids_spliced = other.ids().splice();

        let other_ids = Stream::<'a, &N2::Id, Windowed, N>::new(
            self.node.clone(),
            self.ir_leaves.clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Iter(ids_spliced.into()),
                location_id: self.node.id(),
            },
        )
        .cloned()
        .all_ticks();

        other_ids
            .cross_product(self.assume_windowed())
            .send_bincode(other)
    }

    pub fn broadcast_bincode_interleaved<N2: Location + Cluster<'a> + Clone, Tag, V>(
        self,
        other: &N2,
    ) -> Stream<'a, T, Async, N2>
    where
        N: HfSend<N2, V, In<T> = (N2::Id, T), Out<T> = (Tag, T)>,
        T: Serialize + DeserializeOwned,
        N2::Id: Clone,
    {
        self.broadcast_bincode(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bytes<N2: Location + Cluster<'a> + Clone, V>(
        self,
        other: &N2,
    ) -> Stream<'a, N::Out<Bytes>, Async, N2>
    where
        N: HfSend<N2, V, In<Bytes> = (N2::Id, T)>,
        N2::Id: Clone,
    {
        let ids_spliced = other.ids().splice();

        let other_ids = Stream::<'a, &N2::Id, Windowed, N>::new(
            self.node.clone(),
            self.ir_leaves.clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Iter(ids_spliced.into()),
                location_id: self.node.id(),
            },
        )
        .cloned()
        .all_ticks();

        other_ids
            .cross_product(self.assume_windowed())
            .send_bytes(other)
    }

    pub fn broadcast_bytes_interleaved<N2: Location + Cluster<'a> + Clone, Tag, V>(
        self,
        other: &N2,
    ) -> Stream<'a, Bytes, Async, N2>
    where
        N: HfSend<N2, V, In<Bytes> = (N2::Id, T), Out<Bytes> = (Tag, Bytes)>,
        N2::Id: Clone,
    {
        self.broadcast_bytes(other).map(q!(|(_, b)| b))
    }
}
