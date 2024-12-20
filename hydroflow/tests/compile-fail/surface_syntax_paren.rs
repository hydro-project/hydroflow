use hydroflow::dfir_syntax;

fn main() {
    let mut df = dfir_syntax! {
        source_iter(0..10) (for_each(std::mem::drop))
    };
    df.run_available();
}
