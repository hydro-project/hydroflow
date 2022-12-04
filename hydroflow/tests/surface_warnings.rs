//! `#[allow(warnings)]` doesn't actually do anything with proc-macro warnings.

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
fn test_empty_merge() {
    let mut df: Hydroflow = hydroflow_syntax! {
        merge() -> for_each(|x: usize| println!("{}", x));
    };
    df.run_available();
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

#[test]
#[allow(warnings)]
fn test_empty_tee() {
    let output = <Rc<RefCell<Vec<usize>>>>::default();
    let output_inner = Rc::clone(&output);

    let mut df: Hydroflow = hydroflow_syntax! {
        recv_iter([1, 2, 3]) -> map(|x| { output_inner.borrow_mut().push(x); x }) -> tee();
    };
    df.run_available();

    assert_eq!(&[1, 2, 3], &**output.borrow());
}

// Mainly checking subgraph partitioning pull-push handling.
// But in a different edge order.
#[test]
#[allow(warnings)]
pub fn test_warped_diamond() {
    let mut df: Hydroflow = hydroflow_syntax! {
        // active nodes
        nodes = merge();

        // stream of nodes into the system
        init = join() -> for_each(|(n, (a, b))| {
            println!("DEBUG ({:?}, ({:?}, {:?}))", n, a, b);
        });
        new_node = recv_iter([1, 2, 3]) -> tee();
        // add self
        new_node[0] -> map(|n| (n, 'a')) -> [0]nodes;
        // join peers against active nodes
        nodes -> [0]init;
        new_node[1] -> map(|n| (n, 'b')) -> [1]init;
    };
    df.run_available();
}

#[test]
#[allow(warnings)]
pub fn test_warped_diamond_2() {
    let mut df: Hydroflow = hydroflow_syntax! {
        // active nodes
        nodes = merge();

        // stream of nodes into the system
        init = join() -> for_each(|(n, (a, b))| {
            println!("DEBUG ({:?}, ({:?}, {:?}))", n, a, b);
        });
        new_node = recv_iter([1, 2, 3]) -> tee();
        // add self
        new_node[0] -> map(|n| (n, 'a')) -> [0]nodes;
        // join peers against active nodes
        nodes -> [0]init;
        new_node[1] -> map(|n| (n, 'b')) -> [1]init;

        ntwk = recv_iter([4, 5, 6]) -> tee();
    };
    df.run_available();
}
