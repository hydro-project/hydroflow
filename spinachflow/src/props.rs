pub trait Bool {
    const VALUE: bool;
}
pub enum True {}
impl Bool for True {
    const VALUE: bool = true;
}
pub enum False {}
impl Bool for False {
    const VALUE: bool = false;
}

pub trait OrComp<Other> {
    type Output: Bool;
}
impl OrComp<False> for False {
    type Output = False;
}
impl OrComp<False> for True {
    type Output = True;
}
impl OrComp<True> for False {
    type Output = True;
}
impl OrComp<True> for True {
    type Output = True;
}
pub type Or<A, B> = <A as OrComp<B>>::Output;

pub trait OpProps {
    // type Complete: Bool;
    // type TimeOrd: Bool;
    // type LatOrd: Bool;
}
impl<Complete: Bool, TimeOrd: Bool, LatOrd: Bool> OpProps for (Complete, TimeOrd, LatOrd) {
    // type Complete = Complete;
    // type TimeOrd = TimeOrd;
    // type LatOrd = LatOrd;
}

#[allow(non_camel_case_types)]
pub trait OpProps_OrComp<Other: OpProps>
where
    Self: OpProps,
{
    type Output: OpProps;
}
impl<SelfComplete: Bool, SelfTimeOrd: Bool, SelfLatOrd: Bool, OtherComplete: Bool, OtherTimeOrd: Bool, OtherLatOrd: Bool>
    OpProps_OrComp<(OtherComplete, OtherTimeOrd, OtherLatOrd)> for (SelfComplete, SelfTimeOrd, SelfLatOrd)
where
    SelfComplete: OrComp<OtherComplete>,
    SelfTimeOrd: OrComp<OtherTimeOrd>,
    SelfLatOrd: OrComp<OtherLatOrd>,
{
    type Output = (Or<SelfComplete, OtherComplete>, Or<SelfTimeOrd, OtherTimeOrd>, Or<SelfLatOrd, OtherLatOrd>);
}
#[allow(non_camel_case_types)]
pub type OpProps_Or<A, B> = <A as OpProps_OrComp<B>>::Output;

// pub trait OpProps {
//     type Complete: Bool;
//     type TimeOrd: Bool;
//     type LatOrd: Bool;
// }

// pub enum OpProps_Complete0_TimeOrd0_LatOrd0 {}
// impl OpProps for OpProps_Complete0_TimeOrd0_LatOrd0 {
//     type Complete = False;
//     type TimeOrd = False;
//     type LatOrd = False;
// }
// pub enum OpProps_Complete0_TimeOrd0_LatOrd1 {}
// impl OpProps for OpProps_Complete0_TimeOrd0_LatOrd1 {
//     type Complete = False;
//     type TimeOrd = False;
//     type LatOrd = True;
// }
// pub enum OpProps_Complete0_TimeOrd1_LatOrd0 {}
// impl OpProps for OpProps_Complete0_TimeOrd1_LatOrd0 {
//     type Complete = False;
//     type TimeOrd = True;
//     type LatOrd = False;
// }
// pub enum OpProps_Complete0_TimeOrd1_LatOrd1 {}
// impl OpProps for OpProps_Complete0_TimeOrd1_LatOrd1 {
//     type Complete = False;
//     type TimeOrd = True;
//     type LatOrd = True;
// }
// pub enum OpProps_Complete1_TimeOrd0_LatOrd0 {}
// impl OpProps for OpProps_Complete1_TimeOrd0_LatOrd0 {
//     type Complete = True;
//     type TimeOrd = False;
//     type LatOrd = False;
// }
// pub enum OpProps_Complete1_TimeOrd0_LatOrd1 {}
// impl OpProps for OpProps_Complete1_TimeOrd0_LatOrd1 {
//     type Complete = True;
//     type TimeOrd = False;
//     type LatOrd = True;
// }
// pub enum OpProps_Complete1_TimeOrd1_LatOrd0 {}
// impl OpProps for OpProps_Complete1_TimeOrd1_LatOrd0 {
//     type Complete = True;
//     type TimeOrd = True;
//     type LatOrd = False;
// }
// pub enum OpProps_Complete1_TimeOrd1_LatOrd1 {}
// impl OpProps for OpProps_Complete1_TimeOrd1_LatOrd1 {
//     type Complete = True;
//     type TimeOrd = True;
//     type LatOrd = True;
// }

// pub trait OpProps_FromBools {
//     type Output: OpProps;
// }
// impl OpProps_FromBools for (False, False, False) {
//     type Output = 
// }

// pub trait OpProps_Or<A: OpProps>
// where
//     Self: OpProps,
// {
//     type Output;
// }
// impl OpProps_Or<>


// #[derive(Debug, Clone, Copy, Default)]
// #[derive(PartialOrd, Ord, PartialEq, Eq)]
// pub struct OpProps {
//     pub complete: bool,
//     pub time_ordered: bool,
//     pub lattice_ordered: bool, // E.g. cumulative.
// }

// impl OpProps {
//     pub const fn default() -> Self {
//         Self {
//             complete: false,
//             time_ordered: false,
//             lattice_ordered: false,
//         }
//     }
//     pub const fn combine(a: Self, b: Self) -> Self {
//         Self {
//             complete: a.complete || b.complete,
//             time_ordered: a.time_ordered || b.time_ordered,
//             lattice_ordered: a.lattice_ordered || b.lattice_ordered,
//         }
//     }
// }

// pub const fn require_lattice_ordered(complete: bool, time_ordered: bool) -> OpProps {
//     OpProps {
//         complete, time_ordered,
//         lattice_ordered: true,
//     }
// }

// // MACRO-GENERATED (TODO):
// #[macro_export]
// macro_rules! generate_ops_helper {
//     (
//         $name:ident,
//         $mapper:ident,
//         $( ($a:literal, $b:literal, $c:literal), )*
//     ) => {
//         $(
//             #[doc = "- COMPLETE: "]
//             #[doc = stringify!($a)]
//             #[doc = "\n- TIME_ORDERED: "]
//             #[doc = stringify!($b)]
//             #[doc = "\n- LATTICE_ORDERED: "]
//             #[doc = stringify!($c)]
//             #[allow(trivial_bounds)]
//             impl Op<{OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c }}> for $name
//             where
//                 Self: OpImpl<{OpProps::combine($mapper(OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c }), OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c })}>,
//             {
//                 fn get() {
//                     <$name as OpImpl<{OpProps::combine($mapper(OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c }), OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c })}>>::get()
//                 }
//             }
//         )*
//     };
// }

// #[macro_export]
// macro_rules! generate_ops {
//     ($name:ident, $mapper:ident) => {
//         generate_ops_helper!(
//             $name,
//             $mapper,
//             (false, false, false),
//             (false, false,  true),
//             (false,  true, false),
//             (false,  true,  true),
//             ( true, false, false),
//             ( true, false,  true),
//             ( true,  true, false),
//             ( true,  true,  true),
//         );
//     };
// }
// pub use generate_ops;
