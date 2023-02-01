#[test]
pub fn test_diff_timing() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pos_send, pos_recv) = hydroflow::util::unbounded_channel::<usize>();
    let (neg_send, neg_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow::hydroflow_syntax! {
        diff = difference::<'static>() -> for_each(|x| println!("diff: {:?}", x));

        poss = source_stream(pos_recv); //-> tee();
        poss -> [pos]diff;
        // if you enable the comment below it produces the right answer
        // poss -> for_each(|x| println!("pos: {:?}", x));

        negs = source_stream(neg_recv) -> tee();
        negs -> [neg]diff;
        negs -> for_each(|x| println!("neg: {:?}", x));

    };
    println!("{}", df.serde_graph().unwrap().to_mermaid());

    df.run_tick();
    println!("{}x{}", df.current_tick(), df.current_stratum());

    println!("A");

    pos_send.send(1).unwrap();
    pos_send.send(2).unwrap();
    pos_send.send(3).unwrap();
    pos_send.send(4).unwrap();
    neg_send.send(2).unwrap();
    neg_send.send(3).unwrap();
    df.run_tick();

    println!("B");
    neg_send.send(1).unwrap();
    df.run_tick();
}
