use hydroflow::hydroflow_syntax;
use multiplatform_test::multiplatform_test;

pub fn main() {
    {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    let mut df = hydroflow_syntax! {
        // 3v1
        source_iter([1]) -> items;
        items = union();

        double = items
            // -> persist()
            -> fold::<'static>(|| 0, |accum, x| *accum += x)
            // -> defer_tick_lazy()
            -> inspect(|x| println!("{} {}: {}", context.current_tick(), context.current_stratum(), x))
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
