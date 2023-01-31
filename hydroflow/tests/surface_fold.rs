use std::collections::{HashMap, HashSet};

use hydroflow::{hydroflow_syntax, util::collect_ready};

#[test]
pub fn test_fold_tick() {
    let (items_send, items_recv) = hydroflow::util::unbounded_channel::<Vec<u32>>();
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<Vec<u32>>();

    let mut df = hydroflow::hydroflow_syntax! {
        source_stream(items_recv)
            -> fold::<'tick>(Vec::new(), |mut old: Vec<u32>, mut x: Vec<u32>| { old.append(&mut x); old })
            -> for_each(|v| result_send.send(v).unwrap());
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );

    items_send.send(vec![1, 2]).unwrap();
    items_send.send(vec![3, 4]).unwrap();
    df.run_available();

    assert_eq!(
        &[vec![1, 2, 3, 4]],
        &*hydroflow::util::collect_ready::<Vec<_>, _>(&mut result_recv)
    );

    items_send.send(vec![5, 6]).unwrap();
    items_send.send(vec![7, 8]).unwrap();
    df.run_available();

    assert_eq!(
        &[vec![5, 6, 7, 8]],
        &*hydroflow::util::collect_ready::<Vec<_>, _>(&mut result_recv)
    );
}

#[test]
pub fn test_fold_static() {
    let (items_send, items_recv) = hydroflow::util::unbounded_channel::<Vec<u32>>();
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<Vec<u32>>();

    let mut df = hydroflow::hydroflow_syntax! {
        source_stream(items_recv)
            -> fold::<'static>(Vec::new(), |mut old: Vec<u32>, mut x: Vec<u32>| { old.append(&mut x); old })
            -> for_each(|v| result_send.send(v).unwrap());
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );

    items_send.send(vec![1, 2]).unwrap();
    items_send.send(vec![3, 4]).unwrap();
    df.run_available();

    assert_eq!(
        &[vec![1, 2, 3, 4]],
        &*hydroflow::util::collect_ready::<Vec<_>, _>(&mut result_recv)
    );

    items_send.send(vec![5, 6]).unwrap();
    items_send.send(vec![7, 8]).unwrap();
    df.run_available();

    assert_eq!(
        &[vec![1, 2, 3, 4, 5, 6, 7, 8]],
        &*hydroflow::util::collect_ready::<Vec<_>, _>(&mut result_recv)
    );
}

#[test]
pub fn test_fold_flatten() {
    // test pull
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<(u8, u8)>();
    let mut df_pull = hydroflow_syntax! {
        source_iter([(1,1), (1,2), (2,3), (2,4)])
            -> fold::<'tick>(HashMap::<u8,u8>::new(), |mut ht, t: (u8,u8)| {
                    let e = ht.entry(t.0).or_insert(0);
                    *e += t.1;
                    ht})
            -> flatten()
            -> for_each(|(k,v)| out_send.send((k,v)).unwrap());
    };
    df_pull.run_available();

    let out: HashSet<_> = collect_ready(&mut out_recv);
    for pair in [(1, 3), (2, 7)] {
        assert!(out.contains(&pair));
    }

    // test push
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<(u8, u8)>();
    let mut df_push = hydroflow_syntax! {
        datagen = source_iter([(1,2), (1,2), (2,4), (2,4)]) -> tee();
        datagen[0] -> fold::<'tick>(HashMap::<u8,u8>::new(), |mut ht, t:(u8,u8)| {
                let e = ht.entry(t.0).or_insert(0);
                *e += t.1;
                ht})
            -> flatten()
            -> for_each(|(k,v)| out_send.send((k,v)).unwrap());
        datagen[1] -> null();
    };

    df_push.run_available();

    let out: HashSet<_> = collect_ready(&mut out_recv);
    for pair in [(1, 4), (2, 8)] {
        assert!(out.contains(&pair));
    }
}

#[test]
pub fn test_fold_sort() {
    let (items_send, items_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_stream(items_recv)
            -> fold::<'tick>(Vec::new(), |mut v, x| {
                v.push(x);
                v
            })
            -> flat_map(|mut vec| { vec.sort(); vec })
            -> for_each(|v| print!("{:?}, ", v));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    print!("\nA: ");

    items_send.send(9).unwrap();
    items_send.send(2).unwrap();
    items_send.send(5).unwrap();
    df.run_available();

    print!("\nB: ");

    items_send.send(9).unwrap();
    items_send.send(5).unwrap();
    items_send.send(2).unwrap();
    items_send.send(0).unwrap();
    items_send.send(3).unwrap();
    df.run_available();

    println!();
}
