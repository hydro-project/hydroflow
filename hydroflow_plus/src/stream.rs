use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use hydroflow::bytes::Bytes;
use hydroflow::futures::Sink;
use hydroflow_lang::parse::Pipeline;
use serde::de::DeserializeOwned;
use serde::Serialize;
use stageleft::{q, IntoQuotedMut, Quoted};
use syn::parse_quote;

use crate::builder::FLOW_USED_MESSAGE;
use crate::cycle::{CycleCollection, CycleComplete, DeferTick, ForwardRefMarker, TickCycleMarker};
use crate::ir::{DebugInstantiate, HfPlusLeaf, HfPlusNode, TeeNode};
use crate::location::cluster::ClusterSelfId;
use crate::location::external_process::{ExternalBincodeStream, ExternalBytesPort};
use crate::location::{
    check_matching_location, CanSend, ExternalProcess, Location, LocationId, NoTick, Tick,
};
use crate::staging_util::get_this_crate;
use crate::{Cluster, ClusterId, Optional, Process, Singleton};

/// Marks the stream as being unbounded, which means that it is not
/// guaranteed to be complete in finite time.
pub enum Unbounded {}

/// Marks the stream as being bounded, which means that it is guaranteed
/// to be complete in finite time.
pub enum Bounded {}

/// An ordered sequence stream of elements of type `T`.
///
/// Type Parameters:
/// - `T`: the type of elements in the stream
/// - `L`: the location where the stream is being materialized
/// - `B`: the boundedness of the stream, which is either [`Bounded`]
///    or [`Unbounded`]
pub struct Stream<T, L, B> {
    location: L,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    _phantom: PhantomData<(T, L, B)>,
}

impl<'a, T, L: Location<'a>, B> Stream<T, L, B> {
    fn location_kind(&self) -> LocationId {
        self.location.id()
    }
}

impl<'a, T, L: Location<'a>> DeferTick for Stream<T, Tick<L>, Bounded> {
    fn defer_tick(self) -> Self {
        Stream::defer_tick(self)
    }
}

impl<'a, T, L: Location<'a>> CycleCollection<'a, TickCycleMarker> for Stream<T, Tick<L>, Bounded> {
    type Location = Tick<L>;

    fn create_source(ident: syn::Ident, location: Tick<L>) -> Self {
        let location_id = location.id();
        Stream::new(
            location,
            HfPlusNode::CycleSource {
                ident,
                location_kind: location_id,
            },
        )
    }
}

impl<'a, T, L: Location<'a>> CycleComplete<'a, TickCycleMarker> for Stream<T, Tick<L>, Bounded> {
    fn complete(self, ident: syn::Ident) {
        self.location
            .flow_state()
            .borrow_mut()
            .leaves
            .as_mut()
            .expect(FLOW_USED_MESSAGE)
            .push(HfPlusLeaf::CycleSink {
                ident,
                location_kind: self.location_kind(),
                input: Box::new(self.ir_node.into_inner()),
            });
    }
}

impl<'a, T, L: Location<'a> + NoTick, B> CycleCollection<'a, ForwardRefMarker> for Stream<T, L, B> {
    type Location = L;

    fn create_source(ident: syn::Ident, location: L) -> Self {
        let location_id = location.id();
        Stream::new(
            location,
            HfPlusNode::Persist(Box::new(HfPlusNode::CycleSource {
                ident,
                location_kind: location_id,
            })),
        )
    }
}

impl<'a, T, L: Location<'a> + NoTick, B> CycleComplete<'a, ForwardRefMarker> for Stream<T, L, B> {
    fn complete(self, ident: syn::Ident) {
        self.location
            .flow_state()
            .borrow_mut()
            .leaves
            .as_mut()
            .expect(FLOW_USED_MESSAGE)
            .push(HfPlusLeaf::CycleSink {
                ident,
                location_kind: self.location_kind(),
                input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
            });
    }
}

impl<'a, T, L: Location<'a>, B> Stream<T, L, B> {
    pub(crate) fn new(location: L, ir_node: HfPlusNode) -> Self {
        Stream {
            location,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone, L: Location<'a>, B> Clone for Stream<T, L, B> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: TeeNode(Rc::new(RefCell::new(orig_ir_node))),
            };
        }

        if let HfPlusNode::Tee { inner } = self.ir_node.borrow().deref() {
            Stream {
                location: self.location.clone(),
                ir_node: HfPlusNode::Tee {
                    inner: TeeNode(inner.0.clone()),
                }
                .into(),
                _phantom: PhantomData,
            }
        } else {
            unreachable!()
        }
    }
}

impl<'a, T, L: Location<'a>, B> Stream<T, L, B> {
    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Stream<U, L, B> {
        Stream::new(
            self.location,
            HfPlusNode::Map {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn cloned(self) -> Stream<T, L, B>
    where
        T: Clone,
    {
        self.map(q!(|d| d.clone()))
    }

    pub fn flat_map<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<U, L, B> {
        Stream::new(
            self.location,
            HfPlusNode::FlatMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn flatten<U>(self) -> Stream<U, L, B>
    where
        T: IntoIterator<Item = U>,
    {
        self.flat_map(q!(|d| d))
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Stream<T, L, B> {
        Stream::new(
            self.location,
            HfPlusNode::Filter {
                f: f.splice_fn1_borrow().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<U, L, B> {
        Stream::new(
            self.location,
            HfPlusNode::FilterMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn cross_singleton<O>(
        self,
        other: impl Into<Optional<O, L, Bounded>>,
    ) -> Stream<(T, O), L, B>
    where
        O: Clone,
    {
        let other: Optional<O, L, Bounded> = other.into();
        check_matching_location(&self.location, &other.location);

        Stream::new(
            self.location,
            HfPlusNode::CrossSingleton(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    /// Allow this stream through if the other stream has elements, otherwise the output is empty.
    pub fn continue_if<U>(self, signal: Optional<U, L, Bounded>) -> Stream<T, L, B> {
        self.cross_singleton(signal.map(q!(|_u| ())))
            .map(q!(|(d, _signal)| d))
    }

    /// Allow this stream through if the other stream is empty, otherwise the output is empty.
    pub fn continue_unless<U>(self, other: Optional<U, L, Bounded>) -> Stream<T, L, B> {
        self.continue_if(other.into_stream().count().filter(q!(|c| *c == 0)))
    }

    pub fn cross_product<O>(self, other: Stream<O, L, B>) -> Stream<(T, O), L, B>
    where
        T: Clone,
        O: Clone,
    {
        check_matching_location(&self.location, &other.location);

        Stream::new(
            self.location,
            HfPlusNode::CrossProduct(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn union(self, other: Stream<T, L, B>) -> Stream<T, L, B> {
        check_matching_location(&self.location, &other.location);

        Stream::new(
            self.location,
            HfPlusNode::Union(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn enumerate(self) -> Stream<(usize, T), L, B> {
        Stream::new(
            self.location,
            HfPlusNode::Enumerate(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn unique(self) -> Stream<T, L, B>
    where
        T: Eq + Hash,
    {
        Stream::new(
            self.location,
            HfPlusNode::Unique(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn filter_not_in(self, other: Stream<T, L, Bounded>) -> Stream<T, L, Bounded>
    where
        T: Eq + Hash,
    {
        check_matching_location(&self.location, &other.location);

        Stream::new(
            self.location,
            HfPlusNode::Difference(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn first(self) -> Optional<T, L, Bounded> {
        Optional::new(self.location, self.ir_node.into_inner())
    }

    pub fn inspect<F: Fn(&T) + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Stream<T, L, B> {
        if L::is_top_level() {
            Stream::new(
                self.location,
                HfPlusNode::Persist(Box::new(HfPlusNode::Inspect {
                    f: f.splice_fn1_borrow().into(),
                    input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                })),
            )
        } else {
            Stream::new(
                self.location,
                HfPlusNode::Inspect {
                    f: f.splice_fn1_borrow().into(),
                    input: Box::new(self.ir_node.into_inner()),
                },
            )
        }
    }

    pub fn fold<A, I: Fn() -> A + 'a, F: Fn(&mut A, T)>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Singleton<A, L, B> {
        let mut core = HfPlusNode::Fold {
            init: init.splice_fn0().into(),
            acc: comb.splice_fn2_borrow_mut().into(),
            input: Box::new(self.ir_node.into_inner()),
        };

        if L::is_top_level() {
            // top-level (possibly unbounded) singletons are represented as
            // a stream which produces all values from all ticks every tick,
            // so Unpersist will always give the lastest aggregation
            core = HfPlusNode::Persist(Box::new(core));
        }

        Singleton::new(self.location, core)
    }

    pub fn reduce<F: Fn(&mut T, T) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Optional<T, L, B> {
        let mut core = HfPlusNode::Reduce {
            f: comb.splice_fn2_borrow_mut().into(),
            input: Box::new(self.ir_node.into_inner()),
        };

        if L::is_top_level() {
            core = HfPlusNode::Persist(Box::new(core));
        }

        Optional::new(self.location, core)
    }

    pub fn max(self) -> Optional<T, L, B>
    where
        T: Ord,
    {
        self.reduce(q!(|curr, new| {
            if new > *curr {
                *curr = new;
            }
        }))
    }

    pub fn min(self) -> Optional<T, L, B>
    where
        T: Ord,
    {
        self.reduce(q!(|curr, new| {
            if new < *curr {
                *curr = new;
            }
        }))
    }

    pub fn count(self) -> Singleton<usize, L, B> {
        self.fold(q!(|| 0usize), q!(|count, _| *count += 1))
    }
}

impl<'a, T, L: Location<'a>> Stream<T, L, Bounded> {
    pub fn sort(self) -> Stream<T, L, Bounded>
    where
        T: Ord,
    {
        Stream::new(
            self.location,
            HfPlusNode::Sort(Box::new(self.ir_node.into_inner())),
        )
    }
}

impl<'a, K, V1, L: Location<'a>, B> Stream<(K, V1), L, B> {
    pub fn join<V2>(self, n: Stream<(K, V2), L, B>) -> Stream<(K, (V1, V2)), L, B>
    where
        K: Eq + Hash,
    {
        check_matching_location(&self.location, &n.location);

        Stream::new(
            self.location,
            HfPlusNode::Join(
                Box::new(self.ir_node.into_inner()),
                Box::new(n.ir_node.into_inner()),
            ),
        )
    }

    pub fn anti_join(self, n: Stream<K, L, Bounded>) -> Stream<(K, V1), L, B>
    where
        K: Eq + Hash,
    {
        check_matching_location(&self.location, &n.location);

        Stream::new(
            self.location,
            HfPlusNode::AntiJoin(
                Box::new(self.ir_node.into_inner()),
                Box::new(n.ir_node.into_inner()),
            ),
        )
    }
}

impl<'a, K: Eq + Hash, V, L: Location<'a>> Stream<(K, V), Tick<L>, Bounded> {
    pub fn fold_keyed<A, I: Fn() -> A + 'a, F: Fn(&mut A, V) + 'a>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Stream<(K, A), Tick<L>, Bounded> {
        Stream::new(
            self.location,
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
    ) -> Stream<(K, V), Tick<L>, Bounded> {
        Stream::new(
            self.location,
            HfPlusNode::ReduceKeyed {
                f: comb.splice_fn2_borrow_mut().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, T, L: Location<'a> + NoTick, B> Stream<T, L, B> {
    pub fn tick_batch(self, tick: &Tick<L>) -> Stream<T, Tick<L>, Bounded> {
        Stream::new(
            tick.clone(),
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn tick_prefix(self, tick: &Tick<L>) -> Stream<T, Tick<L>, Bounded>
    where
        T: Clone,
    {
        self.tick_batch(tick).persist()
    }

    pub fn sample_every(
        self,
        interval: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<T, L, Unbounded> {
        let samples = self.location.source_interval(interval);
        let tick = self.location.tick();
        self.tick_batch(&tick)
            .continue_if(samples.tick_batch(&tick).first())
            .all_ticks()
    }

    pub fn for_each<F: Fn(T) + 'a>(self, f: impl IntoQuotedMut<'a, F>) {
        self.location
            .flow_state()
            .borrow_mut()
            .leaves
            .as_mut()
            .expect(FLOW_USED_MESSAGE)
            .push(HfPlusLeaf::ForEach {
                input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                f: f.splice_fn1().into(),
            });
    }

    pub fn dest_sink<S: Unpin + Sink<T> + 'a>(self, sink: impl Quoted<'a, S>) {
        self.location
            .flow_state()
            .borrow_mut()
            .leaves
            .as_mut()
            .expect(FLOW_USED_MESSAGE)
            .push(HfPlusLeaf::DestSink {
                sink: sink.splice_typed().into(),
                input: Box::new(self.ir_node.into_inner()),
            });
    }
}

impl<'a, T, L: Location<'a>> Stream<T, Tick<L>, Bounded> {
    pub fn all_ticks(self) -> Stream<T, L, Unbounded> {
        Stream::new(
            self.location.outer().clone(),
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<T, Tick<L>, Bounded>
    where
        T: Clone,
    {
        Stream::new(
            self.location,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Stream<T, Tick<L>, Bounded> {
        Stream::new(
            self.location,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn delta(self) -> Stream<T, Tick<L>, Bounded> {
        Stream::new(
            self.location,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }
}

fn serialize_bincode<T: Serialize>(is_demux: bool) -> Pipeline {
    let root = get_this_crate();

    let t_type: syn::Type = stageleft::quote_type::<T>();

    if is_demux {
        parse_quote! {
            map(|(id, data): (#root::ClusterId<_>, #t_type)| {
                (id.raw_id, #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into())
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

pub(super) fn deserialize_bincode<T: DeserializeOwned>(tagged: Option<syn::Type>) -> Pipeline {
    let root = get_this_crate();

    let t_type: syn::Type = stageleft::quote_type::<T>();

    if let Some(c_type) = tagged {
        parse_quote! {
            map(|res| {
                let (id, b) = res.unwrap();
                (#root::ClusterId::<#c_type>::from_raw(id), #root::runtime_support::bincode::deserialize::<#t_type>(&b).unwrap())
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

impl<'a, T, L: Location<'a> + NoTick, B> Stream<T, L, B> {
    pub fn decouple_process<P2>(
        self,
        other: &Process<'a, P2>,
    ) -> Stream<T, Process<'a, P2>, Unbounded>
    where
        L: CanSend<'a, Process<'a, P2>, In<T> = T, Out<T> = T>,
        T: Clone + Serialize + DeserializeOwned,
    {
        self.send_bincode::<Process<'a, P2>, T>(other)
    }

    pub fn decouple_cluster<C2, Tag>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<T, Cluster<'a, C2>, Unbounded>
    where
        L: CanSend<'a, Cluster<'a, C2>, In<T> = (ClusterId<C2>, T), Out<T> = (Tag, T)>,
        T: Clone + Serialize + DeserializeOwned,
    {
        let self_node_id = match self.location_kind() {
            LocationId::Cluster(cluster_id) => ClusterSelfId {
                id: cluster_id,
                _phantom: PhantomData,
            },
            _ => panic!("decouple_cluster must be called on a cluster"),
        };

        self.map(q!(move |b| (self_node_id, b.clone())))
            .send_bincode_interleaved(other)
    }

    pub fn send_bincode<L2: Location<'a>, CoreType>(
        self,
        other: &L2,
    ) -> Stream<L::Out<CoreType>, L2, Unbounded>
    where
        L: CanSend<'a, L2, In<CoreType> = T>,
        CoreType: Serialize + DeserializeOwned,
    {
        let serialize_pipeline = Some(serialize_bincode::<CoreType>(L::is_demux()));

        let deserialize_pipeline = Some(deserialize_bincode::<CoreType>(L::tagged_type()));

        Stream::new(
            other.clone(),
            HfPlusNode::Network {
                from_location: self.location_kind(),
                from_key: None,
                to_location: other.id(),
                to_key: None,
                serialize_pipeline,
                instantiate_fn: DebugInstantiate::Building(),
                deserialize_pipeline,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn send_bincode_external<L2: 'a, CoreType>(
        self,
        other: &ExternalProcess<L2>,
    ) -> ExternalBincodeStream<L::Out<CoreType>>
    where
        L: CanSend<'a, ExternalProcess<'a, L2>, In<CoreType> = T, Out<CoreType> = CoreType>,
        CoreType: Serialize + DeserializeOwned,
        // for now, we restirct Out<CoreType> to be CoreType, which means no tagged cluster -> external
    {
        let serialize_pipeline = Some(serialize_bincode::<CoreType>(L::is_demux()));

        let mut flow_state_borrow = self.location.flow_state().borrow_mut();

        let external_key = flow_state_borrow.next_external_out;
        flow_state_borrow.next_external_out += 1;

        let leaves = flow_state_borrow.leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled()");

        let dummy_f: syn::Expr = syn::parse_quote!(());

        leaves.push(HfPlusLeaf::ForEach {
            f: dummy_f.into(),
            input: Box::new(HfPlusNode::Network {
                from_location: self.location_kind(),
                from_key: None,
                to_location: other.id(),
                to_key: Some(external_key),
                serialize_pipeline,
                instantiate_fn: DebugInstantiate::Building(),
                deserialize_pipeline: None,
                input: Box::new(self.ir_node.into_inner()),
            }),
        });

        ExternalBincodeStream {
            process_id: other.id,
            port_id: external_key,
            _phantom: PhantomData,
        }
    }

    pub fn send_bytes<L2: Location<'a>>(self, other: &L2) -> Stream<L::Out<Bytes>, L2, Unbounded>
    where
        L: CanSend<'a, L2, In<Bytes> = T>,
    {
        let root = get_this_crate();
        Stream::new(
            other.clone(),
            HfPlusNode::Network {
                from_location: self.location_kind(),
                from_key: None,
                to_location: other.id(),
                to_key: None,
                serialize_pipeline: None,
                instantiate_fn: DebugInstantiate::Building(),
                deserialize_pipeline: if let Some(c_type) = L::tagged_type() {
                    Some(
                        parse_quote!(map(|(id, b)| (#root::ClusterId<#c_type>::from_raw(id), b.unwrap().freeze()))),
                    )
                } else {
                    Some(parse_quote!(map(|b| b.unwrap().freeze())))
                },
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn send_bytes_external<L2: 'a>(self, other: &ExternalProcess<L2>) -> ExternalBytesPort
    where
        L: CanSend<'a, ExternalProcess<'a, L2>, In<Bytes> = T, Out<Bytes> = Bytes>,
    {
        let mut flow_state_borrow = self.location.flow_state().borrow_mut();
        let external_key = flow_state_borrow.next_external_out;
        flow_state_borrow.next_external_out += 1;

        let leaves = flow_state_borrow.leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled()");

        let dummy_f: syn::Expr = syn::parse_quote!(());

        leaves.push(HfPlusLeaf::ForEach {
            f: dummy_f.into(),
            input: Box::new(HfPlusNode::Network {
                from_location: self.location_kind(),
                from_key: None,
                to_location: other.id(),
                to_key: Some(external_key),
                serialize_pipeline: None,
                instantiate_fn: DebugInstantiate::Building(),
                deserialize_pipeline: None,
                input: Box::new(self.ir_node.into_inner()),
            }),
        });

        ExternalBytesPort {
            process_id: other.id,
            port_id: external_key,
        }
    }

    pub fn send_bincode_interleaved<L2: Location<'a>, Tag, CoreType>(
        self,
        other: &L2,
    ) -> Stream<CoreType, L2, Unbounded>
    where
        L: CanSend<'a, L2, In<CoreType> = T, Out<CoreType> = (Tag, CoreType)>,
        CoreType: Serialize + DeserializeOwned,
    {
        self.send_bincode::<L2, CoreType>(other).map(q!(|(_, b)| b))
    }

    pub fn send_bytes_interleaved<L2: Location<'a>, Tag>(
        self,
        other: &L2,
    ) -> Stream<Bytes, L2, Unbounded>
    where
        L: CanSend<'a, L2, In<Bytes> = T, Out<Bytes> = (Tag, Bytes)>,
    {
        self.send_bytes::<L2>(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bincode<C2>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<L::Out<T>, Cluster<'a, C2>, Unbounded>
    where
        L: CanSend<'a, Cluster<'a, C2>, In<T> = (ClusterId<C2>, T)>,
        T: Clone + Serialize + DeserializeOwned,
    {
        let ids = other.members();

        self.flat_map(q!(|b| ids.iter().map(move |id| (
            ::std::clone::Clone::clone(id),
            ::std::clone::Clone::clone(&b)
        ))))
        .send_bincode(other)
    }

    pub fn broadcast_bincode_interleaved<C2, Tag>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<T, Cluster<'a, C2>, Unbounded>
    where
        L: CanSend<'a, Cluster<'a, C2>, In<T> = (ClusterId<C2>, T), Out<T> = (Tag, T)> + 'a,
        T: Clone + Serialize + DeserializeOwned,
    {
        self.broadcast_bincode(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bytes<C2>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<L::Out<Bytes>, Cluster<'a, C2>, Unbounded>
    where
        L: CanSend<'a, Cluster<'a, C2>, In<Bytes> = (ClusterId<C2>, T)> + 'a,
        T: Clone,
    {
        let ids = other.members();

        self.flat_map(q!(|b| ids.iter().map(move |id| (
            ::std::clone::Clone::clone(id),
            ::std::clone::Clone::clone(&b)
        ))))
        .send_bytes(other)
    }

    pub fn broadcast_bytes_interleaved<C2, Tag>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<Bytes, Cluster<'a, C2>, Unbounded>
    where
        L: CanSend<'a, Cluster<'a, C2>, In<Bytes> = (ClusterId<C2>, T), Out<Bytes> = (Tag, Bytes)>
            + 'a,
        T: Clone,
    {
        self.broadcast_bytes(other).map(q!(|(_, b)| b))
    }
}

#[cfg(test)]
mod tests {
    use hydro_deploy::Deployment;
    use hydroflow::futures::StreamExt;
    use serde::{Deserialize, Serialize};
    use stageleft::q;

    use crate::location::Location;
    use crate::FlowBuilder;

    struct P1 {}
    struct P2 {}

    #[derive(Serialize, Deserialize, Debug)]
    struct SendOverNetwork {
        n: u32,
    }

    #[tokio::test]
    async fn first_ten_distributed() {
        let mut deployment = Deployment::new();

        let flow = FlowBuilder::new();
        let first_node = flow.process::<P1>();
        let second_node = flow.process::<P2>();
        let external = flow.external_process::<P2>();

        let numbers = first_node.source_iter(q!(0..10));
        let out_port = numbers
            .map(q!(|n| SendOverNetwork { n }))
            .send_bincode(&second_node)
            .send_bincode_external(&external);

        let nodes = flow
            .with_process(&first_node, deployment.Localhost())
            .with_process(&second_node, deployment.Localhost())
            .with_external(&external, deployment.Localhost())
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        let mut external_out = nodes.connect_source_bincode(out_port).await;

        deployment.start().await.unwrap();

        for i in 0..10 {
            assert_eq!(external_out.next().await.unwrap().n, i);
        }
    }
}
