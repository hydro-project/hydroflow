use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use hydroflow::bytes::Bytes;
use hydroflow::futures;
use hydroflow_lang::parse::Pipeline;
use serde::de::DeserializeOwned;
use serde::Serialize;
use stageleft::{q, IntoQuotedMut, QuotedWithContext};
use syn::parse_quote;

use crate::builder::FLOW_USED_MESSAGE;
use crate::cycle::{CycleCollection, CycleComplete, DeferTick, ForwardRefMarker, TickCycleMarker};
use crate::ir::{DebugInstantiate, HfPlusLeaf, HfPlusNode, TeeNode};
use crate::location::cluster::CLUSTER_SELF_ID;
use crate::location::external_process::{ExternalBincodeStream, ExternalBytesPort};
use crate::location::{
    check_matching_location, CanSend, ExternalProcess, Location, LocationId, NoTick, Tick,
};
use crate::staging_util::get_this_crate;
use crate::{Bounded, Cluster, ClusterId, Optional, Process, Singleton, Unbounded};

/// Marks the stream as being totally ordered, which means that there are
/// no sources of non-determinism (other than intentional ones) that will
/// affect the order of elements.
pub struct TotalOrder {}

/// Marks the stream as having no order, which means that the order of
/// elements may be affected by non-determinism.
///
/// This restricts certain operators, such as `fold` and `reduce`, to only
/// be used with commutative aggregation functions.
pub struct NoOrder {}

/// Helper trait for determining the weakest of two orderings.
#[sealed::sealed]
pub trait MinOrder<Other> {
    /// The weaker of the two orderings.
    type Min;
}

#[sealed::sealed]
impl<T> MinOrder<T> for T {
    type Min = T;
}

#[sealed::sealed]
impl MinOrder<NoOrder> for TotalOrder {
    type Min = NoOrder;
}

#[sealed::sealed]
impl MinOrder<TotalOrder> for NoOrder {
    type Min = NoOrder;
}

/// An ordered sequence stream of elements of type `T`.
///
/// Type Parameters:
/// - `T`: the type of elements in the stream
/// - `L`: the location where the stream is being materialized
/// - `B`: the boundedness of the stream, which is either [`Bounded`]
///   or [`Unbounded`]
/// - `Order`: the ordering of the stream, which is either [`TotalOrder`]
///   or [`NoOrder`] (default is [`TotalOrder`])
pub struct Stream<T, L, B, Order = TotalOrder> {
    location: L,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    _phantom: PhantomData<(T, L, B, Order)>,
}

impl<'a, T, L: Location<'a>, B> From<Stream<T, L, B, TotalOrder>> for Stream<T, L, B, NoOrder> {
    fn from(stream: Stream<T, L, B, TotalOrder>) -> Stream<T, L, B, NoOrder> {
        Stream {
            location: stream.location,
            ir_node: stream.ir_node,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, L: Location<'a>, B, Order> Stream<T, L, B, Order> {
    fn location_kind(&self) -> LocationId {
        self.location.id()
    }
}

impl<'a, T, L: Location<'a>, Order> DeferTick for Stream<T, Tick<L>, Bounded, Order> {
    fn defer_tick(self) -> Self {
        Stream::defer_tick(self)
    }
}

impl<'a, T, L: Location<'a>, Order> CycleCollection<'a, TickCycleMarker>
    for Stream<T, Tick<L>, Bounded, Order>
{
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

impl<'a, T, L: Location<'a>, Order> CycleComplete<'a, TickCycleMarker>
    for Stream<T, Tick<L>, Bounded, Order>
{
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

impl<'a, T, L: Location<'a> + NoTick, B, Order> CycleCollection<'a, ForwardRefMarker>
    for Stream<T, L, B, Order>
{
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

impl<'a, T, L: Location<'a> + NoTick, B, Order> CycleComplete<'a, ForwardRefMarker>
    for Stream<T, L, B, Order>
{
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

impl<'a, T, L: Location<'a>, B, Order> Stream<T, L, B, Order> {
    pub(crate) fn new(location: L, ir_node: HfPlusNode) -> Self {
        Stream {
            location,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone, L: Location<'a>, B, Order> Clone for Stream<T, L, B, Order> {
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

impl<'a, T, L: Location<'a>, B, Order> Stream<T, L, B, Order> {
    pub fn map<U, F: Fn(T) -> U + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F, L>,
    ) -> Stream<U, L, B, Order> {
        let f = f.splice_fn1_ctx(&self.location).into();
        Stream::new(
            self.location,
            HfPlusNode::Map {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn cloned(self) -> Stream<T, L, B, Order>
    where
        T: Clone,
    {
        self.map(q!(|d| d.clone()))
    }

    pub fn flat_map<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F, L>,
    ) -> Stream<U, L, B, Order> {
        let f = f.splice_fn1_ctx(&self.location).into();
        Stream::new(
            self.location,
            HfPlusNode::FlatMap {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn flatten<U>(self) -> Stream<U, L, B, Order>
    where
        T: IntoIterator<Item = U>,
    {
        self.flat_map(q!(|d| d))
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F, L>,
    ) -> Stream<T, L, B, Order> {
        let f = f.splice_fn1_borrow_ctx(&self.location).into();
        Stream::new(
            self.location,
            HfPlusNode::Filter {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F, L>,
    ) -> Stream<U, L, B, Order> {
        let f = f.splice_fn1_ctx(&self.location).into();
        Stream::new(
            self.location,
            HfPlusNode::FilterMap {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn cross_singleton<O>(
        self,
        other: impl Into<Optional<O, L, Bounded>>,
    ) -> Stream<(T, O), L, B, Order>
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
    pub fn continue_if<U>(self, signal: Optional<U, L, Bounded>) -> Stream<T, L, B, Order> {
        self.cross_singleton(signal.map(q!(|_u| ())))
            .map(q!(|(d, _signal)| d))
    }

    /// Allow this stream through if the other stream is empty, otherwise the output is empty.
    pub fn continue_unless<U>(self, other: Optional<U, L, Bounded>) -> Stream<T, L, B, Order> {
        self.continue_if(other.into_stream().count().filter(q!(|c| *c == 0)))
    }

    pub fn cross_product<O>(self, other: Stream<O, L, B, Order>) -> Stream<(T, O), L, B, Order>
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

    pub fn unique(self) -> Stream<T, L, B, Order>
    where
        T: Eq + Hash,
    {
        Stream::new(
            self.location,
            HfPlusNode::Unique(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn filter_not_in<O2>(self, other: Stream<T, L, Bounded, O2>) -> Stream<T, L, Bounded, Order>
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

    pub fn inspect<F: Fn(&T) + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F, L>,
    ) -> Stream<T, L, B, Order> {
        let f = f.splice_fn1_borrow_ctx(&self.location).into();

        if L::is_top_level() {
            Stream::new(
                self.location,
                HfPlusNode::Persist(Box::new(HfPlusNode::Inspect {
                    f,
                    input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                })),
            )
        } else {
            Stream::new(
                self.location,
                HfPlusNode::Inspect {
                    f,
                    input: Box::new(self.ir_node.into_inner()),
                },
            )
        }
    }

    pub fn assume_ordering<O>(self) -> Stream<T, L, B, O> {
        Stream::new(self.location, self.ir_node.into_inner())
    }
}

impl<'a, T, L: Location<'a>, B, Order> Stream<T, L, B, Order>
where
    Order: MinOrder<NoOrder, Min = NoOrder>,
{
    pub fn fold_commutative<A, I: Fn() -> A + 'a, F: Fn(&mut A, T)>(
        self,
        init: impl IntoQuotedMut<'a, I, L>,
        comb: impl IntoQuotedMut<'a, F, L>,
    ) -> Singleton<A, L, B> {
        let init = init.splice_fn0_ctx(&self.location).into();
        let comb = comb.splice_fn2_borrow_mut_ctx(&self.location).into();

        let mut core = HfPlusNode::Fold {
            init,
            acc: comb,
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

    pub fn reduce_commutative<F: Fn(&mut T, T) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F, L>,
    ) -> Optional<T, L, B> {
        let f = comb.splice_fn2_borrow_mut_ctx(&self.location).into();
        let mut core = HfPlusNode::Reduce {
            f,
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
        self.reduce_commutative(q!(|curr, new| {
            if new > *curr {
                *curr = new;
            }
        }))
    }

    pub fn max_by_key<K: Ord, F: Fn(&T) -> K + 'a>(
        self,
        key: impl IntoQuotedMut<'a, F, L> + Copy,
    ) -> Optional<T, L, B> {
        let f = key.splice_fn1_borrow_ctx(&self.location);

        let wrapped: syn::Expr = parse_quote!({
            let key_fn = #f;
            move |curr, new| {
                if key_fn(&new) > key_fn(&*curr) {
                    *curr = new;
                }
            }
        });

        let mut core = HfPlusNode::Reduce {
            f: wrapped.into(),
            input: Box::new(self.ir_node.into_inner()),
        };

        if L::is_top_level() {
            core = HfPlusNode::Persist(Box::new(core));
        }

        Optional::new(self.location, core)
    }

    pub fn min(self) -> Optional<T, L, B>
    where
        T: Ord,
    {
        self.reduce_commutative(q!(|curr, new| {
            if new < *curr {
                *curr = new;
            }
        }))
    }

    pub fn count(self) -> Singleton<usize, L, B> {
        self.fold_commutative(q!(|| 0usize), q!(|count, _| *count += 1))
    }
}

impl<'a, T, L: Location<'a>, B> Stream<T, L, B, TotalOrder> {
    pub fn enumerate(self) -> Stream<(usize, T), L, B, TotalOrder> {
        if L::is_top_level() {
            Stream::new(
                self.location,
                HfPlusNode::Persist(Box::new(HfPlusNode::Enumerate {
                    is_static: true,
                    input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                })),
            )
        } else {
            Stream::new(
                self.location,
                HfPlusNode::Enumerate {
                    is_static: false,
                    input: Box::new(self.ir_node.into_inner()),
                },
            )
        }
    }

    pub fn first(self) -> Optional<T, L, B> {
        Optional::new(self.location, self.ir_node.into_inner())
    }

    pub fn last(self) -> Optional<T, L, B> {
        self.reduce(q!(|curr, new| *curr = new))
    }

    pub fn fold<A, I: Fn() -> A + 'a, F: Fn(&mut A, T)>(
        self,
        init: impl IntoQuotedMut<'a, I, L>,
        comb: impl IntoQuotedMut<'a, F, L>,
    ) -> Singleton<A, L, B> {
        let init = init.splice_fn0_ctx(&self.location).into();
        let comb = comb.splice_fn2_borrow_mut_ctx(&self.location).into();

        let mut core = HfPlusNode::Fold {
            init,
            acc: comb,
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
        comb: impl IntoQuotedMut<'a, F, L>,
    ) -> Optional<T, L, B> {
        let f = comb.splice_fn2_borrow_mut_ctx(&self.location).into();
        let mut core = HfPlusNode::Reduce {
            f,
            input: Box::new(self.ir_node.into_inner()),
        };

        if L::is_top_level() {
            core = HfPlusNode::Persist(Box::new(core));
        }

        Optional::new(self.location, core)
    }
}

impl<'a, T, L: Location<'a>> Stream<T, L, Bounded, TotalOrder> {
    pub fn chain(
        self,
        other: Stream<T, L, Bounded, TotalOrder>,
    ) -> Stream<T, L, Bounded, TotalOrder> {
        check_matching_location(&self.location, &other.location);

        Stream::new(
            self.location,
            HfPlusNode::Chain(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }
}

impl<'a, T, L: Location<'a> + NoTick> Stream<T, L, Unbounded, NoOrder> {
    pub fn union(
        self,
        other: Stream<T, L, Unbounded, NoOrder>,
    ) -> Stream<T, L, Unbounded, NoOrder> {
        let tick = self.location.tick();
        unsafe {
            // SAFETY: Because the inputs and outputs are unordered,
            // we can interleave batches from both streams.
            self.tick_batch(&tick)
                .union(other.tick_batch(&tick))
                .all_ticks()
        }
    }
}

impl<'a, T, L: Location<'a>, Order> Stream<T, L, Bounded, Order> {
    pub fn sort(self) -> Stream<T, L, Bounded, TotalOrder>
    where
        T: Ord,
    {
        Stream::new(
            self.location,
            HfPlusNode::Sort(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn union<B2, O2>(self, other: Stream<T, L, B2, O2>) -> Stream<T, L, B2, Order::Min>
    where
        Order: MinOrder<O2>,
    {
        check_matching_location(&self.location, &other.location);

        Stream::new(
            self.location,
            HfPlusNode::Chain(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }
}

impl<'a, K, V1, L: Location<'a>, B, Order> Stream<(K, V1), L, B, Order> {
    pub fn join<V2, O2>(self, n: Stream<(K, V2), L, B, O2>) -> Stream<(K, (V1, V2)), L, B, NoOrder>
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

    pub fn anti_join<O2>(self, n: Stream<K, L, Bounded, O2>) -> Stream<(K, V1), L, B, Order>
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
        init: impl IntoQuotedMut<'a, I, Tick<L>>,
        comb: impl IntoQuotedMut<'a, F, Tick<L>>,
    ) -> Stream<(K, A), Tick<L>, Bounded> {
        let init = init.splice_fn0_ctx(&self.location).into();
        let comb = comb.splice_fn2_borrow_mut_ctx(&self.location).into();

        Stream::new(
            self.location,
            HfPlusNode::FoldKeyed {
                init,
                acc: comb,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn reduce_keyed<F: Fn(&mut V, V) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F, Tick<L>>,
    ) -> Stream<(K, V), Tick<L>, Bounded> {
        let f = comb.splice_fn2_borrow_mut_ctx(&self.location).into();

        Stream::new(
            self.location,
            HfPlusNode::ReduceKeyed {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, K: Eq + Hash, V, L: Location<'a>, Order> Stream<(K, V), Tick<L>, Bounded, Order>
where
    Order: MinOrder<NoOrder, Min = NoOrder>,
{
    pub fn fold_keyed_commutative<A, I: Fn() -> A + 'a, F: Fn(&mut A, V) + 'a>(
        self,
        init: impl IntoQuotedMut<'a, I, Tick<L>>,
        comb: impl IntoQuotedMut<'a, F, Tick<L>>,
    ) -> Stream<(K, A), Tick<L>, Bounded, Order> {
        let init = init.splice_fn0_ctx(&self.location).into();
        let comb = comb.splice_fn2_borrow_mut_ctx(&self.location).into();

        Stream::new(
            self.location,
            HfPlusNode::FoldKeyed {
                init,
                acc: comb,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn reduce_keyed_commutative<F: Fn(&mut V, V) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F, Tick<L>>,
    ) -> Stream<(K, V), Tick<L>, Bounded, Order> {
        let f = comb.splice_fn2_borrow_mut_ctx(&self.location).into();

        Stream::new(
            self.location,
            HfPlusNode::ReduceKeyed {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, T, L: Location<'a> + NoTick, B, Order> Stream<T, L, B, Order> {
    pub unsafe fn tick_batch(self, tick: &Tick<L>) -> Stream<T, Tick<L>, Bounded, Order> {
        Stream::new(
            tick.clone(),
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub unsafe fn tick_prefix(self, tick: &Tick<L>) -> Stream<T, Tick<L>, Bounded, Order>
    where
        T: Clone,
    {
        self.tick_batch(tick).persist()
    }

    pub unsafe fn sample_every(
        self,
        interval: impl QuotedWithContext<'a, std::time::Duration, L> + Copy + 'a,
    ) -> Stream<T, L, Unbounded, Order> {
        let samples = self.location.source_interval(interval);
        let tick = self.location.tick();
        self.tick_batch(&tick)
            .continue_if(samples.tick_batch(&tick).first())
            .all_ticks()
    }

    pub fn for_each<F: Fn(T) + 'a>(self, f: impl IntoQuotedMut<'a, F, L>) {
        let f = f.splice_fn1_ctx(&self.location).into();
        self.location
            .flow_state()
            .borrow_mut()
            .leaves
            .as_mut()
            .expect(FLOW_USED_MESSAGE)
            .push(HfPlusLeaf::ForEach {
                input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                f,
            });
    }

    pub fn dest_sink<S: Unpin + futures::Sink<T> + 'a>(
        self,
        sink: impl QuotedWithContext<'a, S, L>,
    ) {
        self.location
            .flow_state()
            .borrow_mut()
            .leaves
            .as_mut()
            .expect(FLOW_USED_MESSAGE)
            .push(HfPlusLeaf::DestSink {
                sink: sink.splice_typed_ctx(&self.location).into(),
                input: Box::new(self.ir_node.into_inner()),
            });
    }
}

impl<'a, T, L: Location<'a>, Order> Stream<T, Tick<L>, Bounded, Order> {
    pub fn all_ticks(self) -> Stream<T, L, Unbounded, Order> {
        Stream::new(
            self.location.outer().clone(),
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<T, Tick<L>, Bounded, Order>
    where
        T: Clone,
    {
        Stream::new(
            self.location,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Stream<T, Tick<L>, Bounded, Order> {
        Stream::new(
            self.location,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn delta(self) -> Stream<T, Tick<L>, Bounded, Order> {
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

impl<'a, T, C1, B, Order> Stream<T, Cluster<'a, C1>, B, Order> {
    pub fn decouple_cluster<C2: 'a, Tag>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<T, Cluster<'a, C2>, Unbounded, Order>
    where
        Cluster<'a, C1>: Location<'a, Root = Cluster<'a, C1>>,
        Cluster<'a, C1>:
            CanSend<'a, Cluster<'a, C2>, In<T> = (ClusterId<C2>, T), Out<T> = (Tag, T)>,
        T: Clone + Serialize + DeserializeOwned,
        Order:
            MinOrder<<Cluster<'a, C1> as CanSend<'a, Cluster<'a, C2>>>::OutStrongestOrder<Order>>,
    {
        self.map(q!(move |b| (
            ClusterId::from_raw(CLUSTER_SELF_ID.raw_id),
            b.clone()
        )))
        .send_bincode_interleaved(other)
        .assume_ordering() // this is safe because we are mapping clusters 1:1
    }
}

impl<'a, T, L: Location<'a> + NoTick, B, Order> Stream<T, L, B, Order> {
    pub fn decouple_process<P2>(
        self,
        other: &Process<'a, P2>,
    ) -> Stream<T, Process<'a, P2>, Unbounded, Order>
    where
        L: CanSend<'a, Process<'a, P2>, In<T> = T, Out<T> = T>,
        T: Clone + Serialize + DeserializeOwned,
        Order: MinOrder<L::OutStrongestOrder<Order>, Min = Order>,
    {
        self.send_bincode::<Process<'a, P2>, T>(other)
    }

    pub fn send_bincode<L2: Location<'a>, CoreType>(
        self,
        other: &L2,
    ) -> Stream<L::Out<CoreType>, L2, Unbounded, Order::Min>
    where
        L: CanSend<'a, L2, In<CoreType> = T>,
        CoreType: Serialize + DeserializeOwned,
        Order: MinOrder<L::OutStrongestOrder<Order>>,
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

    pub fn send_bytes<L2: Location<'a>>(
        self,
        other: &L2,
    ) -> Stream<L::Out<Bytes>, L2, Unbounded, Order::Min>
    where
        L: CanSend<'a, L2, In<Bytes> = T>,
        Order: MinOrder<L::OutStrongestOrder<Order>>,
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
    ) -> Stream<CoreType, L2, Unbounded, Order::Min>
    where
        L: CanSend<'a, L2, In<CoreType> = T, Out<CoreType> = (Tag, CoreType)>,
        CoreType: Serialize + DeserializeOwned,
        Order: MinOrder<L::OutStrongestOrder<Order>>,
    {
        self.send_bincode::<L2, CoreType>(other).map(q!(|(_, b)| b))
    }

    pub fn send_bytes_interleaved<L2: Location<'a>, Tag>(
        self,
        other: &L2,
    ) -> Stream<Bytes, L2, Unbounded, Order::Min>
    where
        L: CanSend<'a, L2, In<Bytes> = T, Out<Bytes> = (Tag, Bytes)>,
        Order: MinOrder<L::OutStrongestOrder<Order>>,
    {
        self.send_bytes::<L2>(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bincode<C2: 'a>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<L::Out<T>, Cluster<'a, C2>, Unbounded, Order::Min>
    where
        L: CanSend<'a, Cluster<'a, C2>, In<T> = (ClusterId<C2>, T)>,
        T: Clone + Serialize + DeserializeOwned,
        Order: MinOrder<L::OutStrongestOrder<Order>>,
    {
        let ids = other.members();

        self.flat_map(q!(|b| ids.iter().map(move |id| (
            ::std::clone::Clone::clone(id),
            ::std::clone::Clone::clone(&b)
        ))))
        .send_bincode(other)
    }

    pub fn broadcast_bincode_interleaved<C2: 'a, Tag>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<T, Cluster<'a, C2>, Unbounded, Order::Min>
    where
        L: CanSend<'a, Cluster<'a, C2>, In<T> = (ClusterId<C2>, T), Out<T> = (Tag, T)> + 'a,
        T: Clone + Serialize + DeserializeOwned,
        Order: MinOrder<L::OutStrongestOrder<Order>>,
    {
        self.broadcast_bincode(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bytes<C2: 'a>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<L::Out<Bytes>, Cluster<'a, C2>, Unbounded, Order::Min>
    where
        L: CanSend<'a, Cluster<'a, C2>, In<Bytes> = (ClusterId<C2>, T)> + 'a,
        T: Clone,
        Order: MinOrder<L::OutStrongestOrder<Order>>,
    {
        let ids = other.members();

        self.flat_map(q!(|b| ids.iter().map(move |id| (
            ::std::clone::Clone::clone(id),
            ::std::clone::Clone::clone(&b)
        ))))
        .send_bytes(other)
    }

    pub fn broadcast_bytes_interleaved<C2: 'a, Tag>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<Bytes, Cluster<'a, C2>, Unbounded, Order::Min>
    where
        L: CanSend<'a, Cluster<'a, C2>, In<Bytes> = (ClusterId<C2>, T), Out<Bytes> = (Tag, Bytes)>
            + 'a,
        T: Clone,
        Order: MinOrder<L::OutStrongestOrder<Order>>,
    {
        self.broadcast_bytes(other).map(q!(|(_, b)| b))
    }
}

#[expect(clippy::type_complexity, reason = "ordering semantics for round-robin")]
impl<'a, T, L: Location<'a> + NoTick, B> Stream<T, L, B, TotalOrder> {
    pub fn round_robin_bincode<C2: 'a>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<
        L::Out<T>,
        Cluster<'a, C2>,
        Unbounded,
        <TotalOrder as MinOrder<L::OutStrongestOrder<TotalOrder>>>::Min,
    >
    where
        L: CanSend<'a, Cluster<'a, C2>, In<T> = (ClusterId<C2>, T)>,
        T: Clone + Serialize + DeserializeOwned,
        TotalOrder: MinOrder<L::OutStrongestOrder<TotalOrder>>,
    {
        let ids = other.members();

        self.enumerate()
            .map(q!(|(i, w)| (ids[i % ids.len()], w)))
            .send_bincode(other)
    }

    pub fn round_robin_bincode_interleaved<C2: 'a, Tag>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<
        T,
        Cluster<'a, C2>,
        Unbounded,
        <TotalOrder as MinOrder<L::OutStrongestOrder<TotalOrder>>>::Min,
    >
    where
        L: CanSend<'a, Cluster<'a, C2>, In<T> = (ClusterId<C2>, T), Out<T> = (Tag, T)> + 'a,
        T: Clone + Serialize + DeserializeOwned,
        TotalOrder: MinOrder<L::OutStrongestOrder<TotalOrder>>,
    {
        self.round_robin_bincode(other).map(q!(|(_, b)| b))
    }

    pub fn round_robin_bytes<C2: 'a>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<
        L::Out<Bytes>,
        Cluster<'a, C2>,
        Unbounded,
        <TotalOrder as MinOrder<L::OutStrongestOrder<TotalOrder>>>::Min,
    >
    where
        L: CanSend<'a, Cluster<'a, C2>, In<Bytes> = (ClusterId<C2>, T)> + 'a,
        T: Clone,
        TotalOrder: MinOrder<L::OutStrongestOrder<TotalOrder>>,
    {
        let ids = other.members();

        self.enumerate()
            .map(q!(|(i, w)| (ids[i % ids.len()], w)))
            .send_bytes(other)
    }

    pub fn round_robin_bytes_interleaved<C2: 'a, Tag>(
        self,
        other: &Cluster<'a, C2>,
    ) -> Stream<
        Bytes,
        Cluster<'a, C2>,
        Unbounded,
        <TotalOrder as MinOrder<L::OutStrongestOrder<TotalOrder>>>::Min,
    >
    where
        L: CanSend<'a, Cluster<'a, C2>, In<Bytes> = (ClusterId<C2>, T), Out<Bytes> = (Tag, Bytes)>
            + 'a,
        T: Clone,
        TotalOrder: MinOrder<L::OutStrongestOrder<TotalOrder>>,
    {
        self.round_robin_bytes(other).map(q!(|(_, b)| b))
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
