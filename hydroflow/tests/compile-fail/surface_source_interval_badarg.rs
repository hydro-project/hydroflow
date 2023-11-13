use hydroflow::hydroflow_syntax;

#[hydroflow::main]
async fn main() {
    let mut df = hydroflow_syntax! {
        // Should be a `Duration`.
        source_interval(5) -> for_each(std::mem::drop);
    };
    df.run_async().await;
}
