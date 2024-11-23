use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use stageleft::{q, IntoQuotedMut, QuotedWithContext};
use syn::parse_quote;

use crate::builder::FLOW_USED_MESSAGE;
use crate::cycle::{CycleCollection, CycleComplete, DeferTick, ForwardRefMarker, TickCycleMarker};
use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource, TeeNode};
use crate::location::tick::{NoTimestamp, Timestamped};
use crate::location::{check_matching_location, LocationId, NoTick};
use crate::singleton::ZipResult;
use crate::stream::NoOrder;
use crate::{Bounded, Location, Singleton, Stream, Tick, Unbounded};

pub struct Optional<T, L, B> {
    pub(crate) location: L,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    _phantom: PhantomData<(T, L, B)>,
}

impl<'a, T, L: Location<'a>, B> Optional<T, L, B> {
    pub(crate) fn new(location: L, ir_node: HfPlusNode) -> Self {
        Optional {
            location,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }

    pub fn some(singleton: Singleton<T, L, B>) -> Self {
        Optional::new(singleton.location, singleton.ir_node.into_inner())
    }

    fn location_kind(&self) -> LocationId {
        self.location.id()
    }
}

impl<'a, T, L: Location<'a>> DeferTick for Optional<T, Tick<L>, Bounded> {
    fn defer_tick(self) -> Self {
        Optional::defer_tick(self)
    }
}

impl<'a, T, L: Location<'a>> CycleCollection<'a, TickCycleMarker>
    for Optional<T, Tick<L>, Bounded>
{
    type Location = Tick<L>;

    fn create_source(ident: syn::Ident, location: Tick<L>) -> Self {
        let location_id = location.id();
        Optional::new(
            location,
            HfPlusNode::CycleSource {
                ident,
                location_kind: location_id,
            },
        )
    }
}

impl<'a, T, L: Location<'a>> CycleComplete<'a, TickCycleMarker> for Optional<T, Tick<L>, Bounded> {
    fn complete(self, ident: syn::Ident, expected_location: LocationId) {
        assert_eq!(
            self.location.id(),
            expected_location,
            "locations do not match"
        );
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

impl<'a, T, L: Location<'a>> CycleCollection<'a, ForwardRefMarker>
    for Optional<T, Tick<L>, Bounded>
{
    type Location = Tick<L>;

    fn create_source(ident: syn::Ident, location: Tick<L>) -> Self {
        let location_id = location.id();
        Optional::new(
            location,
            HfPlusNode::CycleSource {
                ident,
                location_kind: location_id,
            },
        )
    }
}

impl<'a, T, L: Location<'a>> CycleComplete<'a, ForwardRefMarker> for Optional<T, Tick<L>, Bounded> {
    fn complete(self, ident: syn::Ident, expected_location: LocationId) {
        assert_eq!(
            self.location.id(),
            expected_location,
            "locations do not match"
        );
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

impl<'a, T, L: Location<'a> + NoTick, B> CycleCollection<'a, ForwardRefMarker>
    for Optional<T, L, B>
{
    type Location = L;

    fn create_source(ident: syn::Ident, location: L) -> Self {
        let location_id = location.id();
        Optional::new(
            location,
            HfPlusNode::Persist(Box::new(HfPlusNode::CycleSource {
                ident,
                location_kind: location_id,
            })),
        )
    }
}

impl<'a, T, L: Location<'a> + NoTick, B> CycleComplete<'a, ForwardRefMarker> for Optional<T, L, B> {
    fn complete(self, ident: syn::Ident, expected_location: LocationId) {
        assert_eq!(
            self.location.id(),
            expected_location,
            "locations do not match"
        );
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

impl<'a, T, L: Location<'a>> From<Optional<T, L, Bounded>> for Optional<T, L, Unbounded> {
    fn from(singleton: Optional<T, L, Bounded>) -> Self {
        Optional::new(singleton.location, singleton.ir_node.into_inner())
    }
}

impl<'a, T, L: Location<'a>, B> From<Singleton<T, L, B>> for Optional<T, L, B> {
    fn from(singleton: Singleton<T, L, B>) -> Self {
        Optional::some(singleton)
    }
}

impl<'a, T: Clone, L: Location<'a>, B> Clone for Optional<T, L, B> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: TeeNode(Rc::new(RefCell::new(orig_ir_node))),
            };
        }

        if let HfPlusNode::Tee { inner } = self.ir_node.borrow().deref() {
            Optional {
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

impl<'a, T, L: Location<'a>, B> Optional<T, L, B> {
    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F, L>) -> Optional<U, L, B> {
        let f = f.splice_fn1_ctx(&self.location).into();
        Optional::new(
            self.location,
            HfPlusNode::Map {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn flat_map_ordered<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F, L>,
    ) -> Stream<U, L, B> {
        let f = f.splice_fn1_ctx(&self.location).into();
        Stream::new(
            self.location,
            HfPlusNode::FlatMap {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn flat_map_unordered<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F, L>,
    ) -> Stream<U, L, B, NoOrder> {
        let f = f.splice_fn1_ctx(&self.location).into();
        Stream::new(
            self.location,
            HfPlusNode::FlatMap {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn flatten_ordered<U>(self) -> Stream<U, L, B>
    where
        T: IntoIterator<Item = U>,
    {
        self.flat_map_ordered(q!(|v| v))
    }

    pub fn flatten_unordered<U>(self) -> Stream<U, L, B, NoOrder>
    where
        T: IntoIterator<Item = U>,
    {
        self.flat_map_unordered(q!(|v| v))
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F, L>,
    ) -> Optional<T, L, B> {
        let f = f.splice_fn1_borrow_ctx(&self.location).into();
        Optional::new(
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
    ) -> Optional<U, L, B> {
        let f = f.splice_fn1_ctx(&self.location).into();
        Optional::new(
            self.location,
            HfPlusNode::FilterMap {
                f,
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn union(self, other: Optional<T, L, B>) -> Optional<T, L, B> {
        check_matching_location(&self.location, &other.location);

        if L::is_top_level() {
            Optional::new(
                self.location,
                HfPlusNode::Persist(Box::new(HfPlusNode::Chain(
                    Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                    Box::new(HfPlusNode::Unpersist(Box::new(other.ir_node.into_inner()))),
                ))),
            )
        } else {
            Optional::new(
                self.location,
                HfPlusNode::Chain(
                    Box::new(self.ir_node.into_inner()),
                    Box::new(other.ir_node.into_inner()),
                ),
            )
        }
    }

    pub fn zip<O>(self, other: impl Into<Optional<O, L, B>>) -> Optional<(T, O), L, B>
    where
        O: Clone,
    {
        let other: Optional<O, L, B> = other.into();
        check_matching_location(&self.location, &other.location);

        if L::is_top_level() {
            Optional::new(
                self.location,
                HfPlusNode::Persist(Box::new(HfPlusNode::CrossSingleton(
                    Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                    Box::new(HfPlusNode::Unpersist(Box::new(other.ir_node.into_inner()))),
                ))),
            )
        } else {
            Optional::new(
                self.location,
                HfPlusNode::CrossSingleton(
                    Box::new(self.ir_node.into_inner()),
                    Box::new(other.ir_node.into_inner()),
                ),
            )
        }
    }

    pub fn unwrap_or(self, other: Singleton<T, L, B>) -> Singleton<T, L, B> {
        check_matching_location(&self.location, &other.location);

        if L::is_top_level() {
            Singleton::new(
                self.location,
                HfPlusNode::Persist(Box::new(HfPlusNode::Chain(
                    Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                    Box::new(HfPlusNode::Unpersist(Box::new(other.ir_node.into_inner()))),
                ))),
            )
        } else {
            Singleton::new(
                self.location,
                HfPlusNode::Chain(
                    Box::new(self.ir_node.into_inner()),
                    Box::new(other.ir_node.into_inner()),
                ),
            )
        }
    }

    pub fn into_singleton(self) -> Singleton<Option<T>, L, B>
    where
        T: Clone,
    {
        let none: syn::Expr = parse_quote!([::std::option::Option::None]);
        let core_ir = HfPlusNode::Persist(Box::new(HfPlusNode::Source {
            source: HfPlusSource::Iter(none.into()),
            location_kind: self.location.id().root().clone(),
        }));

        let none_singleton = if L::is_top_level() {
            Singleton::new(
                self.location.clone(),
                HfPlusNode::Persist(Box::new(core_ir)),
            )
        } else {
            Singleton::new(self.location.clone(), core_ir)
        };

        self.map(q!(|v| Some(v))).unwrap_or(none_singleton)
    }
}

impl<'a, T, L: Location<'a>> Optional<T, L, Bounded> {
    pub fn continue_if<U>(self, signal: Optional<U, L, Bounded>) -> Optional<T, L, Bounded> {
        self.zip(signal.map(q!(|_u| ()))).map(q!(|(d, _signal)| d))
    }

    pub fn continue_unless<U>(self, other: Optional<U, L, Bounded>) -> Optional<T, L, Bounded> {
        self.continue_if(other.into_stream().count().filter(q!(|c| *c == 0)))
    }

    pub fn then<U>(self, value: Singleton<U, L, Bounded>) -> Optional<U, L, Bounded>
    where
        Singleton<U, L, Bounded>: ZipResult<
            'a,
            Optional<(), L, Bounded>,
            Location = L,
            Out = Optional<(U, ()), L, Bounded>,
        >,
    {
        value.continue_if(self)
    }

    pub fn into_stream(self) -> Stream<T, L, Bounded> {
        if L::is_top_level() {
            panic!("Converting an optional to a stream is not yet supported at the top level");
        }

        Stream::new(self.location, self.ir_node.into_inner())
    }
}

impl<'a, T, L: Location<'a> + NoTick, B> Optional<T, Timestamped<L>, B> {
    /// Given a tick, returns a optional value corresponding to a snapshot of the optional
    /// as of that tick. The snapshot at tick `t + 1` is guaranteed to include at least all
    /// relevant data that contributed to the snapshot at tick `t`.
    ///
    /// # Safety
    /// Because this picks a snapshot of a optional whose value is continuously changing,
    /// the output optional has a non-deterministic value since the snapshot can be at an
    /// arbitrary point in time.
    pub unsafe fn latest_tick(self) -> Optional<T, Tick<L>, Bounded> {
        Optional::new(
            self.location.tick,
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn drop_timestamp(self) -> Optional<T, L, B> {
        Optional::new(self.location.tick.l, self.ir_node.into_inner())
    }
}

impl<'a, T, L: Location<'a> + NoTick, B> Optional<T, L, B> {
    pub fn timestamped(self, tick: &Tick<L>) -> Optional<T, Timestamped<L>, B> {
        Optional::new(
            Timestamped { tick: tick.clone() },
            self.ir_node.into_inner(),
        )
    }

    /// Eagerly samples the optional as fast as possible, returning a stream of snapshots
    /// with order corresponding to increasing prefixes of data contributing to the optional.
    ///
    /// # Safety
    /// At runtime, the optional will be arbitrarily sampled as fast as possible, but due
    /// to non-deterministic batching and arrival of inputs, the output stream is
    /// non-deterministic.
    pub unsafe fn sample_eager(self) -> Stream<T, L, Unbounded> {
        let tick = self.location.tick();

        unsafe {
            // SAFETY: source of intentional non-determinism
            self.timestamped(&tick)
                .latest_tick()
                .all_ticks()
                .drop_timestamp()
        }
    }

    /// Given a time interval, returns a stream corresponding to snapshots of the optional
    /// value taken at various points in time. Because the input optional may be
    /// [`Unbounded`], there are no guarantees on what these snapshots are other than they
    /// represent the value of the optional given some prefix of the streams leading up to
    /// it.
    ///
    /// # Safety
    /// The output stream is non-deterministic in which elements are sampled, since this
    /// is controlled by a clock.
    pub unsafe fn sample_every(
        self,
        interval: impl QuotedWithContext<'a, std::time::Duration, L> + Copy + 'a,
    ) -> Stream<T, L, Unbounded>
    where
        L: NoTimestamp,
    {
        let samples = unsafe {
            // SAFETY: source of intentional non-determinism
            self.location.source_interval(interval)
        };
        let tick = self.location.tick();

        unsafe {
            // SAFETY: source of intentional non-determinism
            self.timestamped(&tick)
                .latest_tick()
                .continue_if(samples.timestamped(&tick).tick_batch().first())
                .all_ticks()
                .drop_timestamp()
        }
    }
}

impl<'a, T, L: Location<'a>> Optional<T, Tick<L>, Bounded> {
    pub fn all_ticks(self) -> Stream<T, Timestamped<L>, Unbounded> {
        Stream::new(
            Timestamped {
                tick: self.location,
            },
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn latest(self) -> Optional<T, Timestamped<L>, Unbounded> {
        Optional::new(
            Timestamped {
                tick: self.location,
            },
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Optional<T, Tick<L>, Bounded> {
        Optional::new(
            self.location,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<T, Tick<L>, Bounded> {
        Stream::new(
            self.location,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn delta(self) -> Optional<T, Tick<L>, Bounded> {
        Optional::new(
            self.location,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }
}
