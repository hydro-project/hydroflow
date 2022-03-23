pub mod context;
pub mod graph;
pub mod graph_ext;
pub mod handoff;
pub mod input;
pub mod net;
pub mod port;
pub mod query;
pub mod reactor;
pub mod state;
pub(crate) mod subgraph;
pub mod type_list;
pub mod util;

/// A subgraph's ID. Invalid if used in a different [`graph::Hydroflow`]
/// instance than the original that created it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SubgraphId(pub(crate) usize);

/// A handoff's ID. Invalid if used in a different [`graph::Hydroflow`]
/// instance than the original that created it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HandoffId(pub(crate) usize);

/// A staten handle's ID. Invalid if used in a different [`graph::Hydroflow`]
/// instance than the original that created it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct StateId(pub(crate) usize);

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        builder::{
            prelude::{BaseSurface, PullSurface, PushSurface},
            HydroflowBuilder,
        },
        lang::{collections::Single, lattice::set_union::SetUnionRepr, tag},
        scheduled::handoff::VecHandoff,
    };

    #[test]
    fn test_batcher() {
        let outputs = Rc::new(RefCell::new(Vec::new()));
        let mut df = HydroflowBuilder::default();

        let (stream_input, stream_hoff) =
            df.add_channel_input::<_, Option<u64>, VecHandoff<_>>("stream input");
        let (ticks_input, ticks_hoff) =
            df.add_channel_input::<_, Option<u64>, VecHandoff<_>>("ticks input");

        let outputs_inner = outputs.clone();
        df.add_subgraph(
            "main",
            stream_hoff
                .flatten()
                .map(Single)
                .batch_with::<_, SetUnionRepr<tag::HASH_SET, u64>, SetUnionRepr<tag::SINGLE, u64>, _>(ticks_hoff.flatten())
                .map(|(x, v)| (x, v))
                .pull_to_push()
                .for_each(move |x| (*outputs_inner).borrow_mut().push(x)),
        );

        let mut df = df.build();

        stream_input.give(Some(1));
        stream_input.give(Some(2));
        stream_input.give(Some(3));
        stream_input.flush();
        ticks_input.give(Some(1));
        ticks_input.flush();

        df.tick();
        assert_eq!(vec![(1, [1, 2, 3].into())], *outputs.borrow());

        ticks_input.give(Some(2));
        ticks_input.flush();

        df.tick();
        assert_eq!(
            vec![(1, [1, 2, 3].into()), (2, [].into())],
            *outputs.borrow()
        );

        stream_input.give(Some(4));
        stream_input.give(Some(5));
        stream_input.flush();

        df.tick();
        assert_eq!(
            vec![(1, [1, 2, 3].into()), (2, [].into())],
            *outputs.borrow()
        );

        ticks_input.give(Some(3));
        ticks_input.flush();

        df.tick();
        assert_eq!(
            vec![(1, [1, 2, 3].into()), (2, [].into()), (3, [4, 5].into())],
            *outputs.borrow()
        );
    }
}
