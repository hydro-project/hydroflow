use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use stageleft::{q, IntoQuotedMut, Quoted};

use crate::builder::FlowLeaves;
use crate::cycle::{CycleCollection, CycleCollectionWithInitial, CycleComplete};
use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource};
use crate::location::{Location, LocationId};
use crate::stream::{Bounded, NoTick, Tick, Unbounded};
use crate::Stream;

pub trait CrossResult<'a, Other> {
    type Out;
    fn other_location(other: &Other) -> LocationId;
    fn other_ir_node(other: Other) -> HfPlusNode<'a>;

    fn make(
        location_kind: LocationId,
        ir_leaves: FlowLeaves<'a>,
        ir_node: HfPlusNode<'a>,
    ) -> Self::Out;
}

impl<'a, T, U: Clone, W, C, N: Location> CrossResult<'a, Singleton<'a, U, W, C, N>>
    for Singleton<'a, T, W, C, N>
{
    type Out = Singleton<'a, (T, U), W, C, N>;

    fn other_location(other: &Singleton<'a, U, W, C, N>) -> LocationId {
        other.location_kind
    }

    fn other_ir_node(other: Singleton<'a, U, W, C, N>) -> HfPlusNode<'a> {
        other.ir_node.into_inner()
    }

    fn make(
        location_kind: LocationId,
        ir_leaves: FlowLeaves<'a>,
        ir_node: HfPlusNode<'a>,
    ) -> Self::Out {
        Singleton::new(location_kind, ir_leaves, ir_node)
    }
}

impl<'a, T, U: Clone, W, C, N: Location> CrossResult<'a, Optional<'a, U, W, C, N>>
    for Singleton<'a, T, W, C, N>
{
    type Out = Optional<'a, (T, U), W, C, N>;

    fn other_location(other: &Optional<'a, U, W, C, N>) -> LocationId {
        other.location_kind
    }

    fn other_ir_node(other: Optional<'a, U, W, C, N>) -> HfPlusNode<'a> {
        other.ir_node.into_inner()
    }

    fn make(
        location_kind: LocationId,
        ir_leaves: FlowLeaves<'a>,
        ir_node: HfPlusNode<'a>,
    ) -> Self::Out {
        Optional::new(location_kind, ir_leaves, ir_node)
    }
}

impl<'a, T, U: Clone, W, C, N: Location> CrossResult<'a, Optional<'a, U, W, C, N>>
    for Optional<'a, T, W, C, N>
{
    type Out = Optional<'a, (T, U), W, C, N>;

    fn other_location(other: &Optional<'a, U, W, C, N>) -> LocationId {
        other.location_kind
    }

    fn other_ir_node(other: Optional<'a, U, W, C, N>) -> HfPlusNode<'a> {
        other.ir_node.into_inner()
    }

    fn make(
        location_kind: LocationId,
        ir_leaves: FlowLeaves<'a>,
        ir_node: HfPlusNode<'a>,
    ) -> Self::Out {
        Optional::new(location_kind, ir_leaves, ir_node)
    }
}

impl<'a, T, U: Clone, W, C, N: Location> CrossResult<'a, Singleton<'a, U, W, C, N>>
    for Optional<'a, T, W, C, N>
{
    type Out = Optional<'a, (T, U), W, C, N>;

    fn other_location(other: &Singleton<'a, U, W, C, N>) -> LocationId {
        other.location_kind
    }

    fn other_ir_node(other: Singleton<'a, U, W, C, N>) -> HfPlusNode<'a> {
        other.ir_node.into_inner()
    }

    fn make(
        location_kind: LocationId,
        ir_leaves: FlowLeaves<'a>,
        ir_node: HfPlusNode<'a>,
    ) -> Self::Out {
        Optional::new(location_kind, ir_leaves, ir_node)
    }
}

pub struct Singleton<'a, T, W, C, N: Location> {
    pub(crate) location_kind: LocationId,

    ir_leaves: FlowLeaves<'a>,
    pub(crate) ir_node: RefCell<HfPlusNode<'a>>,

    _phantom: PhantomData<(&'a mut &'a (), T, N, W, C)>,
}

impl<'a, T, W, C, N: Location> Singleton<'a, T, W, C, N> {
    pub(crate) fn new(
        location_kind: LocationId,
        ir_leaves: FlowLeaves<'a>,
        ir_node: HfPlusNode<'a>,
    ) -> Self {
        Singleton {
            location_kind,
            ir_leaves,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, N: Location> CycleComplete<'a> for Singleton<'a, T, Bounded, Tick, N> {
    fn complete(self, ident: syn::Ident) {
        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(self.ir_node.into_inner()),
        });
    }
}

impl<'a, T, N: Location> CycleCollectionWithInitial<'a> for Singleton<'a, T, Bounded, Tick, N> {
    type Location = N;

    fn create_source(
        ident: syn::Ident,
        ir_leaves: FlowLeaves<'a>,
        initial: Self,
        l: LocationId,
    ) -> Self {
        Singleton::new(
            l,
            ir_leaves,
            HfPlusNode::Union(
                Box::new(HfPlusNode::CycleSource {
                    ident,
                    location_kind: l,
                }),
                initial.ir_node.into_inner().into(),
            ),
        )
    }
}

impl<'a, T: Clone, W, C, N: Location> Clone for Singleton<'a, T, W, C, N> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: Rc::new(RefCell::new(orig_ir_node)),
            };
        }

        if let HfPlusNode::Tee { inner } = self.ir_node.borrow().deref() {
            Singleton {
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

impl<'a, T, W, C, N: Location> Singleton<'a, T, W, C, N> {
    pub fn map<U, F: Fn(T) -> U + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Singleton<'a, U, W, C, N> {
        Singleton::new(
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
    ) -> Optional<'a, T, W, C, N> {
        Optional::new(
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
    ) -> Optional<'a, U, W, C, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::FilterMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, T, N: Location> Singleton<'a, T, Bounded, Tick, N> {
    pub fn cross_singleton<Other>(self, other: Other) -> <Self as CrossResult<'a, Other>>::Out
    where
        Self: CrossResult<'a, Other>,
    {
        if self.location_kind != Self::other_location(&other) {
            panic!("cross_singleton must be called on streams on the same node");
        }

        Self::make(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::CrossSingleton(
                Box::new(self.ir_node.into_inner()),
                Box::new(Self::other_ir_node(other)),
            ),
        )
    }

    pub fn continue_if<U>(
        self,
        signal: Optional<'a, U, Bounded, Tick, N>,
    ) -> Optional<'a, T, Bounded, Tick, N> {
        self.cross_singleton(signal.map(q!(|_u| ())))
            .map(q!(|(d, _signal)| d))
    }

    pub fn continue_unless<U>(
        self,
        other: Optional<'a, U, Bounded, Tick, N>,
    ) -> Optional<'a, T, Bounded, Tick, N> {
        self.continue_if(other.into_stream().count().filter(q!(|c| *c == 0)))
    }
}

impl<'a, T, N: Location> Singleton<'a, T, Bounded, Tick, N> {
    pub fn all_ticks(self) -> Stream<'a, T, Unbounded, NoTick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn latest(self) -> Optional<'a, T, Unbounded, NoTick, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Singleton<'a, T, Bounded, Tick, N> {
        Singleton::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<'a, T, Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn delta(self) -> Optional<'a, T, Bounded, Tick, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }
}

impl<'a, T, B, N: Location> Singleton<'a, T, B, NoTick, N> {
    pub fn latest_tick(self) -> Singleton<'a, T, Bounded, Tick, N> {
        Singleton::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }
}

impl<'a, T, N: Location> Singleton<'a, T, Unbounded, NoTick, N> {
    pub fn cross_singleton<Other>(self, other: Other) -> <Self as CrossResult<'a, Other>>::Out
    where
        Self: CrossResult<'a, Other>,
    {
        if self.location_kind != Self::other_location(&other) {
            panic!("cross_singleton must be called on streams on the same node");
        }

        Self::make(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(HfPlusNode::CrossSingleton(
                Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                Box::new(HfPlusNode::Unpersist(Box::new(Self::other_ir_node(other)))),
            ))),
        )
    }
}

pub struct Optional<'a, T, W, C, N: Location> {
    pub(crate) location_kind: LocationId,

    ir_leaves: FlowLeaves<'a>,
    pub(crate) ir_node: RefCell<HfPlusNode<'a>>,

    _phantom: PhantomData<(&'a mut &'a (), T, N, W, C)>,
}

impl<'a, T, W, C, N: Location> Optional<'a, T, W, C, N> {
    pub(crate) fn new(
        location_kind: LocationId,
        ir_leaves: FlowLeaves<'a>,
        ir_node: HfPlusNode<'a>,
    ) -> Self {
        Optional {
            location_kind,
            ir_leaves,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }

    pub fn some(singleton: Singleton<'a, T, W, C, N>) -> Self {
        Optional::new(
            singleton.location_kind,
            singleton.ir_leaves,
            singleton.ir_node.into_inner(),
        )
    }
}

impl<'a, T, W, N: Location> CycleComplete<'a> for Optional<'a, T, W, Tick, N> {
    fn complete(self, ident: syn::Ident) {
        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(self.ir_node.into_inner()),
        });
    }
}

impl<'a, T, W, N: Location> CycleCollection<'a> for Optional<'a, T, W, Tick, N> {
    type Location = N;

    fn create_source(ident: syn::Ident, ir_leaves: FlowLeaves<'a>, l: LocationId) -> Self {
        Optional::new(
            l,
            ir_leaves,
            HfPlusNode::CycleSource {
                ident,
                location_kind: l,
            },
        )
    }
}

impl<'a, T, W, N: Location> CycleComplete<'a> for Optional<'a, T, W, NoTick, N> {
    fn complete(self, ident: syn::Ident) {
        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
        });
    }
}

impl<'a, T, W, N: Location> CycleCollection<'a> for Optional<'a, T, W, NoTick, N> {
    type Location = N;

    fn create_source(ident: syn::Ident, ir_leaves: FlowLeaves<'a>, l: LocationId) -> Self {
        Optional::new(
            l,
            ir_leaves,
            HfPlusNode::Persist(Box::new(HfPlusNode::CycleSource {
                ident,
                location_kind: l,
            })),
        )
    }
}

impl<'a, T, W, C, N: Location> From<Singleton<'a, T, W, C, N>> for Optional<'a, T, W, C, N> {
    fn from(singleton: Singleton<'a, T, W, C, N>) -> Self {
        Optional::some(singleton)
    }
}

impl<'a, T: Clone, W, C, N: Location> Clone for Optional<'a, T, W, C, N> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: Rc::new(RefCell::new(orig_ir_node)),
            };
        }

        if let HfPlusNode::Tee { inner } = self.ir_node.borrow().deref() {
            Optional {
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

impl<'a, T, W, C, N: Location> Optional<'a, T, W, C, N> {
    pub fn map<U, F: Fn(T) -> U + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Optional<'a, U, W, C, N> {
        Optional::new(
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
    ) -> Optional<'a, T, W, C, N> {
        Optional::new(
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
    ) -> Optional<'a, U, W, C, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::FilterMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, T, N: Location> Optional<'a, T, Bounded, Tick, N> {
    // TODO(shadaj): this is technically incorrect; we should only return the first element of the stream
    pub fn into_stream(self) -> Stream<'a, T, Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            self.ir_node.into_inner(),
        )
    }

    pub fn cross_singleton<O>(
        self,
        other: impl Into<Optional<'a, O, Bounded, Tick, N>>,
    ) -> Optional<'a, (T, O), Bounded, Tick, N>
    where
        O: Clone,
    {
        let other: Optional<'a, O, Bounded, Tick, N> = other.into();
        if self.location_kind != other.location_kind {
            panic!("cross_singleton must be called on streams on the same node");
        }

        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::CrossSingleton(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn continue_if<U>(
        self,
        signal: Optional<'a, U, Bounded, Tick, N>,
    ) -> Optional<'a, T, Bounded, Tick, N> {
        self.cross_singleton(signal.map(q!(|_u| ())))
            .map(q!(|(d, _signal)| d))
    }

    pub fn continue_unless<U>(
        self,
        other: Optional<'a, U, Bounded, Tick, N>,
    ) -> Optional<'a, T, Bounded, Tick, N> {
        self.continue_if(other.into_stream().count().filter(q!(|c| *c == 0)))
    }

    pub fn union(
        self,
        other: Optional<'a, T, Bounded, Tick, N>,
    ) -> Optional<'a, T, Bounded, Tick, N> {
        if self.location_kind != other.location_kind {
            panic!("union must be called on streams on the same node");
        }

        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Union(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }
}

impl<'a, T, N: Location> Optional<'a, T, Bounded, Tick, N> {
    pub fn all_ticks(self) -> Stream<'a, T, Unbounded, NoTick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn latest(self) -> Optional<'a, T, Unbounded, NoTick, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Optional<'a, T, Bounded, Tick, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<'a, T, Bounded, Tick, N> {
        Stream::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn delta(self) -> Optional<'a, T, Bounded, Tick, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }
}

impl<'a, T, B, N: Location> Optional<'a, T, B, NoTick, N> {
    pub fn latest_tick(self) -> Optional<'a, T, Bounded, Tick, N> {
        Optional::new(
            self.location_kind,
            self.ir_leaves,
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn tick_samples(self) -> Stream<'a, T, Unbounded, NoTick, N> {
        self.latest_tick().all_ticks()
    }

    pub fn sample_every(
        self,
        duration: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<'a, T, Unbounded, NoTick, N> {
        let interval = duration.splice_typed();

        let samples = Stream::<'a, (), Bounded, Tick, N>::new(
            self.location_kind,
            self.ir_leaves.clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Interval(interval.into()),
                location_kind: self.location_kind,
            },
        );

        self.latest_tick()
            .continue_if(samples.first())
            .latest()
            .tick_samples()
    }
}

impl<'a, T, N: Location> Optional<'a, T, Unbounded, NoTick, N> {
    pub fn cross_singleton<O>(
        self,
        other: impl Into<Optional<'a, O, Unbounded, NoTick, N>>,
    ) -> Optional<'a, (T, O), Unbounded, NoTick, N>
    where
        O: Clone,
    {
        let other: Optional<'a, O, Unbounded, NoTick, N> = other.into();
        if self.location_kind != other.location_kind {
            panic!("cross_singleton must be called on streams on the same node");
        }

        self.latest_tick()
            .cross_singleton(other.latest_tick())
            .latest()
    }
}
