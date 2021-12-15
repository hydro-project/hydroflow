use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::mpsc,
};

use hydroflow::scheduled::{
    ctx::{RecvCtx, SendCtx},
    graph::Hydroflow,
    graph_ext::GraphExt,
    handoff::VecHandoff,
};

#[test]
fn map_filter() {
    use hydroflow::scheduled::graph_ext::GraphExt;
    use hydroflow::scheduled::handoff::VecHandoff;
    use std::cell::RefCell;
    use std::rc::Rc;

    // A simple dataflow with one source feeding into one sink with some processing in the middle.
    let mut df = Hydroflow::new();

    let data = [1, 2, 3, 4];
    let source = df.add_source(move |_ctx, send| {
        for x in data.into_iter() {
            send.give(Some(x));
        }
    });

    let (map_in, map_out) = df.add_inout(
        |_ctx, recv: &RecvCtx<VecHandoff<i32>>, send: &SendCtx<VecHandoff<_>>| {
            for x in recv.take_inner().into_iter() {
                send.give(Some(3 * x + 1));
            }
        },
    );

    let (filter_in, filter_out) = df.add_inout(
        |_ctx, recv: &RecvCtx<VecHandoff<i32>>, send: &SendCtx<VecHandoff<_>>| {
            for x in recv.take_inner().into_iter() {
                if x % 2 == 0 {
                    send.give(Some(x));
                }
            }
        },
    );

    let outputs = Rc::new(RefCell::new(Vec::new()));
    let inner_outputs = outputs.clone();
    let sink = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<i32>>| {
        for x in recv.take_inner().into_iter() {
            (*inner_outputs).borrow_mut().push(x);
        }
    });

    df.add_edge(source, map_in);
    df.add_edge(map_out, filter_in);
    df.add_edge(filter_out, sink);

    df.tick();

    assert_eq!((*outputs).borrow().clone(), vec![4, 10]);
}

#[test]
fn test_basic_variadic() {
    let mut df = Hydroflow::new();
    let source_handle = df.add_source(move |_ctx, send: &SendCtx<VecHandoff<usize>>| {
        send.give(Some(5));
    });

    let val = <Rc<Cell<Option<usize>>>>::default();
    let val_ref = val.clone();

    let sink_handle = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<usize>>| {
        for v in recv.take_inner().into_iter() {
            let old_val = val_ref.replace(Some(v));
            assert!(old_val.is_none()); // Only run once.
        }
    });

    df.add_edge(source_handle, sink_handle);
    df.tick();

    assert_eq!(Some(5), val.get());
}

#[test]
fn test_basic_n_m() {
    let mut df = Hydroflow::new();
    let (_, mut source_handle) = df.add_n_in_m_out(
        0,
        1,
        move |_: &[&RecvCtx<VecHandoff<usize>>], send: &[&SendCtx<VecHandoff<usize>>]| {
            send[0].give(Some(5));
        },
    );

    let val = <Rc<Cell<Option<usize>>>>::default();
    let val_ref = val.clone();

    let (mut sink_handle, _) = df.add_n_in_m_out(
        1,
        0,
        move |recv: &[&RecvCtx<VecHandoff<usize>>], _: &[&SendCtx<VecHandoff<usize>>]| {
            for v in recv[0].take_inner().into_iter() {
                let old_val = val_ref.replace(Some(v));
                assert!(old_val.is_none()); // Only run once.
            }
        },
    );

    df.add_edge(source_handle.pop().unwrap(), sink_handle.pop().unwrap());
    df.tick();

    assert_eq!(Some(5), val.get());
}

#[test]
fn test_cycle() {
    // A dataflow that represents graph reachability.

    let mut edges: HashMap<usize, Vec<usize>> = HashMap::new();
    for (from, to) in &[
        (1_usize, 2_usize),
        (1, 3),
        (1, 4),
        (2, 3),
        (2, 5),
        (5, 1),
        (6, 7),
        (7, 8),
    ] {
        edges.entry(*from).or_insert_with(Vec::new).push(*to);
    }

    let mut df = Hydroflow::new();

    let mut initially_reachable = vec![1];
    let reachable = df.add_source(move |_ctx, send: &SendCtx<VecHandoff<usize>>| {
        for v in initially_reachable.drain(..) {
            send.give(Some(v));
        }
    });

    let mut seen = HashSet::new();
    let (distinct_in, distinct_out) = df.add_inout(
        move |_ctx, recv: &RecvCtx<VecHandoff<usize>>, send: &SendCtx<VecHandoff<usize>>| {
            for v in recv.take_inner().into_iter() {
                if seen.insert(v) {
                    send.give(Some(v));
                }
            }
        },
    );

    let (merge_lhs, merge_rhs, merge_out) = df.add_binary(
        |_ctx,
         recv1: &RecvCtx<VecHandoff<usize>>,
         recv2: &RecvCtx<VecHandoff<usize>>,
         send: &SendCtx<VecHandoff<usize>>| {
            for v in (recv1.take_inner().into_iter()).chain(recv2.take_inner().into_iter()) {
                send.give(Some(v));
            }
        },
    );

    let (neighbors_in, neighbors_out) =
        df.add_inout(move |_ctx, recv: &RecvCtx<VecHandoff<usize>>, send| {
            for v in recv.take_inner().into_iter() {
                if let Some(neighbors) = edges.get(&v) {
                    for &n in neighbors {
                        send.give(Some(n));
                    }
                }
            }
        });

    let (tee_in, tee_out1, tee_out2) = df.add_binary_out(
        |_ctx,
         recv: &RecvCtx<VecHandoff<usize>>,
         send1: &SendCtx<VecHandoff<usize>>,
         send2: &SendCtx<VecHandoff<usize>>| {
            for v in recv.take_inner().into_iter() {
                send1.give(Some(v));
                send2.give(Some(v));
            }
        },
    );

    let reachable_verts = Rc::new(RefCell::new(Vec::new()));
    let reachable_inner = reachable_verts.clone();
    let sink_in = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<usize>>| {
        for v in recv.take_inner().into_iter() {
            (*reachable_inner).borrow_mut().push(v);
        }
    });

    df.add_edge(reachable, merge_lhs);
    df.add_edge(neighbors_out, merge_rhs);
    df.add_edge(merge_out, distinct_in);
    df.add_edge(distinct_out, tee_in);
    df.add_edge(tee_out1, neighbors_in);
    df.add_edge(tee_out2, sink_in);

    df.tick();

    assert_eq!((*reachable_verts).borrow().clone(), vec![1, 2, 3, 4, 5]);
}

// #[test]
// fn test_auto_tee() {
//     use std::cell::RefCell;
//     use std::rc::Rc;

//     use crate::scheduled::handoff::TeeingHandoff;

//     let mut df = Hydroflow::new();

//     let mut data = vec![1, 2, 3, 4];
//     let source = df.add_source(move |send: &SendCtx<TeeingHandoff<_>>| {
//         send.give(std::mem::take(&mut data));
//     });

//     let out1 = Rc::new(RefCell::new(Vec::new()));
//     let out1_inner = out1.clone();

//     let sink1 = df.add_sink(move |recv: &RecvCtx<_>| {
//         for v in recv.take_inner() {
//             out1_inner.borrow_mut().extend(v);
//         }
//     });

//     let out2 = Rc::new(RefCell::new(Vec::new()));
//     let out2_inner = out2.clone();
//     let sink2 = df.add_sink(move |recv: &RecvCtx<_>| {
//         for v in recv.take_inner() {
//             out2_inner.borrow_mut().extend(v);
//         }
//     });

//     df.add_edge(source.clone(), sink1);
//     df.add_edge(source, sink2);

//     df.tick();

//     assert_eq!((*out1).borrow().clone(), vec![1, 2, 3, 4]);
//     assert_eq!((*out2).borrow().clone(), vec![1, 2, 3, 4]);
// }

#[test]
fn test_input_handle() {
    use hydroflow::scheduled::graph_ext::GraphExt;
    use hydroflow::scheduled::handoff::VecHandoff;
    use std::cell::RefCell;

    let mut df = Hydroflow::new();

    let (input, output_port) = df.add_input();

    let vec = Rc::new(RefCell::new(Vec::new()));
    let inner_vec = vec.clone();
    let input_port = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<usize>>| {
        for v in recv.take_inner() {
            (*inner_vec).borrow_mut().push(v);
        }
    });

    df.add_edge(output_port, input_port);

    input.give(Some(1));
    input.give(Some(2));
    input.give(Some(3));
    input.flush();

    df.tick();

    assert_eq!((*vec).borrow().clone(), vec![1, 2, 3]);

    input.give(Some(4));
    input.give(Some(5));
    input.give(Some(6));
    input.flush();

    df.tick();

    assert_eq!((*vec).borrow().clone(), vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_input_handle_thread() {
    use hydroflow::scheduled::graph_ext::GraphExt;
    use hydroflow::scheduled::handoff::VecHandoff;
    use std::cell::RefCell;

    let mut df = Hydroflow::new();

    let (input, output_port) = df.add_channel_input();

    let vec = Rc::new(RefCell::new(Vec::new()));
    let inner_vec = vec.clone();
    let input_port = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<usize>>| {
        for v in recv.take_inner() {
            (*inner_vec).borrow_mut().push(v);
        }
    });

    df.add_edge(output_port, input_port);

    let (done, wait) = mpsc::channel();

    std::thread::spawn(move || {
        input.give(Some(1));
        input.give(Some(2));
        input.give(Some(3));
        input.flush();
        done.send(()).unwrap();
    });

    wait.recv().unwrap();

    df.tick();

    assert_eq!((*vec).borrow().clone(), vec![1, 2, 3]);
}

#[test]
fn test_input_channel() {
    // This test creates two parallel Hydroflow graphs and bounces messages back
    // and forth between them.

    use futures::channel::mpsc::channel;
    use hydroflow::scheduled::graph_ext::GraphExt;
    use hydroflow::scheduled::handoff::VecHandoff;
    use std::cell::Cell;

    let (s1, r1) = channel(8000);
    let (s2, r2) = channel(8000);

    let mut s1_outer = s1.clone();
    let pairs = [(s1, r2), (s2, r1)];

    // logger/recv is a channel that each graph plops their messages into, to be
    // able to trace what happens.
    let (logger, mut recv) = channel(8000);

    for (mut sender, receiver) in pairs {
        let mut logger = logger.clone();
        std::thread::spawn(move || {
            let done = Rc::new(Cell::new(false));
            let done_inner = done.clone();
            let mut df = Hydroflow::new();

            let in_chan = df.add_input_from_stream::<_, VecHandoff<usize>, _>(receiver);
            let input = df.add_sink(move |_ctx, recv| {
                for v in recv.take_inner() {
                    logger.try_send(v).unwrap();
                    if v > 0 && sender.try_send(Some(v - 1)).is_err() {
                        (*done_inner).set(true);
                    }
                }
            });
            df.add_edge(in_chan, input);

            while !(*done).get() {
                df.tick();
                df.poll_events().unwrap();
            }
        });
    }

    s1_outer.try_send(Some(10_usize)).unwrap();

    let mut result = Vec::new();
    let expected = vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
    loop {
        let val = recv.try_next();
        match val {
            Err(_) => {
                if result.len() >= expected.len() {
                    break;
                }
            }
            Ok(None) => {
                break;
            }
            Ok(Some(v)) => {
                result.push(v);
            }
        }
    }
    assert_eq!(result, expected);
}
