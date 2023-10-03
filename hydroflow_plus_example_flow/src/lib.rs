use hydroflow_plus::quoting::Quoted;
use hydroflow_plus::*;

quse_type!(::regex::Regex);
quse_type!(::hydroflow_plus::hydroflow::tokio_stream::wrappers::UnboundedReceiverStream);
quse_type!(::hydroflow_plus::hydroflow::scheduled::graph::Hydroflow);
qtype! {
    struct Test {
        pub v: String
    }
}

fn filter_by_regex<'a, S: Copy + AsRef<str> + 'a>(
    graph: &HfGraph<'a>,
    input: HfNode<'a, String>,
    pattern: RuntimeData<S>,
) -> HfNode<'a, String> {
    let ctx = graph.runtime_context();

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

#[hydroflow_plus::entry(&'static str)]
pub fn my_example_flow<'a, S: Copy + AsRef<str> + 'a>(
    graph: &'a HfGraph<'a>,
    input_stream: RuntimeData<UnboundedReceiverStream<String>>,
    number_of_foreach: u32,
    regex: RuntimeData<S>,
    text: RuntimeData<&'a str>,
) -> impl Quoted<Hydroflow<'a>> {
    let source = graph.source_stream(q!(input_stream));

    let mapped = filter_by_regex(graph, source, regex);

    for _ in 0..number_of_foreach {
        mapped.for_each(q!(move |x| println!("passed regex {} {}", text, x)));
    }

    graph.build()
}
