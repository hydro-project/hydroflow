fn main() {
    let mut df = hydroflow::dfir_syntax! {
        loop {
            a = identity() -> identity() -> identity() -> identity();
            a -> a;
        }
    };
    df.run_available();
}
