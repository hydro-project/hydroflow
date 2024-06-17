use hydroflow::hydroflow_syntax;
use hydroflow::lattices::generalized_hash_trie::GeneralizedHashTrie;
use hydroflow::lattices::GhtType;
use hydroflow::variadics::{var_expr, var_type}; // Import the Insert trait

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

// #[test]
// fn test_join() {
//     type MyGHT = GhtType!(u16, u16);
//     let r = vec![
//         var_expr!(1, 10),
//         var_expr!(2, 20),
//         var_expr!(3, 30),
//         var_expr!(4, 40),
//     ];
//     let s = vec![var_expr!(1, 10), var_expr!(5, 50)];

//     let mut df = hydroflow_syntax! {
//         R = source_iter(r)
//             -> map(|t| MyGHT::new(vec![t]))
//             -> lattice_fold::<'static>(MyGHT::default)
//             -> state::<MyGHT>();
//         S = source_iter(r)
//             -> map(|t| MyGHT::new(vec![t]))
//             -> lattice_fold::<'static>(MyGHT::default)
//             -> state::<MyGHT>();
//         R[items] -> [0]my_join;
//         S[items] -> [1]my_join;
//         my_join = lattice_bimorphism(KeyedBimorphism::<HashMap<_, _>, _>::from(CartesianProductBimorphism::<HashSet<_>>::default()), #R, #S)
//             -> for_each(|t| println!("{:?}", t));
//     };
//     df.run_available();
// }
