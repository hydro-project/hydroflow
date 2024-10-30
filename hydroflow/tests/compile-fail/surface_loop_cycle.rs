fn main() {
    let mut df = hydroflow::hydroflow_syntax! {
        loop {
            a = identity() -> identity() -> identity() -> identity();
            a -> a;
        }
    };
    df.run_available();
}
