use hydroflow_plus::hydroflow::bytes::Bytes;
use hydroflow_plus_kvs::my_kvs;

fn main() {
    let test = hydroflow_plus_kvs::KVSMessage::Get {
        key: "lol".to_string(),
    };
    let _blah: hydroflow_plus_kvs::__staged::KVSMessage = test;

    let (_send, recv) = hydroflow_plus::hydroflow::util::unbounded_channel::<Bytes>();
    let mut flow = my_kvs!(true, recv);
    flow.run_tick();
}
