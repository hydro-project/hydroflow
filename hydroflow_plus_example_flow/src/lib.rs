use hydroflow_plus::{q, qtype, quse, HydroflowContext, HydroflowNode, RuntimeData};

qtype! {
    struct Test {
        pub v: String
    }
}

fn filter_by_regex<'a, S: Copy + AsRef<str> + 'a>(
    ctx: &HydroflowContext<'a>,
    input: HydroflowNode<'a, String>,
    pattern: RuntimeData<S>,
) -> HydroflowNode<'a, String> {
    quse!(::regex::Regex);

    let ctx = ctx.runtime_context();

    input.filter(q!({
        let regex = Regex::new(pattern.as_ref()).unwrap();
        move |x| {
            dbg!(ctx.current_tick());
            let constructed_test = Test { v: x.clone() };
            dbg!(constructed_test.v);
            regex.is_match(x)
        }
    }))
}

#[hydroflow_plus::flow(&'static str)]
pub fn my_example_flow<'a, S: Copy + AsRef<str> + 'a>(
    ctx: &HydroflowContext<'a>,
    input_stream: RuntimeData<
        ::hydroflow_plus::hydroflow::tokio_stream::wrappers::UnboundedReceiverStream<String>,
    >,
    number_of_foreach: u32,
    regex: RuntimeData<S>,
    text: RuntimeData<&'a str>,
) {
    let source = ctx.source_stream(q!(input_stream));

    let mapped = filter_by_regex(ctx, source, regex);

    for _ in 0..number_of_foreach {
        mapped.for_each(q!(move |x| println!("passed regex {} {}", text, x)));
    }
}
