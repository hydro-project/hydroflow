use hydroflow_plus::*;

struct P1 {}
struct P2 {}

fn test<'a, 'b>(p1: &Process<'a, P1>, p2: &Process<'b, P2>) {
    p1.source_iter(q!(0..10)).send_bincode(p2).for_each(q!(|n| println!("{}", n)));
}

fn main() {}
