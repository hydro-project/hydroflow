use hydroflow_plus::{q, qtype, quse, HydroflowContext, HydroflowNode, RuntimeData};

qtype! {
    struct Test {
        pub v: String
    }
}

fn filter_by_regex<'a, S: AsRef<str> + 'a>(
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

#[hydroflow_plus::flow(String)]
pub fn my_example_flow<'a, S: AsRef<str> + 'a>(
    ctx: &HydroflowContext<'a>,
    number_of_foreach: u32,
    regex: RuntimeData<S>,
    text: RuntimeData<&'a str>,
) {
    let source = ctx.source_iter(q!(vec!["abc".to_string(), "def".to_string()]));

    let mapped = filter_by_regex(ctx, source, regex);

    for _ in 0..number_of_foreach {
        mapped.for_each(q!(move |x| println!("passed regex {} {}", text, x)));
    }
}
