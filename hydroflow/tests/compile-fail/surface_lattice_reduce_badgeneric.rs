fn main() {
    let mut df = hydroflow::hydroflow_syntax! {
        source_iter([1,2,3,4,5])
            -> lattice_reduce::<'static, usize>()
            -> for_each(|x| println!("Least upper bound: {:?}", x));
    };
    df.run_available();
}
