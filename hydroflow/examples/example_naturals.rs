use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut flow = hydroflow_syntax! {
        base = source_iter(vec![1]) -> cycle;
        cycle = union()
                -> map(|i| i + 1)
                -> inspect(|i| println!("{}, ", i))
                -> cycle;
    };

    flow.run_available();
}
