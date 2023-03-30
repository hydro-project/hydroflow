use hydroflow::bytes::Bytes;
use hydroflow::hydroflow_syntax;
use hydroflow::tokio;
use hydroflow::util::cli::{ConnectedBidi, ConnectedSink, ConnectedSource};
use hydroflow::util::{deserialize_from_bytes, serialize_to_bytes};
use std::time::Instant;

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

    let my_merge_function = |((current_time, current_value), _ts), ((x, y), ts_new)| {
        if x > current_time {
            ((x, y), ts_new)
        } else {
            ((current_time, current_value), ts_new)
        }
    };

    // This function just passes through the value and increments the logical time if the value is different.
    let time_incrementer = |((current_time, current_value), _ts): ((i32, i32), Option<Instant>),
                            (new_value, ts_new)| {
        if new_value != current_value {
            ((current_time + 1, new_value), ts_new)
        } else {
            ((current_time, new_value), ts_new)
        }
    };

    // [A, B] -> (time here) map -> [A', B'] -> fold -> [C] -> map -> (time here) output
    //

    fn update_value(
        (current_value, _ts): (i32, Option<Instant>),
        (new_value, ts_new): (i32, Option<Instant>),
    ) -> (i32, Option<Instant>) {
        (current_value + new_value, ts_new)
    }

    let mut df = hydroflow_syntax! {
        from_parent = source_stream(from_parent)
            -> inspect(|_| println!("from_parent_now: {:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)))
            -> map(|x| (x, Some(Instant::now())))
            -> map(|(x, ts)| (deserialize_from_bytes::<(i32,i32)>(x.unwrap()), ts))
            // -> inspect(|x| println!("from_parent: {x:?}"))
            -> fold::<'static>(((0,0), None), my_merge_function)
            -> map(|((_current_time, current_value), ts)| (current_value, ts))
            -> tee();

        from_left = source_stream(from_left)
            -> inspect(|_| println!("from_left_now: {:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)))
            -> map(|x| (x, Some(Instant::now())))
            -> map(|(x, ts)| (deserialize_from_bytes::<(i32,i32)>(x.unwrap()), ts))
            // -> inspect(|x| println!("from_left: {x:?}"))
            -> fold::<'static>(((0,0), None), my_merge_function)
            -> map(|((_current_time, current_value), ts)| (current_value, ts))
            -> tee();

        from_right = source_stream(from_right)
            -> inspect(|_| println!("from_right_now: {:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)))
            -> map(|x| (x, Some(Instant::now())))
            -> map(|(x, ts)| (deserialize_from_bytes::<(i32,i32)>(x.unwrap()), ts))
            // -> inspect(|x| println!("from_right: {x:?}"))
            -> fold::<'static>(((0,0), None), my_merge_function)
            -> map(|((_current_time, current_value), ts)| (current_value, ts))
            -> tee();

        from_local = source_stream(increment_requests) //TODO implement
            -> inspect(|_| println!("from_local_now: {:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)))
            -> map(|x| (x, Some(Instant::now())))
            -> map(|(x, ts)| (String::from_utf8(x.unwrap().to_vec()).unwrap(), ts))
            -> map(|(x, ts)| (serde_json::from_str::<u32>(&x).unwrap(), ts))
            -> fold::<'static>(((0,0), None), |((time, value), _), (req, ts)| {
                ((time + 1, value + (req as i32)), ts)
            })
            -> fold::<'static>(((0,0), None), my_merge_function)
            -> map(|((_current_time, current_value), ts)| (current_value, ts))
            -> tee();

        to_right = merge();

        from_parent -> to_right;
        from_left -> to_right;
        from_local -> to_right;

        to_right
            -> fold::<'tick>((0, None), update_value) // This is just adding from_parent, from_left, from_local in one tick, that's what it's 'tick.
            -> fold::<'static>(((0, 0), None), time_incrementer) // This is persisting the logical timestamp, and incrementing it, that's why it's 'static.
            // -> unique::<'static>()
            // -> inspect(|x| println!("to_right: {x:?}"))
            -> map(|(v, ts)| (serialize_to_bytes(v), ts))
            -> map(|(v, ts)| {
                println!("to_right: {:?}", ts.map(|x| x.elapsed()));
                v
            })
            ->inspect(|x| println!("to_right bytes: {:?}", x.len())) //Measure the byte size of messages sent over the network
            -> dest_sink(to_right); //send result to output channel

        to_left = merge();

        from_parent -> to_left;
        from_right -> to_left;
        from_local -> to_left;

        to_left
            -> fold::<'tick>((0, None), update_value)
            -> fold::<'static>(((0, 0), None), time_incrementer)
            // -> unique::<'static>()
            // -> inspect(|x| println!("to_left: {x:?}"))
            -> map(|(v, ts)| (serialize_to_bytes(v), ts))
            -> map(|(v, ts)| {
                println!("to_left: {:?}", ts.map(|x| x.elapsed()));
                v
            })
            ->inspect(|x| println!("to_left bytes: {:?}", x.len())) //Measure the byte size of messages sent over the network
            -> dest_sink(to_left); //send result to output channel

        to_parent = merge();

        from_right -> to_parent;
        from_left -> to_parent;
        from_local -> to_parent;

        to_parent
            -> fold::<'tick>((0, None), update_value)
            -> fold::<'static>(((0, 0), None), time_incrementer)
            // -> unique::<'static>()
            // -> inspect(|x| println!("to_parent: {x:?}"))
            -> map(|(v, ts)| (serialize_to_bytes(v), ts))
            -> map(|(v, ts)| {
                println!("to_parent: {:?}", ts.map(|x| x.elapsed()));
                v
            })
            ->inspect(|x| println!("to_parent bytes: {:?}", x.len())) //Measure the byte size of messages sent over the network
            -> dest_sink(to_parent); //send result to output channel

        to_query = merge();

        from_parent -> to_query;
        from_left -> to_query;
        from_right -> to_query;
        from_local -> to_query;

        to_query
            -> fold::<'tick>((0, None), update_value)
            -> fold::<'static>(((0, 0), None), time_incrementer)
            // -> unique::<'static>()
            // -> inspect(|x| println!("to_query: {x:?}"))
            -> map(|(v, ts)| (Bytes::from(serde_json::to_string(&v).unwrap()), ts))
            -> map(|(v, ts)| {
                println!("to_query: {:?}", ts.map(|x| x.elapsed()));
                v
            })
            -> dest_sink(query_responses); //send result to output channel
    };

    df.run_async().await.unwrap();
}
