use hydroflow::hydroflow_syntax;

#[test]
fn test_basic() {
    let mut df = hydroflow_syntax! {
        source_iter([1,2,3,4,5])
            -> lattice_merge::<'static, hydroflow::lang::lattice::ord::MaxRepr<u32>>()
            -> for_each(|x| println!("Least upper bound: {:?}", x));
    };
    df.run_available();
}
