use std::cell::RefCell;
use std::rc::Rc;

use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::scheduled::graph_ext::GraphExt;
use hydroflow::scheduled::handoff::TeeingHandoff;
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
fn test_basic() {
    let mut df = Hydroflow::new();
    let mut data = vec![1, 2, 3, 4];
    let (source, sink1) = df.make_edge::<_, TeeingHandoff<i32>>("ok");
    let sink2 = sink1.tee(&mut df);
    let sink3 = sink2.tee(&mut df);

    df.add_subgraph_source("source", source, move |_context, send| {
        send.give(std::mem::take(&mut data));
    });
    let out1 = Rc::new(RefCell::new(Vec::new()));
    let out1_inner = out1.clone();

    df.add_subgraph_sink("sink1", sink1, move |_context, recv| {
        for v in recv.take_inner() {
            out1_inner.borrow_mut().extend(v);
        }
    });

    let out2 = Rc::new(RefCell::new(Vec::new()));
    let out2_inner = out2.clone();
    df.add_subgraph_sink("sink2", sink2, move |_context, recv| {
        for v in recv.take_inner() {
            out2_inner.borrow_mut().extend(v);
        }
    });

    let out3 = Rc::new(RefCell::new(Vec::new()));
    let out3_inner = out3.clone();
    df.add_subgraph_sink("sink2", sink3, move |_context, recv| {
        for v in recv.take_inner() {
            out3_inner.borrow_mut().extend(v);
        }
    });
    df.run_available();
    assert_eq!((*out1).borrow().clone(), vec![1, 2, 3, 4]);
    assert_eq!((*out2).borrow().clone(), vec![1, 2, 3, 4]);
    assert_eq!((*out3).borrow().clone(), vec![1, 2, 3, 4]);
}

#[multiplatform_test]
fn test_scheduling() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut df = Hydroflow::new();
    let input = Rc::new(RefCell::new(vec![1, 2, 3, 4]));
    let input_recv = Rc::clone(&input);

    let (source, sink1) = df.make_edge::<_, TeeingHandoff<i32>>("teeing-handoff");
    let sink2 = sink1.tee(&mut df);
    let sink3 = sink2.tee(&mut df);

    let src_sg_id = df.add_subgraph_source("source", source, move |_context, send| {
        let vec = std::mem::take(&mut *input_recv.borrow_mut());
        println!("! {:?}", vec);
        send.give(vec);
    });
    let out1 = Rc::new(RefCell::new(Vec::new()));
    let out1_inner = out1.clone();

    df.add_subgraph_sink("sink1", sink1, move |_context, recv| {
        for v in recv.take_inner() {
            out1_inner.borrow_mut().extend(v);
        }
    });

    let out2 = Rc::new(RefCell::new(Vec::new()));
    let out2_inner = out2.clone();
    df.add_subgraph_sink("sink2", sink2, move |_context, recv| {
        for v in recv.take_inner() {
            out2_inner.borrow_mut().extend(v);
        }
    });

    let out3 = Rc::new(RefCell::new(Vec::new()));
    let out3_inner = out3.clone();
    df.add_subgraph_sink("sink2", sink3, move |_context, recv| {
        for v in recv.take_inner() {
            out3_inner.borrow_mut().extend(v);
        }
    });

    df.run_available();
    assert_eq!(&*out1.borrow(), &[1, 2, 3, 4], "out1");
    assert_eq!(&*out2.borrow(), &[1, 2, 3, 4], "out2");
    assert_eq!(&*out3.borrow(), &[1, 2, 3, 4], "out3");

    input.borrow_mut().extend(4..8);
    df.schedule_subgraph(src_sg_id);

    df.run_available();
    assert_eq!(&*out1.borrow(), &[1, 2, 3, 4, 5, 6, 7, 8], "out1");
    assert_eq!(&*out2.borrow(), &[1, 2, 3, 4, 5, 6, 7, 8], "out2");
    assert_eq!(&*out3.borrow(), &[1, 2, 3, 4, 5, 6, 7, 8], "out3");
}
