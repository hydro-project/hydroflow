use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use stageleft::{q, IntoQuotedMut, Quoted};

use crate::builder::FlowState;
use crate::cycle::{
    CycleCollection, CycleCollectionWithInitial, CycleComplete, DeferTick, ForwardRef, TickCycle,
};
use crate::ir::{HfPlusLeaf, HfPlusNode, TeeNode};
use crate::location::{check_matching_location, Location, LocationId, NoTick, Tick};
use crate::stream::{Bounded, Unbounded};
use crate::Stream;

pub trait CrossResult<'a, Other> {
    type Out;
    type Location;

    fn other_location(other: &Other) -> Self::Location;
    fn other_ir_node(other: Other) -> HfPlusNode;

    fn make(location: Self::Location, ir_node: HfPlusNode) -> Self::Out;
}

impl<'a, T, U: Clone, W, N: Location<'a>> CrossResult<'a, Singleton<U, W, N>>
    for Singleton<T, W, N>
{
    type Out = Singleton<(T, U), W, N>;
    type Location = N;

    fn other_location(other: &Singleton<U, W, N>) -> N {
        other.location.clone()
    }

    fn other_ir_node(other: Singleton<U, W, N>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location: N, ir_node: HfPlusNode) -> Self::Out {
        Singleton::new(location, ir_node)
    }
}

impl<'a, T, U: Clone, W, N: Location<'a>> CrossResult<'a, Optional<U, W, N>>
    for Singleton<T, W, N>
{
    type Out = Optional<(T, U), W, N>;
    type Location = N;

    fn other_location(other: &Optional<U, W, N>) -> N {
        other.location.clone()
    }

    fn other_ir_node(other: Optional<U, W, N>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location: N, ir_node: HfPlusNode) -> Self::Out {
        Optional::new(location, ir_node)
    }
}

impl<'a, T, U: Clone, W, N: Location<'a>> CrossResult<'a, Optional<U, W, N>> for Optional<T, W, N> {
    type Out = Optional<(T, U), W, N>;
    type Location = N;

    fn other_location(other: &Optional<U, W, N>) -> N {
        other.location.clone()
    }

    fn other_ir_node(other: Optional<U, W, N>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location: N, ir_node: HfPlusNode) -> Self::Out {
        Optional::new(location, ir_node)
    }
}

impl<'a, T, U: Clone, W, N: Location<'a>> CrossResult<'a, Singleton<U, W, N>>
    for Optional<T, W, N>
{
    type Out = Optional<(T, U), W, N>;
    type Location = N;

    fn other_location(other: &Singleton<U, W, N>) -> N {
        other.location.clone()
    }

    fn other_ir_node(other: Singleton<U, W, N>) -> HfPlusNode {
        other.ir_node.into_inner()
    }

    fn make(location: N, ir_node: HfPlusNode) -> Self::Out {
        Optional::new(location, ir_node)
    }
}

pub struct Singleton<T, W, N> {
    location: N,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    _phantom: PhantomData<(T, N, W)>,
}

impl<'a, T, W, N: Location<'a>> Singleton<T, W, N> {
    pub(crate) fn new(location: N, ir_node: HfPlusNode) -> Self {
        Singleton {
            location,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }

    fn location_kind(&self) -> LocationId {
        self.location.id()
    }

    fn flow_state(&self) -> &FlowState {
        self.location.flow_state()
    }
}

impl<'a, T, N: Location<'a>> From<Singleton<T, Bounded, N>> for Singleton<T, Unbounded, N> {
    fn from(singleton: Singleton<T, Bounded, N>) -> Self {
        Singleton::new(singleton.location, singleton.ir_node.into_inner())
    }
}

impl<'a, T, N: Location<'a>> DeferTick for Singleton<T, Bounded, Tick<N>> {
    fn defer_tick(self) -> Self {
        Singleton::defer_tick(self)
    }
}

impl<'a, T, N: Location<'a>> CycleCollectionWithInitial<'a, TickCycle>
    for Singleton<T, Bounded, Tick<N>>
{
    type Location = Tick<N>;

    fn create_source(ident: syn::Ident, initial: Self, location: Tick<N>) -> Self {
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

impl<'a, T, N: Location<'a>> CycleComplete<'a, TickCycle> for Singleton<T, Bounded, Tick<N>> {
    fn complete(self, ident: syn::Ident) {
        self.flow_state().clone().borrow_mut().leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind(),
            input: Box::new(self.ir_node.into_inner()),
        });
    }
}

impl<'a, T, N: Location<'a>> CycleCollection<'a, ForwardRef> for Singleton<T, Bounded, Tick<N>> {
    type Location = Tick<N>;

    fn create_source(ident: syn::Ident, location: Tick<N>) -> Self {
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

impl<'a, T, N: Location<'a>> CycleComplete<'a, ForwardRef> for Singleton<T, Bounded, Tick<N>> {
    fn complete(self, ident: syn::Ident) {
        self.flow_state().clone().borrow_mut().leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind(),
            input: Box::new(self.ir_node.into_inner()),
        });
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

impl<'a, T, W, N: Location<'a>> Singleton<T, W, N> {
    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Singleton<U, W, N> {
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
    ) -> Stream<U, W, N> {
        Stream::new(
            self.location,
            HfPlusNode::FlatMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Optional<T, W, N> {
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
    ) -> Optional<U, W, N> {
        Optional::new(
            self.location,
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
        Stream::new(self.location, self.ir_node.into_inner())
    }

    pub fn cross_singleton<Other>(self, other: Other) -> <Self as CrossResult<'a, Other>>::Out
    where
        Self: CrossResult<'a, Other, Location = Tick<N>>,
    {
        check_matching_location(&self.location, &Self::other_location(&other));

        Self::make(
            self.location,
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
            self.location.outer().clone(),
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn latest(self) -> Singleton<T, Unbounded, N> {
        Singleton::new(
            self.location.outer().clone(),
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Singleton<T, Bounded, Tick<N>> {
        Singleton::new(
            self.location,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<T, Bounded, Tick<N>> {
        Stream::new(
            self.location,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn delta(self) -> Optional<T, Bounded, Tick<N>> {
        Optional::new(
            self.location,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }
}

impl<'a, T, B, N: Location<'a> + NoTick> Singleton<T, B, N> {
    pub fn latest_tick(self) -> Singleton<T, Bounded, Tick<N>> {
        Singleton::new(
            self.location.nest(),
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn sample_every(
        self,
        interval: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<T, Unbounded, N> {
        let samples = self.location.source_interval(interval).tick_batch();

        self.latest_tick()
            .continue_if(samples.first())
            .latest()
            .tick_samples()
    }
}

impl<'a, T, N: Location<'a> + NoTick> Singleton<T, Unbounded, N> {
    pub fn cross_singleton<Other>(self, other: Other) -> <Self as CrossResult<'a, Other>>::Out
    where
        Self: CrossResult<'a, Other, Location = N>,
    {
        check_matching_location(&self.location, &Self::other_location(&other));

        Self::make(
            self.location,
            HfPlusNode::Persist(Box::new(HfPlusNode::CrossSingleton(
                Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
                Box::new(HfPlusNode::Unpersist(Box::new(Self::other_ir_node(other)))),
            ))),
        )
    }
}

pub struct Optional<T, W, N> {
    pub(crate) location: N,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    _phantom: PhantomData<(T, N, W)>,
}

impl<'a, T, W, N: Location<'a>> Optional<T, W, N> {
    pub(crate) fn new(location: N, ir_node: HfPlusNode) -> Self {
        Optional {
            location,
            ir_node: RefCell::new(ir_node),
            _phantom: PhantomData,
        }
    }

    pub fn some(singleton: Singleton<T, W, N>) -> Self {
        Optional::new(singleton.location, singleton.ir_node.into_inner())
    }

    fn location_kind(&self) -> LocationId {
        self.location.id()
    }

    fn flow_state(&self) -> &FlowState {
        self.location.flow_state()
    }
}

impl<'a, T, N: Location<'a>> DeferTick for Optional<T, Bounded, Tick<N>> {
    fn defer_tick(self) -> Self {
        Optional::defer_tick(self)
    }
}

impl<'a, T, N: Location<'a>> CycleCollection<'a, TickCycle> for Optional<T, Bounded, Tick<N>> {
    type Location = Tick<N>;

    fn create_source(ident: syn::Ident, location: Tick<N>) -> Self {
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

impl<'a, T, N: Location<'a>> CycleComplete<'a, TickCycle> for Optional<T, Bounded, Tick<N>> {
    fn complete(self, ident: syn::Ident) {
        self.flow_state().clone().borrow_mut().leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind(),
            input: Box::new(self.ir_node.into_inner()),
        });
    }
}

impl<'a, T, N: Location<'a>> CycleCollection<'a, ForwardRef> for Optional<T, Bounded, Tick<N>> {
    type Location = Tick<N>;

    fn create_source(ident: syn::Ident, location: Tick<N>) -> Self {
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

impl<'a, T, N: Location<'a>> CycleComplete<'a, ForwardRef> for Optional<T, Bounded, Tick<N>> {
    fn complete(self, ident: syn::Ident) {
        self.flow_state().clone().borrow_mut().leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind(),
            input: Box::new(self.ir_node.into_inner()),
        });
    }
}

impl<'a, T, W, N: Location<'a> + NoTick> CycleCollection<'a, ForwardRef> for Optional<T, W, N> {
    type Location = N;

    fn create_source(ident: syn::Ident, location: N) -> Self {
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

impl<'a, T, W, N: Location<'a> + NoTick> CycleComplete<'a, ForwardRef> for Optional<T, W, N> {
    fn complete(self, ident: syn::Ident) {
        self.flow_state().clone().borrow_mut().leaves.as_mut().expect("Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind(),
            input: Box::new(HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner()))),
        });
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

impl<'a, T, W, N: Location<'a>> Optional<T, W, N> {
    // TODO(shadaj): this is technically incorrect; we should only return the first element of the stream
    pub fn into_stream(self) -> Stream<T, W, N> {
        if N::is_top_level() {
            panic!("Converting an optional to a stream is not yet supported at the top level");
        }

        Stream::new(self.location, self.ir_node.into_inner())
    }

    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Optional<U, W, N> {
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
    ) -> Stream<U, W, N> {
        Stream::new(
            self.location,
            HfPlusNode::FlatMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Optional<T, W, N> {
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
    ) -> Optional<U, W, N> {
        Optional::new(
            self.location,
            HfPlusNode::FilterMap {
                f: f.splice_fn1().into(),
                input: Box::new(self.ir_node.into_inner()),
            },
        )
    }
}

impl<'a, T, N: Location<'a>> Optional<T, Bounded, Tick<N>> {
    pub fn cross_singleton<O>(
        self,
        other: impl Into<Optional<O, Bounded, Tick<N>>>,
    ) -> Optional<(T, O), Bounded, Tick<N>>
    where
        O: Clone,
    {
        let other: Optional<O, Bounded, Tick<N>> = other.into();
        check_matching_location(&self.location, &other.location);

        Optional::new(
            self.location,
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
        check_matching_location(&self.location, &other.location);

        Optional::new(
            self.location,
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
        check_matching_location(&self.location, &other.location);

        Singleton::new(
            self.location,
            HfPlusNode::Union(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            ),
        )
    }

    pub fn into_singleton(self) -> Singleton<Option<T>, Bounded, Tick<N>>
    where
        T: Clone,
        N: NoTick,
    {
        let none_singleton = self.location.outer().singleton_each_tick(q!(None));
        self.map(q!(|v| Some(v))).unwrap_or(none_singleton)
    }
}

impl<'a, T, N: Location<'a>> Optional<T, Bounded, Tick<N>> {
    pub fn all_ticks(self) -> Stream<T, Unbounded, N> {
        Stream::new(
            self.location.outer().clone(),
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn latest(self) -> Optional<T, Unbounded, N> {
        Optional::new(
            self.location.outer().clone(),
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn defer_tick(self) -> Optional<T, Bounded, Tick<N>> {
        Optional::new(
            self.location,
            HfPlusNode::DeferTick(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn persist(self) -> Stream<T, Bounded, Tick<N>> {
        Stream::new(
            self.location,
            HfPlusNode::Persist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn delta(self) -> Optional<T, Bounded, Tick<N>> {
        Optional::new(
            self.location,
            HfPlusNode::Delta(Box::new(self.ir_node.into_inner())),
        )
    }
}

impl<'a, T, B, N: Location<'a> + NoTick> Optional<T, B, N> {
    pub fn latest_tick(self) -> Optional<T, Bounded, Tick<N>> {
        Optional::new(
            self.location.nest(),
            HfPlusNode::Unpersist(Box::new(self.ir_node.into_inner())),
        )
    }

    pub fn tick_samples(self) -> Stream<T, Unbounded, N> {
        self.latest_tick().all_ticks()
    }

    pub fn sample_every(
        self,
        interval: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<T, Unbounded, N> {
        let samples = self.location.source_interval(interval).tick_batch();

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
        check_matching_location(&self.location, &other.location);

        self.latest_tick().unwrap_or(other.latest_tick()).latest()
    }

    pub fn into_singleton(self) -> Singleton<Option<T>, Unbounded, N>
    where
        T: Clone,
    {
        let none_singleton = self.location.singleton(q!(None));
        self.map(q!(|v| Some(v))).unwrap_or(none_singleton)
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
        check_matching_location(&self.location, &other.location);

        self.latest_tick()
            .cross_singleton(other.latest_tick())
            .latest()
    }
}
