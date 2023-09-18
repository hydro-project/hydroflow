use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut df = hydroflow_syntax! {
        source_iter(["Hello World"])
            -> assert_eq(["Hello World"]);
    };
    df.run_available();
}

#[test]
fn test() {
    main();
}
