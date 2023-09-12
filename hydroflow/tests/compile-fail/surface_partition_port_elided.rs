use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        my_partition = source_iter(0..10) -> partition(|item, [evens, odds]| {
            if 0 == item % 2 {
                evens
            }
            else {
                odds
            }
        });
        my_partition[evens] -> for_each(std::mem::drop);
        my_partition -> for_each(std::mem::drop);
    };
    df.run_available();
}
