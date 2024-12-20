fn main() {
    let mut df = hydroflow::dfir_syntax! {
        loop {
            source_iter(0..10) -> null();
        }
    };
    df.run_available();
}
