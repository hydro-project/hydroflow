use std::cell::RefCell;
use std::rc::Rc;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;

#[test]
#[allow(warnings)]
fn test_degenerate_merge() {
    let output = <Rc<RefCell<Vec<usize>>>>::default();
    let output_inner = Rc::clone(&output);

    let mut df: Hydroflow = hydroflow_syntax! {
        recv_iter([1, 2, 3]) -> merge() -> for_each(|x| output_inner.borrow_mut().push(x));
    };
    df.run_available();

    assert_eq!(&[1, 2, 3], &**output.borrow());
}

#[test]
#[allow(warnings)]
fn test_degenerate_tee() {
    let output = <Rc<RefCell<Vec<usize>>>>::default();
    let output_inner = Rc::clone(&output);

    let mut df: Hydroflow = hydroflow_syntax! {
        recv_iter([1, 2, 3]) -> tee() -> for_each(|x| output_inner.borrow_mut().push(x));
    };
    df.run_available();

    assert_eq!(&[1, 2, 3], &**output.borrow());
}
