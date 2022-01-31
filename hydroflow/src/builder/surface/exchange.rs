use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::Hasher,
};

use crate::{builder::HydroflowBuilder, scheduled::handoff::VecHandoff};

use super::{
    flatten::FlattenSurface, pull_chain::ChainPullSurface, pull_handoff::HandoffPullSurface,
    pull_iter::IterPullSurface, push_handoff::HandoffPushSurfaceReversed,
    push_start::StartPushSurface, BaseSurface, PullSurface, PushSurface,
};

// Pulled out to satisfy clippy for "complex types."
pub type ExchangeSurface<Key, Val, Other> =
    ChainPullSurface<FlattenSurface<HandoffPullSurface<VecHandoff<(Key, Val)>>>, Other>;

pub type NetworkOut<T> = HandoffPushSurfaceReversed<VecHandoff<(String, T)>, Option<(String, T)>>;

pub trait Exchange
where
    Self: PullSurface,
{
    fn exchange<Other, Key, Val>(
        self,
        builder: &mut HydroflowBuilder,
        address_book: HashMap<u64, String>,
        remote_input: Other,
        my_id: u64,
        outbound_messages: NetworkOut<<Self as BaseSurface>::ItemOut>,
    ) -> ExchangeSurface<Key, Val, Other>
    where
        Self: Sized,
        Key: Eq + std::hash::Hash + Clone,
        Val: Eq + Clone,
        Self: 'static + PullSurface<ItemOut = (Key, Val)>,
        Other: PullSurface<ItemOut = (Key, Val)>;
}

impl<T> Exchange for T
where
    T: PullSurface,
{
    fn exchange<Other, Key, Val>(
        self,
        builder: &mut HydroflowBuilder,
        address_book: HashMap<u64, String>,
        remote_input: Other,
        my_id: u64,
        outbound_messages: NetworkOut<<Self as BaseSurface>::ItemOut>,
    ) -> ExchangeSurface<Key, Val, Other>
    where
        Self: Sized,
        Key: Eq + std::hash::Hash + Clone,
        Val: Eq + Clone,
        Self: 'static + PullSurface<ItemOut = (Key, Val)>,
        Other: PullSurface<ItemOut = (Key, Val)>,
    {
        let (local_inputs_send, local_inputs_recv) =
            builder.make_handoff::<VecHandoff<(Key, Val)>, Option<(Key, Val)>>();

        let num_participants: u64 = address_book.len().try_into().unwrap();

        builder.add_subgraph(
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
                .pivot()
                .partition(
                    move |&(id, _, _)| id == my_id,
                    StartPushSurface::new()
                        .map(|(_, _, v)| Some(v))
                        .reverse(local_inputs_send),
                    StartPushSurface::new()
                        .map(|(_id, address, data)| Some((address, data)))
                        .reverse(outbound_messages),
                ),
        );

        local_inputs_recv.flatten().chain(remote_input)
    }
}
