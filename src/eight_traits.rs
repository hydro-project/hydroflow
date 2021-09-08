#[derive(Debug, Clone, Copy, Default)]
#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct OpProps {
    pub complete: bool,
    pub time_ordered: bool,
    pub lattice_ordered: bool,
}

impl OpProps {
    pub const fn default() -> Self {
        Self {
            complete: false,
            time_ordered: false,
            lattice_ordered: false,
        }
    }
    pub const fn combine(a: Self, b: Self) -> Self {
        Self {
            complete: a.complete || b.complete,
            time_ordered: a.time_ordered || b.time_ordered,
            lattice_ordered: a.lattice_ordered || b.lattice_ordered,
        }
    }
}

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
            impl Op<{OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c }}> for $name
            where
                Self: OpImpl<{OpProps::combine($mapper(OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c }), OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c })}>,
            {
                fn get() {
                    <$name as OpImpl<{OpProps::combine($mapper(OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c }), OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c })}>>::get()
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

pub trait OpImpl<const META: OpProps> {
    fn get() {}
}

pub trait Op<const META: OpProps> {
    fn get() {}
}




pub enum MySpookyOp {}

impl OpImpl<{OpProps { complete: true, ..OpProps::default() }}> for MySpookyOp {
    fn get() {}
}
impl OpImpl<{OpProps { time_ordered: true, ..OpProps::default() }}> for MySpookyOp {
    fn get() {}
}

const fn spooky_disambig(meta: OpProps) -> OpProps {
    match meta {
        OpProps { complete: false, time_ordered: false, lattice_ordered: false } => OpProps { complete: true, ..OpProps::default() },
        _ => OpProps::default(),
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