use hydroflow_plus::hydroflow::bytes::Bytes;
use hydroflow_plus_kvs::my_kvs;
use regex::Regex;

fn main() {
    let (send_0, recv_0) = hydroflow_plus::hydroflow::util::unbounded_channel::<_>();
    let mut _other_flow = hydroflow_plus_kvs::my_example_flow!(
        recv_0,
        &send_0,
        1,
        {
            let _hygiene = Regex::new("hygiene").unwrap();
            "lol"
        },
        "lol"
    );

    let test = hydroflow_plus_kvs::KVSMessage::Get {
        key: "lol".to_string(),
    };
    let _blah: hydroflow_plus_kvs::__staged::KVSMessage = test;

    let (_send, recv) = hydroflow_plus::hydroflow::util::unbounded_channel::<Bytes>();
    let mut flow = my_kvs!(true, recv);
    flow.run_tick();
}

#[cfg(test)]
mod tests {
    use hydroflow_plus_kvs::raise_to_power;

    #[test]
    fn power() {
        let power = raise_to_power!(2, 15);
        assert_eq!(power, 2i32.pow(15));
    }
}
