use hydroflow::hydroflow_syntax;

#[test]
pub fn test_basic_tee() {
    let mut df = hydroflow_syntax! {
        t = recv_iter([1]) -> tee();
        t -> for_each(|v| println!("A {}", v));
        t -> for_each(|v| println!("B {}", v));
    };
    df.run_available();
}