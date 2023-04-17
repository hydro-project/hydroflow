use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut df = hydroflow_syntax! {
        source_iter(["Hello World"]) -> for_each(|string| println!("{}", string));
    };

    df.run_available();
}
