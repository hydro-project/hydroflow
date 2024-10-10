use hydroflow::hydroflow_syntax;
use hydroflow::lattices::ght::GeneralizedHashTrieNode;
use hydroflow::lattices::ght_lattice::{DeepJoinLatticeBimorphism, GhtBimorphism};
use hydroflow::lattices::GhtType;
use hydroflow::util::collect_ready;
use hydroflow::variadics::{var_expr, var_type};
use variadics::variadic_collections::VariadicHashSet; // Import the Insert trait

#[test]
fn test_basic() {
    type MyGht = GhtType!(u16, u32 => u64: Set);
    type FlatTup = var_type!(u16, u32, u64);
    let input: Vec<FlatTup> = vec![
        var_expr!(42, 314, 43770),
        var_expr!(42, 315, 43770),
        var_expr!(42, 314, 30619),
        var_expr!(43, 10, 600),
    ];
    let mut merged = MyGht::default();
    for i in input.clone() {
        merged.insert(i);
    }
    println!("merged: {:?}", merged);
    let mut df = hydroflow_syntax! {
        source_iter(input)
            -> map(|t| MyGht::new_from(vec![t]))
            -> lattice_fold::<'static>(MyGht::default)
            -> inspect(|t| println!("{:?}", t))
            -> assert(|x: &MyGht| x.eq(&merged))
            -> null();
    };
    df.run_available();
}

#[test]
fn test_join() {
    type MyGht = GhtType!(u8 => u16: Set);
    type ResultGht = GhtType!(u8 => u16, u16: Set);
    let (out_send, out_recv) = hydroflow::util::unbounded_channel::<_>();

    let r = vec![
        var_expr!(1, 10),
        var_expr!(2, 20),
        var_expr!(3, 30),
        var_expr!(4, 40),
    ];
    let s = vec![var_expr!(1, 10), var_expr!(5, 50)];

    type MyNodeBim = <(MyGht, MyGht) as DeepJoinLatticeBimorphism<
        VariadicHashSet<var_type!(u8, u16, u16)>,
    >>::DeepJoinLatticeBimorphism;
    type MyBim = GhtBimorphism<MyNodeBim>;

    let mut df = hydroflow_syntax! {
        R = source_iter(r)
            -> map(|t| MyGht::new_from([t]))
            -> state::<MyGht>();
        S = source_iter(s)
            -> map(|t| MyGht::new_from([t]))
            -> state::<MyGht>();
        R[items] -> [0]my_join;
        S[items] -> [1]my_join;
        my_join = lattice_bimorphism(MyBim::default(), #R, #S)
            -> lattice_reduce()
            -> for_each(|x| out_send.send(x).unwrap());
    };
    df.run_available();

    assert_eq!(
        &[ResultGht::new_from(vec![var_expr!(1, 10, 10),])],
        &*collect_ready::<Vec<_>, _>(out_recv)
    );
}
