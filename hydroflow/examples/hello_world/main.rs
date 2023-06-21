use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut df = hydroflow_syntax! {
        source_iter([1,2,3,4,5])
            -> map(hydroflow::lattices::Max::new)
            -> lattice_merge::<'static, hydroflow::lattices::Max<usize>>()
            -> assert([hydroflow::lattices::Max(5)]);
    };
    df.run_available();
}
