use hydroflow::hydroflow_syntax;
use hydroflow::lattices::ght::GeneralizedHashTrie;
use hydroflow::lattices::GhtType;
use hydroflow::util::collect_ready;
use hydroflow::variadics::{var_expr, var_type};
use lattices::ght_lattice::{DeepJoinLatticeBimorphism, GhtBimorphism};

#[test]
fn test_basic() {
    type MyGHT = GhtType!(u16, u32 => u64);
    type FlatTup = var_type!(u16, u32, u64);
    let input: Vec<FlatTup> = vec![
        var_expr!(42, 314, 43770),
        var_expr!(42, 315, 43770),
        var_expr!(42, 314, 30619),
        var_expr!(43, 10, 600),
    ];
    let mut merged = MyGHT::default();
    for i in input.clone() {
        merged.insert(i);
    }
    println!("merged: {:?}", merged);
    let mut df = hydroflow_syntax! {
        source_iter(input)
            -> map(|t| MyGHT::new_from(vec![t]))
            -> lattice_fold::<'static>(MyGHT::default)
            -> inspect(|t| println!("{:?}", t))
            -> assert(|x: &MyGHT| x.eq(&merged))
            -> null();
    };
    df.run_available();
}

#[test]
fn test_join() {
    type MyGHT = GhtType!(u16 => u16);
    type MyGhtTrie = <MyGHT as GeneralizedHashTrie>::Trie;
    type ResultGht = GhtType!(u16 => u16, u16);
    let (out_send, out_recv) = hydroflow::util::unbounded_channel::<_>();

    let r = vec![
        var_expr!(1, 10),
        var_expr!(2, 20),
        var_expr!(3, 30),
        var_expr!(4, 40),
    ];
    let s = vec![var_expr!(1, 100), var_expr!(5, 500)];

    type MyNodeBim =
        <(MyGhtTrie, MyGhtTrie) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism;
    type MyBim = GhtBimorphism<MyNodeBim>;

    let mut df = hydroflow_syntax! {
        R = source_iter(r)
            -> map(|t| MyGHT::new_from([t]))
            -> state::<MyGHT>();
        S = source_iter(s)
            -> map(|t| MyGHT::new_from([t]))
            -> state::<MyGHT>();
        R[items] -> [0]my_join;
        S[items] -> [1]my_join;
        my_join = lattice_bimorphism(MyBim::default(), #R, #S)
            -> lattice_reduce() // currently required to remove spurious "early returns"
            -> for_each(|x| out_send.send(x).unwrap());
    };
    df.run_available();

    assert_eq!(
        &[ResultGht::new_from(vec![var_expr!(1, 10, 100),])],
        &*collect_ready::<Vec<_>, _>(out_recv)
    );
}
