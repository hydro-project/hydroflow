use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        recv_iter() -> for_each();
    };
    df.run_available();
}
