use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        j = join::<usize>() -> for_each(std::mem::drop);
        source_iter(0..10) -> map(|x| (x, x)) -> [0]j;
        source_iter(0..10) -> map(|x| (x, x)) -> [1]j;
    };
    df.run_available();
}
