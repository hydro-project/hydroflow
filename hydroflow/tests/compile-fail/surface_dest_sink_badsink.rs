fn main() {
    let sink = "not a sink";
    let mut df = hydroflow::hydroflow_syntax! {
        source_iter(0..10) -> dest_sink(sink);
    };
    df.run_available();
}
