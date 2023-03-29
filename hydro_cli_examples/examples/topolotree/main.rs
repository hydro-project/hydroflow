use hydroflow::bytes::Bytes;
use hydroflow::hydroflow_syntax;
use hydroflow::tokio;
use hydroflow::util::cli::{ConnectedBidi, ConnectedSink, ConnectedSource};
use hydroflow::util::{deserialize_from_bytes, serialize_to_bytes};

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let increment_requests = ports
        .remove("increment_requests")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    // let query_requests = ports
    //     .remove("query_requests")
    //     .unwrap()
    //     .connect::<ConnectedBidi>()
    //     .await
    //     .into_source();

    let query_responses = ports
        .remove("query_responses")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_sink();

    let to_parent = ports
        .remove("to_parent")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_sink();

    let from_parent = ports
        .remove("from_parent")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let to_left = ports
        .remove("to_left")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_sink();

    let from_left = ports
        .remove("from_left")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let to_right = ports
        .remove("to_right")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_sink();

    let from_right = ports
        .remove("from_right")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let my_merge_function = |(mut current_time, mut current_value), (x, y)| {
        if x > current_time {
            current_time = x;
            current_value = y;
        }

        (current_time, current_value)
    };

    let time_incrementer = |(mut current_time, mut current_value), new_value| {
        if new_value != current_value {
            current_time = current_time + 1;
            current_value = new_value;
        }

        (current_time, current_value)
    };

    let update_value = |mut current_value, new_value| {
        current_value = current_value + new_value;

        current_value
    };

    let mut df = hydroflow_syntax! {
        from_parent = source_stream(from_parent)
            -> map(|x| deserialize_from_bytes::<(i32,i32)>(x.unwrap()))
            // -> inspect(|x| println!("from_parent: {x:?}"))
            -> fold::<'static>((0,0), my_merge_function)
            -> map(|(_current_time, current_value)| current_value)
            -> tee();

        from_left = source_stream(from_left)
            -> map(|x| deserialize_from_bytes::<(i32,i32)>(x.unwrap()))
            // -> inspect(|x| println!("from_left: {x:?}"))
            -> fold::<'static>((0,0), my_merge_function)
            -> map(|(_current_time, current_value)| current_value)
            -> tee();

        from_right = source_stream(from_right)
            -> map(|x| deserialize_from_bytes::<(i32,i32)>(x.unwrap()))
            // -> inspect(|x| println!("from_right: {x:?}"))
            -> fold::<'static>((0,0), my_merge_function)
            -> map(|(_current_time, current_value)| current_value)
            -> tee();

        from_local = source_stream(increment_requests) //TODO implement
            -> map(|x| String::from_utf8(x.unwrap().to_vec()).unwrap())
            -> map(|x| serde_json::from_str::<u32>(&x).unwrap())
            -> fold::<'static>((0,0), |old, req| {
                let (time, value) = old;
                (time + 1, value + (req as i32))
            })
            -> fold::<'static>((0,0), my_merge_function)
            -> map(|(_current_time, current_value)| current_value)
            -> tee();

        to_right = merge();

        from_parent -> to_right;
        from_left -> to_right;
        from_local -> to_right;

        to_right
            -> fold::<'tick>(0, update_value)
            -> fold::<'static>((0, 0), time_incrementer)
            -> unique::<'static>()
            // -> inspect(|x| println!("to_right: {x:?}"))
            -> map(|v| serialize_to_bytes(v))
            -> dest_sink(to_right); //send result to output channel

        to_left = merge();

        from_parent -> to_left;
        from_right -> to_left;
        from_local -> to_left;

        to_left
            -> fold::<'tick>(0, update_value)
            -> fold::<'static>((0, 0), time_incrementer)
            -> unique::<'static>()
            // -> inspect(|x| println!("to_left: {x:?}"))
            -> map(|v| serialize_to_bytes(v))
            -> dest_sink(to_left); //send result to output channel

        to_parent = merge();

        from_right -> to_parent;
        from_left -> to_parent;
        from_local -> to_parent;

        to_parent
            -> fold::<'tick>(0, update_value)
            -> fold::<'static>((0, 0), time_incrementer)
            -> unique::<'static>()
            // -> inspect(|x| println!("to_parent: {x:?}"))
            -> map(|v| serialize_to_bytes(v))
            -> dest_sink(to_parent); //send result to output channel

        to_query = merge();

        from_parent -> to_query;
        from_left -> to_query;
        from_right -> to_query;
        from_local -> to_query;

        to_query
            -> fold::<'tick>(0, update_value)
            -> fold::<'static>((0, 0), time_incrementer)
            -> unique::<'static>()
            // -> inspect(|x| println!("to_query: {x:?}"))
            -> map(|v| Bytes::from(serde_json::to_string(&v).unwrap()))
            -> dest_sink(query_responses); //send result to output channel
    };

    df.run_async().await.unwrap();
}
