use hydroflow::dfir_syntax;

fn main() {
    let mut df = dfir_syntax! {
        source_iter() -> for_each();
    };
    df.run_available();
}
