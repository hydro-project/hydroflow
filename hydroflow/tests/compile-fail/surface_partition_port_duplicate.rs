use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        my_partition = source_iter(0..10) -> partition(|item, [a, b, c]| {
            match item % 3 {
                0 => a,
                1 => b,
                2 => c,
            }
        });
        my_partition[a] -> for_each(std::mem::drop);
        my_partition[b] -> for_each(std::mem::drop);
        my_partition[c] -> for_each(std::mem::drop);
        my_partition[c] -> for_each(std::mem::drop);
    };
    df.run_available();
}
