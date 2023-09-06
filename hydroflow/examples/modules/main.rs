use std::cell::RefCell;
use std::rc::Rc;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::multiset::HashMultiSet;

pub fn main() {
    let output = Rc::new(RefCell::new(
        HashMultiSet::<(usize, usize, usize)>::default(),
    ));

    let mut df: Hydroflow = {
        let output = output.clone();
        hydroflow_syntax! {
            source_iter(0..2) -> [0]cj;
            source_iter(0..2) -> [1]cj;
            source_iter(0..2) -> [2]cj;

            cj = import!("triple_cross_join.hf")
                -> for_each(|x| output.borrow_mut().insert(x));
        }
    };

    println!("{}", df.meta_graph().unwrap().to_mermaid());

    df.run_available();

    #[rustfmt::skip]
    assert_eq!(
        *output.borrow(),
        HashMultiSet::from_iter([
            (0, 0, 0),
            (0, 0, 1),
            (0, 1, 0),
            (0, 1, 1),
            (1, 0, 0),
            (1, 0, 1),
            (1, 1, 0),
            (1, 1, 1),
        ])
    );
}

#[test]
fn test() {
    main();
}
