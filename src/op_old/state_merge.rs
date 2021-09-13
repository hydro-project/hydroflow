use std::borrow::Cow;
use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::{LatticeRepr, Merge};
use crate::lattice::pair::{PairRepr};

use super::{Op, OpDelta, OpCumul};

pub struct StateMerge<PrecOp, Lr>
where
    PrecOp: OpDelta,
    Lr: LatticeRepr + Merge<PrecOp::LatReprDeltaOut>,
{
    _phantom: std::marker::PhantomData<(PrecOp, Lr)>,
}

impl<PrecOp, Lr> Op for StateMerge<PrecOp, Lr>
where
    PrecOp: OpDelta,
    Lr: LatticeRepr + Merge<PrecOp::LatReprDeltaOut>,
{
    type Lat = <PrecOp::LatReprDeltaOut as LatticeRepr>::Lattice;

    type State = PairRepr<PrecOp::State, Lr>;
}

impl<PrecOp, Lr> OpDelta for StateMerge<PrecOp, Lr>
where
    PrecOp: OpDelta,
    Lr: LatticeRepr + Merge<PrecOp::LatReprDeltaOut>,
{
    type LatReprDeltaIn = PrecOp::LatReprDeltaIn;
    type LatReprDeltaOut = PrecOp::LatReprDeltaOut;

    fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>)
        -> Cow<'h, Hide<Delta, Self::LatReprDeltaOut>>
    {
        let (prec_state, self_state) = state.split_mut();
        let element = PrecOp::get_delta(prec_state, element);
        Merge::merge_hide(self_state, element.clone().into_owned());
        return element;
    }
}

impl<PrecOp, Lr> OpCumul for StateMerge<PrecOp, Lr>
where
    PrecOp: OpDelta,
    Lr: LatticeRepr + Merge<PrecOp::LatReprDeltaOut>,
{
    type LatReprCumulOut = Lr;

    fn get_cumul<'h>(state: &'h mut Hide<Cumul, Self::State>)
        -> Cow<'h, Hide<Cumul, Self::LatReprCumulOut>>
    {
        let (_prec_state, self_state) = state.split_mut();
        return Cow::Borrowed(&*self_state);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use ref_cast::RefCast;
    use crate::lattice::ord::MaxRepr;
    use crate::op::identity::Identity;

    #[test]
    fn test_basic() {
        type MyLatRepr = MaxRepr<u32>;
        type MyPipeline = StateMerge<Identity<MyLatRepr>, MyLatRepr>;

        type MyStateRepr = <<MyPipeline as Op>::State as LatticeRepr>::Repr;
        let mut state: MyStateRepr = ((), 110);

        let element_100: Hide<Delta, MyLatRepr> = Hide::new(100);
        let element_120: Hide<Delta, MyLatRepr> = Hide::new(120);
        let element_150: Hide<Delta, MyLatRepr> = Hide::new(150);

        let output_delta = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(element_100));
        assert_eq!(100, output_delta.into_owned().into_reveal()); // TODO: Remove extra elements.
        let output_cumul = MyPipeline::get_cumul(RefCast::ref_cast_mut(&mut state));
        assert_eq!(110, output_cumul.into_owned().into_reveal());

        let output_delta = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(element_120));
        assert_eq!(120, output_delta.into_owned().into_reveal());
        let output_cumul = MyPipeline::get_cumul(RefCast::ref_cast_mut(&mut state));
        assert_eq!(120, output_cumul.into_owned().into_reveal());

        let output_delta = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(element_150));
        assert_eq!(150, output_delta.into_owned().into_reveal());
        let output_cumul = MyPipeline::get_cumul(RefCast::ref_cast_mut(&mut state));
        assert_eq!(150, output_cumul.into_owned().into_reveal());
    }

    #[test]
    fn test_merge_basic() {
        type MyLatRepr = MaxRepr<u32>;
        type MyPipeline = StateMerge<Identity<MyLatRepr>, MyLatRepr>;

        type MyStateLatRepr = <MyPipeline as Op>::State;
        type MyStateRepr = <MyStateLatRepr as LatticeRepr>::Repr;
        let mut state:   MyStateRepr = ((), 50);
        let state_merge: MyStateRepr = ((), 110);

        <MyStateLatRepr as Merge<MyStateLatRepr>>::merge(&mut state, state_merge);

        let element_100: Hide<Delta, MyLatRepr> = Hide::new(100);
        let element_120: Hide<Delta, MyLatRepr> = Hide::new(120);
        let element_150: Hide<Delta, MyLatRepr> = Hide::new(150);

        let output_delta = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(element_100));
        assert_eq!(100, output_delta.into_owned().into_reveal()); // TODO: Remove extra elements.
        let output_cumul = MyPipeline::get_cumul(RefCast::ref_cast_mut(&mut state));
        assert_eq!(110, output_cumul.into_owned().into_reveal());

        let output_delta = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(element_120));
        assert_eq!(120, output_delta.into_owned().into_reveal());
        let output_cumul = MyPipeline::get_cumul(RefCast::ref_cast_mut(&mut state));
        assert_eq!(120, output_cumul.into_owned().into_reveal());

        let output_delta = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(element_150));
        assert_eq!(150, output_delta.into_owned().into_reveal());
        let output_cumul = MyPipeline::get_cumul(RefCast::ref_cast_mut(&mut state));
        assert_eq!(150, output_cumul.into_owned().into_reveal());
    }

    #[test]
    fn test_nested() {
        type MyLatRepr = MaxRepr<u32>;
        type MyPipeline = StateMerge<Identity<MyLatRepr>, MyLatRepr>;

        type MyPipelineStateLatRepr = <MyPipeline as Op>::State;

        {
            type MetaPipeline = StateMerge<Identity<MyPipelineStateLatRepr>, MyPipelineStateLatRepr>;

            type MetaPipelineStateLatRepr = <MetaPipeline as Op>::State;
            type MetaPipelineStateRepr    = <MetaPipelineStateLatRepr as LatticeRepr>::Repr;

            let mut meta_state: MetaPipelineStateRepr = ((), ((), 020));

            let meta_element_050: Hide<Delta, MyPipelineStateLatRepr> = Hide::new(((), 050));
            let meta_element_110: Hide<Delta, MyPipelineStateLatRepr> = Hide::new(((), 110));

            {
                // Get the nested graph, get cumulative 20 out.
                let mut meta_output_cumul = MetaPipeline::get_cumul(RefCast::ref_cast_mut(&mut meta_state));
                let output_cumul = MyPipeline::get_cumul(meta_output_cumul.to_mut());
                assert_eq!(020, output_cumul.into_owned().into_reveal());
            }

            // Merge into the nested graph another graph containing 50, get cumulative state out.
            let meta_output_delta = MetaPipeline::get_delta(RefCast::ref_cast_mut(&mut meta_state), Cow::Owned(meta_element_050));
            assert_eq!(((), 050), meta_output_delta.into_owned().into_reveal());

            {
                // Get the nested graph, get cumulative 50 out.
                let mut meta_output_cumul = MetaPipeline::get_cumul(RefCast::ref_cast_mut(&mut meta_state));
                let output_cumul = MyPipeline::get_cumul(meta_output_cumul.to_mut());
                assert_eq!(050, output_cumul.into_owned().into_reveal());

                // Run the nested graph, merge 60 in.
                let element_060: Hide<Delta, MyLatRepr> = Hide::new(060);
                // Get 60 out.
                let output_delta = MyPipeline::get_delta(meta_output_cumul.to_mut(), Cow::Owned(element_060));
                assert_eq!(060, output_delta.into_owned().into_reveal());
                let output_cumul = MyPipeline::get_cumul(meta_output_cumul.to_mut());
                assert_eq!(060, output_cumul.into_owned().into_reveal());
            }

            // Merge into the nested graph another graph containing 110, get cumulative state out.
            let meta_output_delta = MetaPipeline::get_delta(RefCast::ref_cast_mut(&mut meta_state), Cow::Owned(meta_element_110));
            assert_eq!(((), 110), meta_output_delta.into_owned().into_reveal());

            {
                // Get the nested graph, get cumulative 110 out.
                let mut meta_output_cumul = MetaPipeline::get_cumul(RefCast::ref_cast_mut(&mut meta_state));
                let output_cumul = MyPipeline::get_cumul(meta_output_cumul.to_mut());
                assert_eq!(110, output_cumul.into_owned().into_reveal());
            }
        }
    }
}
