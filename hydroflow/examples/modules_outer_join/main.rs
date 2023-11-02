use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut df = hydroflow_syntax! {
        lhs = source_iter([("a", 0), ("b", 1)]) -> tee();
        rhs = source_iter([("a", 2), ("c", 3)]) -> tee();

        lhs -> [0]inner_join;
        rhs -> [1]inner_join;
        inner_join = join() -> assert_eq([("a", (0, 2))]);

        lhs -> [0]left_outer_join;
        rhs -> [1]left_outer_join;
        left_outer_join = import!("left_outer_join.hf") -> assert_eq([("a", (0, Some(2))), ("b", (1, None))]);

        lhs -> [0]right_outer_join;
        rhs -> [1]right_outer_join;
        right_outer_join = import!("right_outer_join.hf") -> assert_eq([("a", (Some(0), 2)), ("c", (None, 3))]);

        lhs -> [0]full_outer_join;
        rhs -> [1]full_outer_join;
        full_outer_join = import!("full_outer_join.hf") -> assert_eq([("a", (Some(0), Some(2))), ("b", (Some(1), None)), ("c", (None, Some(3)))]);
    };
    df.run_available();
}

#[test]
fn test() {
    main();
}
