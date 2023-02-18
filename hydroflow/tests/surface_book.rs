#[test]
fn test_surface_flows_1() {
    let mut df = hydroflow::hydroflow_syntax! {
        my_tee = source_iter(vec!["Hello", "world"]) -> tee();
        my_tee[0] -> map(|x| x.to_uppercase()) -> [0]my_merge;
        my_tee[1] -> map(|x| x.to_lowercase()) -> [1]my_merge;
        my_merge = merge() -> for_each(|x| println!("{}", x));
    };
    println!("{}", df.serde_graph().unwrap().to_mermaid());
    df.run_available();
}
