use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use stageleft::{q, IntoQuotedMut, Quoted};
use syn::parse_quote;

use crate::builder::FLOW_USED_MESSAGE;
use crate::cycle::{CycleCollection, CycleComplete, DeferTick, ForwardRef, TickCycle};
use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource, TeeNode};
use crate::location::{check_matching_location, LocationId, NoTick};
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

impl<'a, T, L: Location<'a>> CycleCollection<'a, TickCycle> for Optional<T, Tick<L>, Bounded> {
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

impl<'a, T, L: Location<'a>> CycleComplete<'a, TickCycle> for Optional<T, Tick<L>, Bounded> {
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

impl<'a, T, L: Location<'a>> CycleCollection<'a, ForwardRef> for Optional<T, Tick<L>, Bounded> {
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

impl<'a, T, L: Location<'a>> CycleComplete<'a, ForwardRef> for Optional<T, Tick<L>, Bounded> {
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

impl<'a, T, L: Location<'a> + NoTick, B> CycleCollection<'a, ForwardRef> for Optional<T, L, B> {
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

impl<'a, T, L: Location<'a> + NoTick, B> CycleComplete<'a, ForwardRef> for Optional<T, L, B> {
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
    // TODO(shadaj): this is technically incorrect; we should only return the first element of the stream
    pub fn into_stream(self) -> Stream<T, L, B> {
        if L::is_top_level() {
            panic!("Converting an optional to a stream is not yet supported at the top level");
        }

        Stream::new(self.location, self.ir_node.into_inner())
    }

    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Optional<U, L, B> {
        Optional::new(
            self.location,
            HfPlusNode::Map {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
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

    pub fn filter<F: Fn(&T) -> bool + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Optional<T, L, B> {
        Optional::new(
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
    ) -> Optional<U, L, B> {
        Optional::new(
            self.location,
            HfPlusNode::FilterMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn union(self, other: Optional<T, L, B>) -> Optional<T, L, B> {
        check_matching_location(&self.location, &other.location);

        if L::is_top_level() {
            Optional::new(
                self.location,
                HfPlusNode::Persist(Box::new(HfPlusNode::Union(
                    Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                    Box::new(HfPlusNode::Unpersist(Box::new(other.ir_node.into_inner()))),
                ))),
            )
        } else {
            Optional::new(
                self.location,
                HfPlusNode::Union(
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
                HfPlusNode::Persist(Box::new(HfPlusNode::Union(
                    Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                    Box::new(HfPlusNode::Unpersist(Box::new(other.ir_node.into_inner()))),
                ))),
            )
        } else {
            Singleton::new(
                self.location,
                HfPlusNode::Union(
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

    pub fn then<U>(self, value: Singleton<U, L, Bounded>) -> Optional<U, L, Bounded> {
        value.continue_if(self)
    }
}

impl<'a, T, L: Location<'a> + NoTick, B> Optional<T, L, B> {
    pub fn latest_tick(self, tick: &Tick<L>) -> Optional<T, Tick<L>, Bounded> {
        Optional::new(
            tick.clone(),
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn tick_samples(self) -> Stream<T, L, Unbounded> {
        let tick = self.location.tick();
        self.latest_tick(&tick).all_ticks()
    }

    pub fn sample_every(
        self,
        interval: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<T, L, Unbounded> {
        let samples = self.location.source_interval(interval);
        let tick = self.location.tick();

        self.latest_tick(&tick)
            .continue_if(samples.tick_batch(&tick).first())
            .all_ticks()
    }
}

impl<'a, T, L: Location<'a>> Optional<T, Tick<L>, Bounded> {
    pub fn all_ticks(self) -> Stream<T, L, Unbounded> {
        Stream::new(
            self.location.outer().clone(),
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn latest(self) -> Optional<T, L, Unbounded> {
        Optional::new(
            self.location.outer().clone(),
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
