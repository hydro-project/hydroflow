use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        source_iter(["Hello", "World"])
            -> map(|s| s.to_uppercase())
            -> for_each(|s| println!("{}", s));
    };

    df.run_available();
}
