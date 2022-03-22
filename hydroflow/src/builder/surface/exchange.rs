use std::{
    borrow::Cow,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::Hasher,
};

use crate::{builder::HydroflowBuilder, scheduled::handoff::VecHandoff};

use super::{
    flatten::FlattenSurface, pull_chain::ChainPullSurface, pull_handoff::HandoffPullSurface,
    pull_iter::IterPullSurface, push_handoff::HandoffPushSurfaceReversed,
    push_start::StartPushSurface, BaseSurface, PullSurface, PushSurface, TrackPullDependencies,
};

// Pulled out to satisfy clippy for "complex types."
pub type ExchangeSurface<Key, Val, Other> =
    ChainPullSurface<FlattenSurface<HandoffPullSurface<VecHandoff<(Key, Val)>>>, Other>;

pub type BroadcastSurface<T, Other> =
    ChainPullSurface<FlattenSurface<HandoffPullSurface<VecHandoff<T>>>, Other>;

pub type NetworkOut<T> = HandoffPushSurfaceReversed<VecHandoff<(String, T)>, Option<(String, T)>>;

pub trait Exchange
where
    Self: PullSurface,
{
    fn exchange<Name, Other, Key, Val>(
        self,
        builder: &mut HydroflowBuilder,
        name: Name,
        address_book: HashMap<u64, String>,
        remote_input: Other,
        my_id: u64,
        outbound_messages: NetworkOut<<Self as BaseSurface>::ItemOut>,
    ) -> ExchangeSurface<Key, Val, Other>
    where
        Name: Into<Cow<'static, str>>,
        Self: Sized,
        Key: Eq + std::hash::Hash + Clone,
        Val: Eq + Clone,
        Self: 'static + PullSurface<ItemOut = (Key, Val)>,
        Other: PullSurface<ItemOut = (Key, Val)>;

    fn broadcast<Name, Other, T>(
        self,
        builder: &mut HydroflowBuilder,
        name: Name,
        addresses: Vec<String>,
        remote_input: Other,
        outbound_messages: NetworkOut<<Self as BaseSurface>::ItemOut>,
    ) -> BroadcastSurface<T, Other>
    where
        Name: Into<Cow<'static, str>>,
        Self: Sized,
        T: Eq + Clone,
        Self: 'static + PullSurface<ItemOut = T>,
        Other: PullSurface<ItemOut = T>;
}

impl<T> Exchange for T
where
    T: PullSurface + TrackPullDependencies,
{
    fn exchange<Name, Other, Key, Val>(
        self,
        builder: &mut HydroflowBuilder,
        name: Name,
        address_book: HashMap<u64, String>,
        remote_input: Other,
        my_id: u64,
        outbound_messages: NetworkOut<<Self as BaseSurface>::ItemOut>,
    ) -> ExchangeSurface<Key, Val, Other>
    where
        Name: Into<Cow<'static, str>>,
        Self: Sized,
        Key: Eq + std::hash::Hash + Clone,
        Val: Eq + Clone,
        Self: 'static + PullSurface<ItemOut = (Key, Val)>,
        Other: PullSurface<ItemOut = (Key, Val)>,
    {
        let name = name.into();

        let (local_inputs_send, local_inputs_recv) = builder
            .make_edge::<_, VecHandoff<(Key, Val)>, Option<(Key, Val)>>(format!(
                "{} handoff",
                name
            ));

        let num_participants: u64 = address_book.len().try_into().unwrap();

        builder.add_subgraph(
            name,
            IterPullSurface::new(address_book.into_iter())
                .join(self.map(move |(x, v)| {
                    // TODO(justin): We should make our own thing here, I don't
                    // know if it's guaranteed this will be consistent across
                    // machines? And we might want to implement our own policy
                    // here.
                    let mut s = DefaultHasher::new();
                    x.hash(&mut s);
                    let hash_val = s.finish();
                    (hash_val % num_participants, (x, v))
                }))
                .pull_to_push()
                .partition_with_context(
                    move |_ctx, &(id, _, _)| id == my_id,
                    StartPushSurface::new()
                        .map(|(_, _, v)| Some(v))
                        .push_to(local_inputs_send),
                    StartPushSurface::new()
                        .map(|(_id, address, data)| Some((address, data)))
                        .push_to(outbound_messages),
                ),
        );

        local_inputs_recv.flatten().chain(remote_input)
    }

    fn broadcast<Name, Other, U>(
        self,
        builder: &mut HydroflowBuilder,
        name: Name,
        addresses: Vec<String>,
        remote_input: Other,
        outbound_messages: NetworkOut<<Self as BaseSurface>::ItemOut>,
    ) -> BroadcastSurface<U, Other>
    where
        Name: Into<Cow<'static, str>>,
        Self: Sized,
        U: Eq + Clone,
        Self: 'static + PullSurface<ItemOut = U>,
        Other: PullSurface<ItemOut = U>,
    {
        let name = name.into();

        let (local_inputs_send, local_inputs_recv) =
            builder.make_edge::<_, VecHandoff<U>, Option<U>>(format!("{} handoff", name));

        builder.add_subgraph(
            name,
            IterPullSurface::new(addresses.into_iter())
                .cross_join(self)
                .pull_to_push()
                .tee(
                    StartPushSurface::new()
                        .map(|(_, v)| Some(v))
                        .push_to(local_inputs_send),
                    StartPushSurface::new()
                        .map(|(address, data)| Some((address, data)))
                        .push_to(outbound_messages),
                ),
        );

        local_inputs_recv.flatten().chain(remote_input)
    }
}
