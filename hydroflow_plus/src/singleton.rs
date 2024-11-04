use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use stageleft::{q, IntoQuotedMut, Quoted};

use crate::builder::FlowState;
use crate::cycle::{
    CycleCollection, CycleCollectionWithInitial, CycleComplete, DeferTick, TickCycle,
};
use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource, TeeNode};
use crate::location::{Location, LocationId};
use crate::stream::{Bounded, NoTick, Tick, Unbounded};
use crate::Stream;

pub trait CrossResult<'a, Other> {
    type Out;
    fn other_location(other: &Other) -> LocationId;
    fn other_ir_node(other: Other) -> HfPlusNode;

    fn make(location_kind: LocationId, flow_state: FlowState, ir_node: HfPlusNode) -> Self::Out;
}

impl<'a, T, U: Clone, W, N: Location<'a>> CrossResult<'a, Singleton<U, W, N>>
    for Singleton<T, W, N>
{
    type Out = Singleton<(T, U), W, N>;

    fn other_location(other: &Singleton<U, W, N>) -> LocationId {
        other.location_kind
    }

    fn other_ir_node(other: Singleton<U, W, N>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location_kind: LocationId, flow_state: FlowState, ir_node: HfPlusNode) -> Self::Out {
        Singleton::new(location_kind, flow_state, ir_node)
    }
}

impl<'a, T, U: Clone, W, N: Location<'a>> CrossResult<'a, Optional<U, W, N>>
    for Singleton<T, W, N>
{
    type Out = Optional<(T, U), W, N>;

    fn other_location(other: &Optional<U, W, N>) -> LocationId {
        other.location_kind
    }

    fn other_ir_node(other: Optional<U, W, N>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location_kind: LocationId, flow_state: FlowState, ir_node: HfPlusNode) -> Self::Out {
        Optional::new(location_kind, flow_state, ir_node)
    }
}

impl<'a, T, U: Clone, W, N: Location<'a>> CrossResult<'a, Optional<U, W, N>> for Optional<T, W, N> {
    type Out = Optional<(T, U), W, N>;

    fn other_location(other: &Optional<U, W, N>) -> LocationId {
        other.location_kind
    }

    fn other_ir_node(other: Optional<U, W, N>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location_kind: LocationId, flow_state: FlowState, ir_node: HfPlusNode) -> Self::Out {
        Optional::new(location_kind, flow_state, ir_node)
    }
}

impl<'a, T, U: Clone, W, N: Location<'a>> CrossResult<'a, Singleton<U, W, N>>
    for Optional<T, W, N>
{
    type Out = Optional<(T, U), W, N>;

    fn other_location(other: &Singleton<U, W, N>) -> LocationId {
        other.location_kind
    }

    fn other_ir_node(other: Singleton<U, W, N>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location_kind: LocationId, flow_state: FlowState, ir_node: HfPlusNode) -> Self::Out {
        Optional::new(location_kind, flow_state, ir_node)
    }
}

pub struct Singleton<T, W, N> {
    pub(crate) location_kind: LocationId,

    flow_state: FlowState,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    _phantom: PhantomData<(T, N, W)>,
}

impl<'a, T, W, N: Location<'a>> Singleton<T, W, N> {
    pub(crate) fn new(
        location_kind: LocationId,
        flow_state: FlowState,
        ir_node: HfPlusNode,
    ) -> Self {
        Singleton {
            location_kind,
            flow_state,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, N: Location<'a>> From<Singleton<T, Bounded, N>> for Singleton<T, Unbounded, N> {
    fn from(singleton: Singleton<T, Bounded, N>) -> Self {
        Singleton::new(
            singleton.location_kind,
            singleton.flow_state,
            singleton.ir_node.into_inner(),
        )
    }
}

impl<'a, T, N: Location<'a>> DeferTick for Singleton<T, Bounded, Tick<N>> {
    fn defer_tick(self) -> Self {
        Singleton::defer_tick(self)
    }
}

impl<'a, T, N: Location<'a>> CycleComplete<'a, TickCycle> for Singleton<T, Bounded, Tick<N>> {
    fn complete(self, ident: syn::Ident) {
        self.flow_state.borrow_mut().leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(self.ir_node.into_inner()),
        });
    }
}

impl<'a, T, N: Location<'a>> CycleCollection<'a, TickCycle> for Singleton<T, Bounded, Tick<N>> {
    type Location = N;

    fn create_source(ident: syn::Ident, flow_state: FlowState, l: LocationId) -> Self {
        Singleton::new(
            l,
            flow_state,
            HfPlusNode::CycleSource {
                ident,
                location_kind: l,
            },
        )
    }
}

impl<'a, T, N: Location<'a>> CycleCollectionWithInitial<'a, TickCycle>
    for Singleton<T, Bounded, Tick<N>>
{
    type Location = N;

    fn create_source(
        ident: syn::Ident,
        flow_state: FlowState,
        initial: Self,
        l: LocationId,
    ) -> Self {
        Singleton::new(
            l,
            flow_state,
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

impl<'a, T: Clone, W, N: Location<'a>> Clone for Singleton<T, W, N> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: TeeNode(Rc::new(RefCell::new(orig_ir_node))),
            };
        }

        if let HfPlusNode::Tee { inner } = self.ir_node.borrow().deref() {
            Singleton {
                location_kind: self.location_kind,
                flow_state: self.flow_state.clone(),
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

impl<'a, T, W, N: Location<'a>> Singleton<T, W, N> {
    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Singleton<U, W, N> {
        Singleton::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Map {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn flat_map<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<U, W, N> {
        Stream::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::FlatMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Optional<T, W, N> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Filter {
                f: f.splice_fn1_borrow().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Optional<U, W, N> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::FilterMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, T, N: Location<'a>> Singleton<T, Bounded, Tick<N>> {
    // TODO(shadaj): this is technically incorrect; we should only return the first element of the stream
    pub fn into_stream(self) -> Stream<T, Bounded, Tick<N>> {
        Stream::new(
            self.location_kind,
            self.flow_state,
            self.ir_node.into_inner(),
        )
    }

    pub fn cross_singleton<Other>(self, other: Other) -> <Self as CrossResult<'a, Other>>::Out
    where
        Self: CrossResult<'a, Other>,
    {
        if self.location_kind != Self::other_location(&other) {
            panic!("cross_singleton must be called on streams on the same node");
        }

        Self::make(
            self.location_kind,
            self.flow_state,
            HfPlusNode::CrossSingleton(
                Box::new(self.ir_node.into_inner()),
                Box::new(Self::other_ir_node(other)),
            ),
        )
    }

    pub fn continue_if<U>(
        self,
        signal: Optional<U, Bounded, Tick<N>>,
    ) -> Optional<T, Bounded, Tick<N>> {
        self.cross_singleton(signal.map(q!(|_u| ())))
            .map(q!(|(d, _signal)| d))
    }

    pub fn continue_unless<U>(
        self,
        other: Optional<U, Bounded, Tick<N>>,
    ) -> Optional<T, Bounded, Tick<N>> {
        self.continue_if(other.into_stream().count().filter(q!(|c| *c == 0)))
    }
}

impl<'a, T, N: Location<'a>> Singleton<T, Bounded, Tick<N>> {
    pub fn all_ticks(self) -> Stream<T, Unbounded, N> {
        Stream::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn latest(self) -> Singleton<T, Unbounded, N> {
        Singleton::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Singleton<T, Bounded, Tick<N>> {
        Singleton::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<T, Bounded, Tick<N>> {
        Stream::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn delta(self) -> Optional<T, Bounded, Tick<N>> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }
}

impl<'a, T, B, N: Location<'a> + NoTick> Singleton<T, B, N> {
    pub fn latest_tick(self) -> Singleton<T, Bounded, Tick<N>> {
        Singleton::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn sample_every(
        self,
        duration: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<T, Unbounded, N> {
        let interval = duration.splice_typed();

        let samples = Stream::<(), Bounded, Tick<N>>::new(
            self.location_kind,
            self.flow_state.clone(),
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

impl<'a, T, N: Location<'a> + NoTick> Singleton<T, Unbounded, N> {
    pub fn cross_singleton<Other>(self, other: Other) -> <Self as CrossResult<'a, Other>>::Out
    where
        Self: CrossResult<'a, Other>,
    {
        if self.location_kind != Self::other_location(&other) {
            panic!("cross_singleton must be called on streams on the same node");
        }

        Self::make(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Persist(Box::new(HfPlusNode::CrossSingleton(
                Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                Box::new(HfPlusNode::Unpersist(Box::new(Self::other_ir_node(other)))),
            ))),
        )
    }
}

pub struct Optional<T, W, N> {
    pub(crate) location_kind: LocationId,

    flow_state: FlowState,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    _phantom: PhantomData<(T, N, W)>,
}

impl<'a, T, W, N: Location<'a>> Optional<T, W, N> {
    pub(crate) fn new(
        location_kind: LocationId,
        flow_state: FlowState,
        ir_node: HfPlusNode,
    ) -> Self {
        Optional {
            location_kind,
            flow_state,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }

    pub fn some(singleton: Singleton<T, W, N>) -> Self {
        Optional::new(
            singleton.location_kind,
            singleton.flow_state,
            singleton.ir_node.into_inner(),
        )
    }
}

impl<'a, T, N: Location<'a>> DeferTick for Optional<T, Bounded, Tick<N>> {
    fn defer_tick(self) -> Self {
        Optional::defer_tick(self)
    }
}

impl<'a, T, N: Location<'a>> CycleComplete<'a, TickCycle> for Optional<T, Bounded, Tick<N>> {
    fn complete(self, ident: syn::Ident) {
        self.flow_state.borrow_mut().leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(self.ir_node.into_inner()),
        });
    }
}

impl<'a, T, N: Location<'a>> CycleCollection<'a, TickCycle> for Optional<T, Bounded, Tick<N>> {
    type Location = N;

    fn create_source(ident: syn::Ident, flow_state: FlowState, l: LocationId) -> Self {
        Optional::new(
            l,
            flow_state,
            HfPlusNode::CycleSource {
                ident,
                location_kind: l,
            },
        )
    }
}

impl<'a, T, W, N: Location<'a> + NoTick> CycleComplete<'a, ()> for Optional<T, W, N> {
    fn complete(self, ident: syn::Ident) {
        self.flow_state.borrow_mut().leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
        });
    }
}

impl<'a, T, W, N: Location<'a> + NoTick> CycleCollection<'a, ()> for Optional<T, W, N> {
    type Location = N;

    fn create_source(ident: syn::Ident, flow_state: FlowState, l: LocationId) -> Self {
        Optional::new(
            l,
            flow_state,
            HfPlusNode::Persist(Box::new(HfPlusNode::CycleSource {
                ident,
                location_kind: l,
            })),
        )
    }
}

impl<'a, T, W, N: Location<'a>> From<Singleton<T, W, N>> for Optional<T, W, N> {
    fn from(singleton: Singleton<T, W, N>) -> Self {
        Optional::some(singleton)
    }
}

impl<'a, T: Clone, W, N: Location<'a>> Clone for Optional<T, W, N> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Placeholder);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: TeeNode(Rc::new(RefCell::new(orig_ir_node))),
            };
        }

        if let HfPlusNode::Tee { inner } = self.ir_node.borrow().deref() {
            Optional {
                location_kind: self.location_kind,
                flow_state: self.flow_state.clone(),
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

impl<'a, T, W, N: Location<'a>> Optional<T, W, N> {
    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Optional<U, W, N> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Map {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn flat_map<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<U, W, N> {
        Stream::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::FlatMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Optional<T, W, N> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Filter {
                f: f.splice_fn1_borrow().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Optional<U, W, N> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::FilterMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, T, N: Location<'a>> Optional<T, Bounded, Tick<N>> {
    // TODO(shadaj): this is technically incorrect; we should only return the first element of the stream
    pub fn into_stream(self) -> Stream<T, Bounded, Tick<N>> {
        Stream::new(
            self.location_kind,
            self.flow_state,
            self.ir_node.into_inner(),
        )
    }

    pub fn cross_singleton<O>(
        self,
        other: impl Into<Optional<O, Bounded, Tick<N>>>,
    ) -> Optional<(T, O), Bounded, Tick<N>>
    where
        O: Clone,
    {
        let other: Optional<O, Bounded, Tick<N>> = other.into();
        if self.location_kind != other.location_kind {
            panic!("cross_singleton must be called on streams on the same node");
        }

        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::CrossSingleton(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn continue_if<U>(
        self,
        signal: Optional<U, Bounded, Tick<N>>,
    ) -> Optional<T, Bounded, Tick<N>> {
        self.cross_singleton(signal.map(q!(|_u| ())))
            .map(q!(|(d, _signal)| d))
    }

    pub fn then<U>(self, value: Singleton<U, Bounded, Tick<N>>) -> Optional<U, Bounded, Tick<N>> {
        value.continue_if(self)
    }

    pub fn continue_unless<U>(
        self,
        other: Optional<U, Bounded, Tick<N>>,
    ) -> Optional<T, Bounded, Tick<N>> {
        self.continue_if(other.into_stream().count().filter(q!(|c| *c == 0)))
    }

    pub fn union(self, other: Optional<T, Bounded, Tick<N>>) -> Optional<T, Bounded, Tick<N>> {
        if self.location_kind != other.location_kind {
            panic!("union must be called on streams on the same node");
        }

        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Union(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn unwrap_or(
        self,
        other: Singleton<T, Bounded, Tick<N>>,
    ) -> Singleton<T, Bounded, Tick<N>> {
        if self.location_kind != other.location_kind {
            panic!("or_else must be called on streams on the same node");
        }

        Singleton::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Union(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }
}

impl<'a, T, N: Location<'a>> Optional<T, Bounded, Tick<N>> {
    pub fn all_ticks(self) -> Stream<T, Unbounded, N> {
        Stream::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn latest(self) -> Optional<T, Unbounded, N> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Optional<T, Bounded, Tick<N>> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<T, Bounded, Tick<N>> {
        Stream::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn delta(self) -> Optional<T, Bounded, Tick<N>> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }
}

impl<'a, T, B, N: Location<'a> + NoTick> Optional<T, B, N> {
    pub fn latest_tick(self) -> Optional<T, Bounded, Tick<N>> {
        Optional::new(
            self.location_kind,
            self.flow_state,
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn tick_samples(self) -> Stream<T, Unbounded, N> {
        self.latest_tick().all_ticks()
    }

    pub fn sample_every(
        self,
        duration: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<T, Unbounded, N> {
        let interval = duration.splice_typed();

        let samples = Stream::<(), Bounded, Tick<N>>::new(
            self.location_kind,
            self.flow_state.clone(),
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

    pub fn unwrap_or(
        self,
        other: impl Into<Singleton<T, Unbounded, N>>,
    ) -> Singleton<T, Unbounded, N> {
        let other = other.into();
        if self.location_kind != other.location_kind {
            panic!("or_else must be called on streams on the same node");
        }

        self.latest_tick().unwrap_or(other.latest_tick()).latest()
    }
}

impl<'a, T, N: Location<'a> + NoTick> Optional<T, Unbounded, N> {
    pub fn cross_singleton<O>(
        self,
        other: impl Into<Optional<O, Unbounded, N>>,
    ) -> Optional<(T, O), Unbounded, N>
    where
        O: Clone,
    {
        let other: Optional<O, Unbounded, N> = other.into();
        if self.location_kind != other.location_kind {
            panic!("cross_singleton must be called on streams on the same node");
        }

        self.latest_tick()
            .cross_singleton(other.latest_tick())
            .latest()
    }
}
