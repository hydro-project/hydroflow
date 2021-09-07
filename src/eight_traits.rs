// pub trait IsComplete {}
// pub enum Complete {}
// impl IsComplete for Complete {}
// pub enum Incomplete {}
// impl IsComplete for Incomplete {}

// pub trait IsTimeOrdered {}
// pub enum TimeOrdered {}
// impl IsTimeOrdered for TimeOrdered {}
// pub enum NotTimeOrdered {}
// impl IsTimeOrdered for NotTimeOrdered {}

// pub trait IsLatticeOrdered {}
// pub enum LatticeOrdered {}
// impl IsLatticeOrdered for LatticeOrdered {}
// pub enum NotLatticeOrdered {}
// impl IsLatticeOrdered for NotLatticeOrdered {}


// MACRO-GENERATED (TODO):
macro_rules! generate_ops_helper {
    (
        $name:ident,
        $mapper:ident,
        $( ($a:literal, $b:literal, $c:literal), )*
    ) => {
        $(
            #[doc = "- COMPLETE: "]
            #[doc = stringify!($a)]
            #[doc = "\n- TIME_ORDERED: "]
            #[doc = stringify!($b)]
            #[doc = "\n- LATTICE_ORDERED: "]
            #[doc = stringify!($c)]
            #[allow(trivial_bounds)]
            impl Op<$a, $b, $c> for $name
            where
                Self: OpImpl<{$mapper($a, $b, $c).0 || $a}, {$mapper($a, $b, $c).1 || $b}, {$mapper($a, $b, $c).2 || $c}>,
            {
                fn get() {
                    <$name as OpImpl<{$mapper($a, $b, $c).0 || $a}, {$mapper($a, $b, $c).1 || $b}, {$mapper($a, $b, $c).2 || $c}>>::get()
                }
            }
        )*
    };
}

macro_rules! generate_ops {
    ($name:ident, $mapper:ident) => {
        generate_ops_helper!(
            $name,
            $mapper,
            (false, false, false),
            (false, false,  true),
            (false,  true, false),
            (false,  true,  true),
            ( true, false, false),
            ( true, false,  true),
            ( true,  true, false),
            ( true,  true,  true),
        );
    };
}

pub trait OpImpl<const COMPLETE: bool, const TIME_ORDERED: bool, const LATTICE_ORDERED: bool> {
    fn get() {}
}

pub trait Op<const COMPLETE: bool, const TIME_ORDERED: bool, const LATTICE_ORDERED: bool> {
    fn get() {}
}




pub enum MySpookyOp {}

impl OpImpl<true, false, false> for MySpookyOp {
    fn get() {}
}
impl OpImpl<false, true, false> for MySpookyOp {
    fn get() {}
}

const fn spooky_disambig(complete: bool, time_ordered: bool, lattice_ordered: bool) -> (bool, bool, bool) {
    match (complete, time_ordered, lattice_ordered) {
        (false, false, false) => ( true, false, false),
        _ => (false, false, false),
    }
}

generate_ops!(MySpookyOp, spooky_disambig);

// #[allow(trivial_bounds)]
// impl Op<false, false, false> for MySpookyOp
// where
//     Self: OpImpl<{mapper(false, false, false).0}, {mapper(false, false, false).1}, {mapper(false, false, false).2}>,
// {
//     fn get() {
//         <MySpookyOp as OpImpl<{mapper(false, false, false).0}, {mapper(false, false, false).1}, {mapper(false, false, false).2}>>::get()
//     }
// }

// // example invalid
// #[allow(trivial_bounds)]
// impl Op<true, true, false> for MySpookyOp
// where
//     Self: OpImpl<{mapper(true, true, false).0}, {mapper(true, true, false).1}, {mapper(true, true, false).2}>,
// {
//     fn get() {
//         <MySpookyOp as OpImpl<{mapper(true, true, false).0}, {mapper(true, true, false).1}, {mapper(true, true, false).2}>>::get()
//     }
// }

// #[allow(trivial_bounds)]
// impl Op<true, false, false> for MySpookyOp
// where
//     Self: OpImpl<{mapper(true, false, false).0}, {mapper(true, false, false).1}, {mapper(true, false, false).2}>,
// {
//     fn get() {
//         <MySpookyOp as OpImpl<{mapper(true, false, false).0}, {mapper(true, false, false).1}, {mapper(true, false, false).2}>>::get()
//     }
// }

// impl<const TIME_ORDERED: bool, const LATTICE_ORDERED: bool> Op<true, TIME_ORDERED, LATTICE_ORDERED> for MySpookyOp {
// }

// impl<const COMPLETE: bool, const LATTICE_ORDERED: bool> Op<COMPLETE, true, LATTICE_ORDERED> for MySpookyOp {
// }