use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        a = source_iter(10..20) -> map(|x| (x, x)) -> (my_join = join() -> for_each(std::mem::drop));
        b = source_iter(20..30) -> map(|x| (x, x)) -> my_join;
    };
    df.run_available();
}
