use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use stageleft::{q, IntoQuotedMut, Quoted};

use crate::builder::FLOW_USED_MESSAGE;
use crate::cycle::{
    CycleCollection, CycleCollectionWithInitial, CycleComplete, DeferTick, ForwardRef, TickCycle,
};
use crate::ir::{HfPlusLeaf, HfPlusNode, TeeNode};
use crate::location::{check_matching_location, Location, LocationId, NoTick, Tick};
use crate::stream::{Bounded, Unbounded};
use crate::{Optional, Stream};

pub struct Singleton<T, L, B> {
    pub(crate) location: L,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    _phantom: PhantomData<(T, L, B)>,
}

impl<'a, T, L: Location<'a>, B> Singleton<T, L, B> {
    pub(crate) fn new(location: L, ir_node: HfPlusNode) -> Self {
        Singleton {
            location,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }

    fn location_kind(&self) -> LocationId {
        self.location.id()
    }
}

impl<'a, T, L: Location<'a>> From<Singleton<T, L, Bounded>> for Singleton<T, L, Unbounded> {
    fn from(singleton: Singleton<T, L, Bounded>) -> Self {
        Singleton::new(singleton.location, singleton.ir_node.into_inner())
    }
}

impl<'a, T, L: Location<'a>> DeferTick for Singleton<T, Tick<L>, Bounded> {
    fn defer_tick(self) -> Self {
        Singleton::defer_tick(self)
    }
}

impl<'a, T, L: Location<'a>> CycleCollectionWithInitial<'a, TickCycle>
    for Singleton<T, Tick<L>, Bounded>
{
    type Location = Tick<L>;

    fn create_source(ident: syn::Ident, initial: Self, location: Tick<L>) -> Self {
        let location_id = location.id();
        Singleton::new(
            location,
            HfPlusNode::Union(
                Box::new(HfPlusNode::CycleSource {
                    ident,
                    location_kind: location_id,
                }),
                initial.ir_node.into_inner().into(),
            ),
        )
    }
}

impl<'a, T, L: Location<'a>> CycleComplete<'a, TickCycle> for Singleton<T, Tick<L>, Bounded> {
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

impl<'a, T, L: Location<'a>> CycleCollection<'a, ForwardRef> for Singleton<T, Tick<L>, Bounded> {
    type Location = Tick<L>;

    fn create_source(ident: syn::Ident, location: Tick<L>) -> Self {
        let location_id = location.id();
        Singleton::new(
            location,
            HfPlusNode::CycleSource {
                ident,
                location_kind: location_id,
            },
        )
    }
}

impl<'a, T, L: Location<'a>> CycleComplete<'a, ForwardRef> for Singleton<T, Tick<L>, Bounded> {
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

impl<'a, T: Clone, L: Location<'a>, B> Clone for Singleton<T, L, B> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: TeeNode(Rc::new(RefCell::new(orig_ir_node))),
            };
        }

        if let HfPlusNode::Tee { inner } = self.ir_node.borrow().deref() {
            Singleton {
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

impl<'a, T, L: Location<'a>, B> Singleton<T, L, B> {
    // TODO(shadaj): this is technically incorrect; we should only return the first element of the stream
    pub fn into_stream(self) -> Stream<T, L, Bounded> {
        Stream::new(self.location, self.ir_node.into_inner())
    }

    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Singleton<U, L, B> {
        Singleton::new(
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

    pub fn zip<Other>(self, other: Other) -> <Self as ZipResult<'a, Other>>::Out
    where
        Self: ZipResult<'a, Other, Location = L>,
    {
        check_matching_location(&self.location, &Self::other_location(&other));

        if L::is_top_level() {
            Self::make(
                self.location,
                HfPlusNode::Persist(Box::new(HfPlusNode::CrossSingleton(
                    Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                    Box::new(HfPlusNode::Unpersist(Box::new(Self::other_ir_node(other)))),
                ))),
            )
        } else {
            Self::make(
                self.location,
                HfPlusNode::CrossSingleton(
                    Box::new(self.ir_node.into_inner()),
                    Box::new(Self::other_ir_node(other)),
                ),
            )
        }
    }
}

impl<'a, T, L: Location<'a>> Singleton<T, L, Bounded> {
    pub fn continue_if<U>(self, signal: Optional<U, L, Bounded>) -> Optional<T, L, Bounded> {
        self.zip(signal.map(q!(|_u| ()))).map(q!(|(d, _signal)| d))
    }

    pub fn continue_unless<U>(self, other: Optional<U, L, Bounded>) -> Optional<T, L, Bounded> {
        self.continue_if(other.into_stream().count().filter(q!(|c| *c == 0)))
    }
}

impl<'a, T, L: Location<'a> + NoTick, B> Singleton<T, L, B> {
    pub fn latest_tick(self, tick: &Tick<L>) -> Singleton<T, Tick<L>, Bounded> {
        Singleton::new(
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

impl<'a, T, L: Location<'a>> Singleton<T, Tick<L>, Bounded> {
    pub fn all_ticks(self) -> Stream<T, L, Unbounded> {
        Stream::new(
            self.location.outer().clone(),
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn latest(self) -> Singleton<T, L, Unbounded> {
        Singleton::new(
            self.location.outer().clone(),
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Singleton<T, Tick<L>, Bounded> {
        Singleton::new(
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

pub trait ZipResult<'a, Other> {
    type Out;
    type Location;

    fn other_location(other: &Other) -> Self::Location;
    fn other_ir_node(other: Other) -> HfPlusNode;

    fn make(location: Self::Location, ir_node: HfPlusNode) -> Self::Out;
}

impl<'a, T, U: Clone, L: Location<'a>, B> ZipResult<'a, Singleton<U, L, B>> for Singleton<T, L, B> {
    type Out = Singleton<(T, U), L, B>;
    type Location = L;

    fn other_location(other: &Singleton<U, L, B>) -> L {
        other.location.clone()
    }

    fn other_ir_node(other: Singleton<U, L, B>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location: L, ir_node: HfPlusNode) -> Self::Out {
        Singleton::new(location, ir_node)
    }
}

impl<'a, T, U: Clone, L: Location<'a>, B> ZipResult<'a, Optional<U, L, B>> for Singleton<T, L, B> {
    type Out = Optional<(T, U), L, B>;
    type Location = L;

    fn other_location(other: &Optional<U, L, B>) -> L {
        other.location.clone()
    }

    fn other_ir_node(other: Optional<U, L, B>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location: L, ir_node: HfPlusNode) -> Self::Out {
        Optional::new(location, ir_node)
    }
}
