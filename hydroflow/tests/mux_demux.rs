use std::{cell::RefCell, rc::Rc};

use hydroflow::{
    lang::collections::Iter,
    scheduled::{
        ctx::{RecvCtx, SendCtx},
        graph::Hydroflow,
        graph_demux::GraphDemux,
        graph_ext::GraphExt,
        handoff::VecHandoff,
    },
};

#[test]
fn test_mux_demux() {
    // This test creates a demux which dispatches on the parity of the input
    // integer, then adds two sinks, and verifies they get the right values.
    let mut df = Hydroflow::new();
    let out_even = Rc::new(RefCell::new(Vec::new()));
    let out_even_inner = out_even.clone();
    let even_sink = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<usize>>| {
        (*out_even_inner)
            .borrow_mut()
            .extend(recv.take_inner().into_iter());
    });
    let out_odd = Rc::new(RefCell::new(Vec::new()));
    let out_odd_inner = out_odd.clone();
    let odd_sink = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<usize>>| {
        (*out_odd_inner)
            .borrow_mut()
            .extend(recv.take_inner().into_iter());
    });
    let mut data: Vec<_> = (0..20).collect();
    let source = df.add_source(move |_ctx, send: &SendCtx<VecHandoff<usize>>| {
        send.give(Iter(data.drain(..)));
    });

    let (demux, demux_input) = df.add_demux(|x| x % 2);

    df.add_edge(source, demux_input);
    df.add_demux_edge(&demux, 0, even_sink);
    df.add_demux_edge(&demux, 1, odd_sink);

    df.tick();

    assert_eq!((*out_even).take(), vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
    assert_eq!((*out_odd).take(), vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19]);
}
