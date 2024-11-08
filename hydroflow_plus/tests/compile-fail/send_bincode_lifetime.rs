use hydroflow_plus::*;

pub struct P1 {}
pub struct P2 {}

pub fn test<'a, 'b>(p1: &Process<'a, P1>, p2: &Process<'b, P2>) {
    p1.source_iter(q!(0..10)).send_bincode(p2).for_each(q!(|n| println!("{}", n)));
}

fn main() {}
