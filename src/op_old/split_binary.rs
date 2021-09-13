use std::borrow::Cow;
use either::Either;
use ref_cast::RefCast;
use crate::func::SplitBinaryMorphism;
use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::{LatticeRepr, Convert};
use crate::lattice::pair::{PairRepr, PairEitherRepr};
use super::{Op, OpDelta, OpCumul};

pub struct SplitBinary<PrecOpA, PrecOpB, F>
where
    PrecOpA: OpDelta + OpCumul,
    PrecOpB: OpDelta + OpCumul,
    PrecOpA::LatReprDeltaOut: Convert<PrecOpA::LatReprCumulOut>,
    PrecOpB::LatReprDeltaOut: Convert<PrecOpB::LatReprCumulOut>,
    F: SplitBinaryMorphism<
        InLatReprA = PrecOpA::LatReprCumulOut,
        InLatReprB = PrecOpB::LatReprCumulOut,
    >,
{
    _phantom: std::marker::PhantomData<(PrecOpA, PrecOpB, F)>,
}

impl<PrecOpA, PrecOpB, F> Op for SplitBinary<PrecOpA, PrecOpB, F>
where
    PrecOpA: OpDelta + OpCumul,
    PrecOpB: OpDelta + OpCumul,
    PrecOpA::LatReprDeltaOut: Convert<PrecOpA::LatReprCumulOut>,
    PrecOpB::LatReprDeltaOut: Convert<PrecOpB::LatReprCumulOut>,
    F: SplitBinaryMorphism<
        InLatReprA = PrecOpA::LatReprCumulOut,
        InLatReprB = PrecOpB::LatReprCumulOut,
    >,
{
    type Lat = <F::OutLatRepr as LatticeRepr>::Lattice;

    type State = PairRepr<PrecOpA::State, PrecOpB::State>;
}

impl<PrecOpA, PrecOpB, F> OpDelta for SplitBinary<PrecOpA, PrecOpB, F>
where
    PrecOpA: OpDelta + OpCumul,
    PrecOpB: OpDelta + OpCumul,
    PrecOpA::LatReprDeltaOut: Convert<PrecOpA::LatReprCumulOut>,
    PrecOpB::LatReprDeltaOut: Convert<PrecOpB::LatReprCumulOut>,
    F: SplitBinaryMorphism<
        InLatReprA = PrecOpA::LatReprCumulOut,
        InLatReprB = PrecOpB::LatReprCumulOut,
    >,
{
    type LatReprDeltaIn  = PairEitherRepr<
        PrecOpA::LatReprDeltaIn,
        PrecOpB::LatReprDeltaIn,
    >;
    type LatReprDeltaOut = F::OutLatRepr;

    fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>)
        -> Cow<'h, Hide<Delta, Self::LatReprDeltaOut>>
    {
        let (state_a, state_b) = state.split_mut();

        match &*Hide::reveal_cow(element) {
            Either::Left(element_a) => {
                let element_a = PrecOpA::get_delta(state_a, Cow::Borrowed(Hide::ref_cast(element_a)));
                let element_a = <PrecOpA::LatReprDeltaOut as Convert<PrecOpA::LatReprCumulOut>>::convert_hide_cow(element_a);

                let element_a = Cow::<'h, Hide<Delta, PrecOpA::LatReprCumulOut>>::Owned(element_a);
                let element_b = Hide::as_delta_cow(PrecOpB::get_cumul(state_b));
                F::call(element_a, element_b)
            },
            Either::Right(element_b) => {
                let element_b = PrecOpB::get_delta(state_b, Cow::Borrowed(Hide::ref_cast(element_b)));
                let element_b = <PrecOpB::LatReprDeltaOut as Convert<PrecOpB::LatReprCumulOut>>::convert_hide_cow(element_b);

                let element_a = Hide::as_delta_cow(PrecOpA::get_cumul(state_a));
                let element_b = Cow::Owned(element_b);
                F::call(element_a, element_b)
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use ref_cast::RefCast;
    use crate::tag;
    use crate::hide::Qualifier;
    use crate::lattice::map_union::MapUnionRepr;
    use crate::lattice::set_union::SetUnionRepr;
    use crate::op::state_merge::StateMerge;
    use crate::op::identity::Identity;

    #[test]
    fn test_shj() {
        type ColPk = &'static str;
        type ColA = &'static str;
        type ColB = u32;
        type MyLatReprA = MapUnionRepr<tag::HASH_MAP, ColPk, SetUnionRepr<tag::HASH_SET, ColA>>;
        type MyLatReprB = MapUnionRepr<tag::HASH_MAP, ColPk, SetUnionRepr<tag::HASH_SET, ColB>>;

        type MyOutValLatRepr = SetUnionRepr<tag::HASH_SET, (ColA, ColB)>;
        type MyOutLatRepr = MapUnionRepr<tag::HASH_MAP, ColPk, MyOutValLatRepr>;
        
        pub enum Join {}
        impl SplitBinaryMorphism for Join {
            type InLatReprA = MyLatReprA;
            type InLatReprB = MyLatReprB;
            type OutLatRepr = MyOutLatRepr;
        
            fn call<'h, Y: Qualifier>(
                item_a: Cow<'h, Hide<Y, Self::InLatReprA>>,
                item_b: Cow<'h, Hide<Y, Self::InLatReprB>>,
            )
                -> Cow<'h, Hide<Y, Self::OutLatRepr>>
            {
                let item_a = item_a.reveal_ref();
                let item_b = item_b.reveal_ref();

                let mut out: <MyOutLatRepr as LatticeRepr>::Repr = Default::default();
                for (key, vals_a) in item_a.iter() {
                    if let Some(vals_b) = item_b.get(key) {
                        let mut vals: <MyOutValLatRepr as LatticeRepr>::Repr = Default::default();

                        for val_a in vals_a.iter() {
                            for val_b in vals_b.iter() {
                                vals.insert((*val_a, *val_b));
                            }
                        }

                        out.insert(key, vals);
                    }
                }
                Cow::Owned(Hide::new(out))
            }
        }

        type MyPipeline = StateMerge<SplitBinary<StateMerge<Identity<MyLatReprA>, MyLatReprA>, StateMerge<Identity<MyLatReprB>, MyLatReprB>, Join>, MyOutLatRepr>;

        type MyPipelineStateRepr = <<MyPipeline as Op>::State as LatticeRepr>::Repr;
        let mut state: MyPipelineStateRepr = ((((), Default::default()), ((), Default::default())), Default::default());

        let element_a_0 = {
            let mut map: <MyLatReprA as LatticeRepr>::Repr = Default::default();
            map.insert("Mingwei", vec!["Samuel"].into_iter().collect());
            map
        };
        let element_a_1 = {
            let mut map: <MyLatReprA as LatticeRepr>::Repr = Default::default();
            map.insert("Joseph", vec!["Hellerstein"].into_iter().collect());
            map
        };
        let element_a_2 = {
            let mut map: <MyLatReprA as LatticeRepr>::Repr = Default::default();
            map.insert("Mae", vec!["Milano"].into_iter().collect());
            map
        };
        let element_a_3 = {
            let mut map: <MyLatReprA as LatticeRepr>::Repr = Default::default();
            map.insert("Joseph", vec!["Gonzalez"].into_iter().collect());
            map
        };

        let element_b_0 = {
            let mut map: <MyLatReprB as LatticeRepr>::Repr = Default::default();
            map.insert("Mingwei", vec![ 2020 ].into_iter().collect());
            map
        };
        let element_b_1 = {
            let mut map: <MyLatReprB as LatticeRepr>::Repr = Default::default();
            map.insert("Joseph", vec![ 1990 ].into_iter().collect());
            map
        };
        let element_b_2 = {
            let mut map: <MyLatReprB as LatticeRepr>::Repr = Default::default();
            map.insert("Mae", vec![ 2013 ].into_iter().collect());
            map
        };
        let element_b_3 = {
            let mut map: <MyLatReprB as LatticeRepr>::Repr = Default::default();
            map.insert("Joseph", vec![ 2006 ].into_iter().collect());
            map
        };

        let out = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(Hide::new(Either::Left( element_a_0))));
        println!("{:?}", out.reveal_ref());
        let out = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(Hide::new(Either::Left( element_a_1))));
        println!("{:?}", out.reveal_ref());
        let out = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(Hide::new(Either::Right(element_b_0))));
        println!("{:?}", out.reveal_ref());
        let out = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(Hide::new(Either::Right(element_b_1))));
        println!("{:?}", out.reveal_ref());
        let out = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(Hide::new(Either::Right(element_b_2))));
        println!("{:?}", out.reveal_ref());
        let out = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(Hide::new(Either::Left( element_a_2))));
        println!("{:?}", out.reveal_ref());
        let out = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(Hide::new(Either::Left( element_a_3))));
        println!("{:?}", out.reveal_ref());
        let out = MyPipeline::get_delta(RefCast::ref_cast_mut(&mut state), Cow::Owned(Hide::new(Either::Right(element_b_3))));
        println!("{:?}", out.reveal_ref());

        let out = MyPipeline::get_cumul(RefCast::ref_cast_mut(&mut state));
        println!("{:?}", out.reveal_ref());
    }
}