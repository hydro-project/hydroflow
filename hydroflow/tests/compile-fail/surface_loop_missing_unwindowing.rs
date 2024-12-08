fn main() {
    let mut df = hydroflow::hydroflow_syntax! {
        a = source_iter(0..10);
        loop {
            b = a -> batch();
        }
        b -> null();
    };
    df.run_available();
}
