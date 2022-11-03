use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        t = recv_iter([1]) -> tee();
        t[1] -> for_each(std::mem::drop);
        t[1] -> for_each(std::mem::drop);
    };
    df.run_available();
}
