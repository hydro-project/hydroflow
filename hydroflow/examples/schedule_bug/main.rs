use hydroflow::hydroflow_syntax;
use multiplatform_test::multiplatform_test;

pub fn main() {
    let mut df = hydroflow_syntax! {
        // 3v1
        source_iter([1]) -> items;
        items = union();

        double = items
            -> persist()
            -> fold(|| 0, |accum, x| *accum += x)
            -> defer_tick_lazy()
            -> filter(|_| false)
            -> tee();

        double -> null();

        double -> items;
    };
    df.meta_graph()
        .unwrap()
        .open_mermaid(&Default::default())
        .unwrap();

    df.run_available();
}

// #[multiplatform_test(test, env_tracing)]
// fn my_test() {
//     main();
// }
