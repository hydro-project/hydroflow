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