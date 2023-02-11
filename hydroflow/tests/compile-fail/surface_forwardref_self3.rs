use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        f = f;
        f -> for_each(std::mem::drop);
    };
    df.run_available();
}
