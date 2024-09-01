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

use crate::builder::{ClusterIds, FlowLeaves};
use crate::cycle::{CycleCollection, CycleComplete};
use crate::ir::{DebugInstantiate, HfPlusLeaf, HfPlusNode, HfPlusSource};
use crate::location::{CanSend, Location, LocationId};
use crate::{Cluster, Optional, Singleton};

/// Marks the stream as being unbounded, which means that it is not
/// guaranteed to be complete in finite time.
pub enum Unbounded {}

/// Marks the stream as being bounded, which means that it is guaranteed
/// to be complete in finite time.
pub enum Bounded {}

/// Marks the stream as existing outside of a clock domain.
pub enum NoTick {}
/// Marks the stream as being inside the single global clock domain.
pub enum Tick {}

/// An infinite stream of elements of type `T`.
///
/// Type Parameters:
/// - `'a`: the lifetime of the final Hydroflow graph, which constraints
///   which values can be captured in closures passed to operators
/// - `T`: the type of elements in the stream
/// - `W`: the boundedness of the stream, which is either [`Bounded`]
///    or [`Unbounded`]
/// - `C`: the tick domain of the stream, which is either [`Tick`] or
///   [`NoTick`]
/// - `N`: the type of the node that the stream is materialized on
pub struct Stream<'a, T, W, C, N: Location> {
    location_kind: LocationId,

    ir_leaves: FlowLeaves<'a>,
    pub(crate) ir_node: RefCell<HfPlusNode<'a>>,

    _phantom: PhantomData<(&'a mut &'a (), T, N, W, C)>,
}

impl<'a, T, W, N: Location> CycleComplete<'a> for Stream<'a, T, W, Tick, N> {
    fn complete(self, ident: syn::Ident) {
        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(self.ir_node.into_inner()),
        });
    }
}

impl<'a, T, W, N: Location> CycleCollection<'a> for Stream<'a, T, W, Tick, N> {
    type Location = N;

    fn create_source(ident: syn::Ident, ir_leaves: FlowLeaves<'a>, l: LocationId) -> Self {
        Stream::new(
            l,
            ir_leaves,
            HfPlusNode::CycleSource {
                ident,
                location_kind: l,
            },
        )
    }
}

impl<'a, T, W, N: Location> CycleComplete<'a> for Stream<'a, T, W, NoTick, N> {
    fn complete(self, ident: syn::Ident) {
        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
        });
    }
}

impl<'a, T, W, N: Location> CycleCollection<'a> for Stream<'a, T, W, NoTick, N> {
    type Location = N;

    fn create_source(ident: syn::Ident, ir_leaves: FlowLeaves<'a>, l: LocationId) -> Self {
        Stream::new(
            l,
            ir_leaves,
            HfPlusNode::Persist(Box::new(HfPlusNode::CycleSource {
                ident,
                location_kind: l,
            })),
        )
    }
}

impl<'a, T, W, C, N: Location> Stream<'a, T, W, C, N> {
    pub(crate) fn new(
        location_kind: LocationId,
        ir_leaves: FlowLeaves<'a>,
        ir_node: HfPlusNode<'a>,
    ) -> Self {
        Stream {
            location_kind,
            ir_leaves,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone, W, C, N: Location> Clone for Stream<'a, T, W, C, N> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: Rc::new(RefCell::new(orig_ir_node)),
            };
        }

        if let HfPlusNode::Tee { inner } = self.ir_node.borrow().deref() {
            Stream {
                location_kind: self.location_kind,
                ir_leaves: self.ir_leaves.clone(),
                ir_node: HfPlusNode::Tee {
                    inner: inner.clone(),
                }
                .into(),
                _phantom: PhantomData,
            }
        } else {
            unreachable!()
        }
    }
}

impl<'a, T, W, C, N: Location> Stream<'a, T, W, C, N> {
    pub fn map<U, F: Fn(T) -> U + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, C, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Map {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn flat_map<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, C, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::FlatMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, T, W, C, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Filter {
                f: f.splice_fn1_borrow().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, C, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::FilterMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn cross_singleton<O>(
        self,
        other: impl Into<Optional<'a, O, Bounded, C, N>>,
    ) -> Stream<'a, (T, O), W, C, N>
    where
        O: Clone,
    {
        let other: Optional<'a, O, Bounded, C, N> = other.into();
        if self.location_kind != other.location_kind {
            panic!("cross_singleton must be called on streams on the same node");
        }

        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::CrossSingleton(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    // TODO(shadaj): should allow for differing windows, using strongest one
    pub fn cross_product<O>(self, other: Stream<'a, O, W, C, N>) -> Stream<'a, (T, O), W, C, N>
    where
        T: Clone,
        O: Clone,
    {
        if self.location_kind != other.location_kind {
            panic!("cross_product must be called on streams on the same node");
        }

        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::CrossProduct(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn union(self, other: Stream<'a, T, W, C, N>) -> Stream<'a, T, W, C, N> {
        if self.location_kind != other.location_kind {
            panic!("union must be called on streams on the same node");
        }

        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Union(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn dest_sink<S: Unpin + Sink<T> + 'a>(self, sink: impl Quoted<'a, S>) {
        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::DestSink {
            sink: sink.splice_typed().into(),
            input: Box::new(self.ir_node.into_inner()),
        });
    }
}

impl<'a, T, N: Location> Stream<'a, T, Bounded, Tick, N> {
    pub fn all_ticks(self) -> Stream<'a, T, Unbounded, NoTick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<'a, T, Bounded, Tick, N>
    where
        T: Clone,
    {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Stream<'a, T, Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn inspect<F: Fn(&T) + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, T, Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Inspect {
                f: f.splice_fn1_borrow().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn first(self) -> Optional<'a, T, Bounded, Tick, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            self.ir_node.into_inner(),
        )
    }

    /// Allow this stream through if the other stream has elements, otherwise the output is empty.
    pub fn continue_if<U>(
        self,
        signal: Optional<'a, U, Bounded, Tick, N>,
    ) -> Stream<'a, T, Bounded, Tick, N> {
        self.cross_singleton(signal.map(q!(|_u| ())))
            .map(q!(|(d, _signal)| d))
    }

    /// Allow this stream through if the other stream is empty, otherwise the output is empty.
    pub fn continue_unless<U>(
        self,
        other: Optional<'a, U, Bounded, Tick, N>,
    ) -> Stream<'a, T, Bounded, Tick, N> {
        self.continue_if(other.into_stream().count().filter(q!(|c| *c == 0)))
    }

    pub fn enumerate(self) -> Stream<'a, (usize, T), Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Enumerate(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn fold<A, I: Fn() -> A + 'a, F: Fn(&mut A, T)>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Singleton<'a, A, Bounded, Tick, N> {
        Singleton::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Fold {
                init: init.splice_fn0().into(),
                acc: comb.splice_fn2_borrow_mut().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn reduce<F: Fn(&mut T, T) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Optional<'a, T, Bounded, Tick, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Reduce {
                f: comb.splice_fn2_borrow_mut().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn sort(self) -> Stream<'a, T, Bounded, Tick, N>
    where
        T: Ord,
    {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Sort(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn count(self) -> Singleton<'a, usize, Bounded, Tick, N> {
        self.fold(q!(|| 0usize), q!(|count, _| *count += 1))
    }

    pub fn delta(self) -> Stream<'a, T, Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn unique(self) -> Stream<'a, T, Bounded, Tick, N>
    where
        T: Eq + Hash,
    {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Unique(Box::new(self.ir_node.into_inner())),
        )
    }
}

impl<'a, T, W, N: Location> Stream<'a, T, W, NoTick, N> {
    pub fn tick_batch(self) -> Stream<'a, T, Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn tick_prefix(self) -> Stream<'a, T, Bounded, Tick, N>
    where
        T: Clone,
    {
        self.tick_batch().persist()
    }

    pub fn inspect<F: Fn(&T) + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, T, W, NoTick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(HfPlusNode::Inspect {
                f: f.splice_fn1_borrow().into(),
                input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
            })),
        )
    }
}

impl<'a, T, W, N: Location> Stream<'a, T, W, NoTick, N> {
    pub fn for_each<F: Fn(T) + 'a>(self, f: impl IntoQuotedMut<'a, F>) {
        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::ForEach {
            input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
            f: f.splice_fn1().into(),
        });
    }
}

impl<'a, T, N: Location> Stream<'a, T, Unbounded, NoTick, N> {
    pub fn sample_every(
        self,
        duration: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<'a, T, Unbounded, NoTick, N> {
        let interval = duration.splice_typed();

        let samples = Stream::<'a, hydroflow::tokio::time::Instant, Bounded, Tick, N>::new(
            self.location_kind,
            self.ir_leaves.clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Interval(interval.into()),
                location_kind: self.location_kind,
            },
        );

        self.tick_batch().continue_if(samples.first()).all_ticks()
    }

    pub fn fold<A, I: Fn() -> A + 'a, F: Fn(&mut A, T)>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Singleton<'a, A, Unbounded, NoTick, N> {
        // unbounded singletons are represented as a stream
        // which produces all values from all ticks every tick,
        // so delta will always give the lastest aggregation
        Singleton::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(HfPlusNode::Fold {
                init: init.splice_fn0().into(),
                acc: comb.splice_fn2_borrow_mut().into(),
                input: Box::new(self.ir_node.into_inner()),
            })),
        )
    }

    pub fn reduce<F: Fn(&mut T, T) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Optional<'a, T, Unbounded, NoTick, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(HfPlusNode::Reduce {
                f: comb.splice_fn2_borrow_mut().into(),
                input: Box::new(self.ir_node.into_inner()),
            })),
        )
    }
}

impl<'a, T, C, N: Location> Stream<'a, T, Bounded, C, N> {
    pub fn filter_not_in(self, other: Stream<'a, T, Bounded, C, N>) -> Stream<'a, T, Bounded, C, N>
    where
        T: Eq + Hash,
    {
        if self.location_kind != other.location_kind {
            panic!("union must be called on streams on the same node");
        }

        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Difference(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }
}

impl<'a, T: Clone, W, C, N: Location> Stream<'a, &T, W, C, N> {
    pub fn cloned(self) -> Stream<'a, T, W, C, N> {
        self.map(q!(|d| d.clone()))
    }
}

impl<'a, K, V1, W, C, N: Location> Stream<'a, (K, V1), W, C, N> {
    // TODO(shadaj): figure out window semantics
    pub fn join<W2, V2>(
        self,
        n: Stream<'a, (K, V2), W2, C, N>,
    ) -> Stream<'a, (K, (V1, V2)), W, C, N>
    where
        K: Eq + Hash,
    {
        if self.location_kind != n.location_kind {
            panic!("join must be called on streams on the same node");
        }

        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Join(
                Box::new(self.ir_node.into_inner()),
                Box::new(n.ir_node.into_inner()),
            ),
        )
    }

    pub fn anti_join<W2>(self, n: Stream<'a, K, W2, C, N>) -> Stream<'a, (K, V1), W, C, N>
    where
        K: Eq + Hash,
    {
        if self.location_kind != n.location_kind {
            panic!("anti_join must be called on streams on the same node");
        }

        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::AntiJoin(
                Box::new(self.ir_node.into_inner()),
                Box::new(n.ir_node.into_inner()),
            ),
        )
    }
}

impl<'a, K: Eq + Hash, V, N: Location> Stream<'a, (K, V), Bounded, Tick, N> {
    pub fn fold_keyed<A, I: Fn() -> A + 'a, F: Fn(&mut A, V) + 'a>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, (K, A), Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::FoldKeyed {
                init: init.splice_fn0().into(),
                acc: comb.splice_fn2_borrow_mut().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn reduce_keyed<F: Fn(&mut V, V) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, (K, V), Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::ReduceKeyed {
                f: comb.splice_fn2_borrow_mut().into(),
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

impl<'a, T, W, N: Location> Stream<'a, T, W, NoTick, N> {
    pub fn send_bincode<N2: Location, CoreType>(
        self,
        other: &N2,
    ) -> Stream<'a, N::Out<CoreType>, Unbounded, NoTick, N2>
    where
        N: CanSend<N2, In<CoreType> = T>,
        CoreType: Serialize + DeserializeOwned,
    {
        let serialize_pipeline = Some(serialize_bincode::<CoreType>(N::is_demux()));

        let deserialize_pipeline = Some(deserialize_bincode::<CoreType>(N::is_tagged()));

        Stream::new(
            other.id(),
            self.ir_leaves,
            HfPlusNode::Network {
                from_location: self.location_kind,
                to_location: other.id(),
                serialize_pipeline,
                instantiate_fn: DebugInstantiate::Building(),
                deserialize_pipeline,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn send_bytes<N2: Location>(
        self,
        other: &N2,
    ) -> Stream<'a, N::Out<Bytes>, Unbounded, NoTick, N2>
    where
        N: CanSend<N2, In<Bytes> = T>,
    {
        Stream::new(
            other.id(),
            self.ir_leaves,
            HfPlusNode::Network {
                from_location: self.location_kind,
                to_location: other.id(),
                serialize_pipeline: None,
                instantiate_fn: DebugInstantiate::Building(),
                deserialize_pipeline: if N::is_tagged() {
                    Some(parse_quote!(map(|(id, b)| (id, b.unwrap().freeze()))))
                } else {
                    Some(parse_quote!(map(|b| b.unwrap().freeze())))
                },
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn send_bincode_interleaved<N2: Location, Tag, CoreType>(
        self,
        other: &N2,
    ) -> Stream<'a, CoreType, Unbounded, NoTick, N2>
    where
        N: CanSend<N2, In<CoreType> = T, Out<CoreType> = (Tag, CoreType)>,
        CoreType: Serialize + DeserializeOwned,
    {
        self.send_bincode::<N2, CoreType>(other).map(q!(|(_, b)| b))
    }

    pub fn send_bytes_interleaved<N2: Location, Tag>(
        self,
        other: &N2,
    ) -> Stream<'a, Bytes, Unbounded, NoTick, N2>
    where
        N: CanSend<N2, In<Bytes> = T, Out<Bytes> = (Tag, Bytes)>,
    {
        self.send_bytes::<N2>(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bincode<C2>(
        self,
        other: &Cluster<C2>,
    ) -> Stream<'a, N::Out<T>, Unbounded, NoTick, Cluster<C2>>
    where
        N: CanSend<Cluster<C2>, In<T> = (u32, T)>,
        T: Clone + Serialize + DeserializeOwned,
    {
        let ids = ClusterIds::<'a> {
            id: other.id,
            _phantom: PhantomData,
        };

        self.flat_map(q!(|b| ids.iter().map(move |id| (
            ::std::clone::Clone::clone(id),
            ::std::clone::Clone::clone(&b)
        ))))
        .send_bincode(other)
    }

    pub fn broadcast_bincode_interleaved<C2, Tag>(
        self,
        other: &Cluster<C2>,
    ) -> Stream<'a, T, Unbounded, NoTick, Cluster<C2>>
    where
        N: CanSend<Cluster<C2>, In<T> = (u32, T), Out<T> = (Tag, T)> + 'a,
        T: Clone + Serialize + DeserializeOwned,
    {
        self.broadcast_bincode(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bytes<C2>(
        self,
        other: &Cluster<C2>,
    ) -> Stream<'a, N::Out<Bytes>, Unbounded, NoTick, Cluster<C2>>
    where
        N: CanSend<Cluster<C2>, In<Bytes> = (u32, T)> + 'a,
        T: Clone,
    {
        let ids = ClusterIds::<'a> {
            id: other.id,
            _phantom: PhantomData,
        };

        self.flat_map(q!(|b| ids.iter().map(move |id| (
            ::std::clone::Clone::clone(id),
            ::std::clone::Clone::clone(&b)
        ))))
        .send_bytes(other)
    }

    pub fn broadcast_bytes_interleaved<C2, Tag>(
        self,
        other: &Cluster<C2>,
    ) -> Stream<'a, Bytes, Unbounded, NoTick, Cluster<C2>>
    where
        N: CanSend<Cluster<C2>, In<Bytes> = (u32, T), Out<Bytes> = (Tag, Bytes)> + 'a,
        T: Clone,
    {
        self.broadcast_bytes(other).map(q!(|(_, b)| b))
    }
}
