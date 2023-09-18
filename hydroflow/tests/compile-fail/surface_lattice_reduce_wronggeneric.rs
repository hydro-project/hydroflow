use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        source_iter([1,2,3,4,5])
            -> lattice_reduce::<'static, hydroflow::lattices::set_union::SetUnionHashSet<u32>>()
            -> for_each(|x| println!("Least upper bound: {:?}", x));
    };
    df.run_available();
}
