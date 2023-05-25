use std::cell::RefCell;
use std::rc::Rc;

use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::collect_ready;

macro_rules! test_warnings {
    (
        $hf:tt,
        $( $msg:literal ),*
        $( , )?
    ) => {
        {
            let __file = std::file!();
            let __line = std::line!() as usize;
            let __hf = hydroflow::hydroflow_syntax_noemit! $hf;

            let diagnostics = __hf.diagnostics().expect("Expected `diagnostics()` to be set.");
            let expecteds = &[
                $( $msg , )*
            ];
            assert_eq!(diagnostics.len(), expecteds.len(), "Wrong number of diagnostics.");
            for (expected, diagnostic) in expecteds.iter().zip(diagnostics.iter()) {
                let mut diagnostic = diagnostic.clone();
                diagnostic.span.line -= __line;
                assert_eq!(expected.to_string(), diagnostic.to_string().replace(__file, "$FILE"));
            }

            __hf
        }
    };
}

#[test]
fn test_degenerate_union() {
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = test_warnings! {
        {
            source_iter([1, 2, 3]) -> union() -> for_each(|x| result_send.send(x).unwrap());
        },
        "Warning: `union` should have at least 2 input(s), actually has 1.\n  --> $FILE:2:39",
    };
    df.run_available();

    assert_eq!(&[1, 2, 3], &*collect_ready::<Vec<_>, _>(&mut result_recv));
}

#[test]
fn test_empty_union() {
    let mut df = test_warnings! {
        {
            union() -> for_each(|x: usize| println!("{}", x));
        },
        "Warning: `union` should have at least 2 input(s), actually has 0.\n  --> $FILE:2:13",
    };
    df.run_available();
}

#[test]
fn test_degenerate_tee() {
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = test_warnings! {
        {
            source_iter([1, 2, 3]) -> tee() -> for_each(|x| result_send.send(x).unwrap());
        },
        "Warning: `tee` should have at least 2 output(s), actually has 1.\n  --> $FILE:2:39"
    };
    df.run_available();

    assert_eq!(&[1, 2, 3], &*collect_ready::<Vec<_>, _>(&mut result_recv));
}

#[test]
fn test_empty_tee() {
    let output = <Rc<RefCell<Vec<usize>>>>::default();
    let output_inner = Rc::clone(&output);

    let mut df = test_warnings! {
        {
            source_iter([1, 2, 3]) -> inspect(|&x| output_inner.borrow_mut().push(x)) -> tee();
        },
        "Warning: `tee` should have at least 2 output(s), actually has 0.\n  --> $FILE:2:90",
    };
    df.run_available();

    assert_eq!(&[1, 2, 3], &**output.borrow());
}

// Mainly checking subgraph partitioning pull-push handling.
// But in a different edge order.
#[test]
pub fn test_warped_diamond() {
    let mut df = test_warnings! {
        {
            // active nodes
            nodes = union();

            // stream of nodes into the system
            init = join() -> for_each(|(n, (a, b))| {
                println!("DEBUG ({:?}, ({:?}, {:?}))", n, a, b);
            });
            new_node = source_iter([1, 2, 3]) -> tee();
            // add self
            new_node[0] -> map(|n| (n, 'a')) -> [0]nodes;
            // join peers against active nodes
            nodes -> [0]init;
            new_node[1] -> map(|n| (n, 'b')) -> [1]init;
        },
        "Warning: `union` should have at least 2 input(s), actually has 1.\n  --> $FILE:3:21",
    };
    df.run_available();
}

#[test]
pub fn test_warped_diamond_2() {
    let mut hf: Hydroflow = test_warnings! {
        {
            // active nodes
            nodes = union();

            // stream of nodes into the system
            init = join() -> for_each(|(n, (a, b))| {
                println!("DEBUG ({:?}, ({:?}, {:?}))", n, a, b);
            });
            new_node = source_iter([1, 2, 3]) -> tee();
            // add self
            new_node[0] -> map(|n| (n, 'a')) -> [0]nodes;
            // join peers against active nodes
            nodes -> [0]init;
            new_node[1] -> map(|n| (n, 'b')) -> [1]init;

            ntwk = source_iter([4, 5, 6]) -> tee();
        },
        "Warning: `union` should have at least 2 input(s), actually has 1.\n  --> $FILE:3:21",
        "Warning: `tee` should have at least 2 output(s), actually has 0.\n  --> $FILE:16:46",
    };
    hf.run_available();
}
