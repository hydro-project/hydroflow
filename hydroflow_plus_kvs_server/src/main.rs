use hydroflow_plus::hydroflow::bytes::Bytes;
use hydroflow_plus_kvs::my_example_flow;

fn main() {
    let test = hydroflow_plus_kvs::KVSMessage::Get {
        key: "lol".to_string(),
    };
    let _blah: hydroflow_plus_kvs::__flow::KVSMessage = test;

    let (_send, recv) = hydroflow_plus::hydroflow::util::unbounded_channel::<Bytes>();
    let mut flow = my_example_flow!(1, recv);
    flow.run_tick();
}
