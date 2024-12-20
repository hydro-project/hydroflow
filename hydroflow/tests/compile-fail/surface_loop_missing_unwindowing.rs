fn main() {
    let mut df = hydroflow::dfir_syntax! {
        a = source_iter(0..10);
        loop {
            b = a -> batch();
        }
        b -> null();
    };
    df.run_available();
}
