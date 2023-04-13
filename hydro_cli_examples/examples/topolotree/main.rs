use hydroflow::bytes::Bytes;
use hydroflow::hydroflow_syntax;
use hydroflow::serde::Deserialize;
use hydroflow::serde::Serialize;
use hydroflow::tokio;
use hydroflow::util::cli::{ConnectedBidi, ConnectedSink, ConnectedSource};
use hydroflow::util::{deserialize_from_bytes, serialize_to_bytes};
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::time::Duration;
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct IncrementRequest {
    tweet_id: u64,
    likes: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TimestampedValue<T> {
    pub value: T,
    pub timestamp: u32,
}

impl<T> PartialEq for TimestampedValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl<T> Eq for TimestampedValue<T> {}

impl<T> Hash for TimestampedValue<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.timestamp.hash(state);
    }
}

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let increment_requests = ports
        .remove("increment_requests")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

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

    let f1 = async {
        #[cfg(target_os = "linux")]
        loop {
            let x = procinfo::pid::stat_self().unwrap();
            println!("memory: {} bytes", x.rss * 1024 * 4);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    fn my_merge_function(
        (cur, _): (TimestampedValue<MyType>, Option<Instant>),
        (other, ts_new): (TimestampedValue<MyType>, Option<Instant>),
    ) -> (TimestampedValue<MyType>, Option<Instant>) {
        if other.timestamp > cur.timestamp {
            (
                TimestampedValue {
                    value: other.value,
                    timestamp: other.timestamp,
                },
                ts_new,
            )
        } else {
            (cur, ts_new)
        }
    }

    fn time_incrementer(
        (cur, _ts): (TimestampedValue<MyType>, Option<Instant>),
        (new_value, ts_new): (MyType, Option<Instant>),
    ) -> (TimestampedValue<MyType>, Option<Instant>) {
        if new_value != cur.value {
            (
                TimestampedValue {
                    value: new_value,
                    timestamp: cur.timestamp + 1,
                },
                ts_new,
            )
        } else {
            (cur, ts_new)
        }
    }

    fn binary_op_twitter((key, like): (u64, i32), mut map: HashMap<u64, i32>) -> HashMap<u64, i32> {
        *map.entry(key).or_default() += like;
        map
    }

    fn combine_values(
        (mut current_value, _ts): (MyType, Option<Instant>),
        (new_value, ts_new): (MyType, Option<Instant>),
    ) -> (MyType, Option<Instant>) {
        for (k, v) in new_value {
            *current_value.entry(k).or_insert(0) += v;
        }

        (current_value, ts_new)
    }

    type UpdateType = (u64, i32);
    type MyType = HashMap<u64, i32>;

    let df = hydroflow_syntax! {
        from_parent = source_stream(from_parent)
            -> inspect(|_| println!("from_parent_now: {:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)))
            -> map(|x| (x, Some(Instant::now())))
            -> map(|(x, ts)| (deserialize_from_bytes::<TimestampedValue<MyType>>(x.unwrap()).unwrap(), ts))
            // -> inspect(|x| println!("from_parent: {x:?}"))
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, None), my_merge_function)
            -> map(|(cur, ts)| (cur.value, ts))
            -> tee();

        from_left = source_stream(from_left)
            -> inspect(|_| println!("from_left_now: {:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)))
            -> map(|x| (x, Some(Instant::now())))
            -> map(|(x, ts)| (deserialize_from_bytes::<TimestampedValue<MyType>>(x.unwrap()).unwrap(), ts))
            // -> inspect(|x| println!("froM_left: {x:?}"))
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, None), my_merge_function)
            -> map(|(cur, ts)| (cur.value, ts))
            -> tee();

        from_right = source_stream(from_right)
            -> inspect(|_| println!("from_right_now: {:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)))
            -> map(|x| (x, Some(Instant::now())))
            -> map(|(x, ts)| (deserialize_from_bytes::<TimestampedValue<MyType>>(x.unwrap()).unwrap(), ts))
            // -> inspect(|x| println!("from_right: {x:?}"))
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, None), my_merge_function)
            -> map(|(cur, ts)| (cur.value, ts))
            -> tee();

        from_local = source_stream(increment_requests) //TODO implement
            -> inspect(|_| println!("from_local_now: {:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)))
            -> map(|x| (x, Some(Instant::now())))
            // -> inspect(|x| println!("debug_from_local: {x:?}"))
            -> map(|(x, ts)| (String::from_utf8(x.unwrap().to_vec()).unwrap(), ts))
            -> map(|(x, ts)| (serde_json::from_str::<IncrementRequest>(&x).unwrap(), ts))
            -> map(|(x, ts)| ((x.tweet_id, x.likes), ts))
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, None), |(prev, _): (TimestampedValue<MyType>, Option<Instant>), (req, ts): (UpdateType, Option<Instant>)| {
                (TimestampedValue {
                    value: binary_op_twitter(req, prev.value),
                    timestamp: prev.timestamp + 1,
                }, ts)
            })
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, None), my_merge_function)
            -> map(|(cur, ts)| (cur.value, ts))
            -> tee();

        to_right = merge();

        from_parent -> to_right;
        from_left -> to_right;
        from_local -> to_right;

        to_right
            -> fold::<'tick>((MyType::default(), None), combine_values) // This is just adding from_parent, from_left, from_local in one tick, that's what it's 'tick.
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, None), time_incrementer) // This is persisting the logical timestamp, and incrementing it, that's why it's 'static.
            -> unique::<'static>()
            // -> inspect(|x| println!("to_right: {x:?}"))
            -> map(|(v, ts)| (serialize_to_bytes(v), ts))
            -> map(|(v, ts)| {
                println!("to_right: {:?}", ts.map(|x| x.elapsed()));
                v
            })
            -> inspect(|x| println!("to_right bytes: {:?}", x.len())) //Measure the byte size of messages sent over the network
            -> dest_sink(to_right); //send result to output channel

        to_left = merge();

        from_parent -> to_left;
        from_right -> to_left;
        from_local -> to_left;

        to_left
            -> fold::<'tick>((MyType::default(), None), combine_values)
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, None), time_incrementer)
            -> unique::<'static>()
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
            -> fold::<'tick>((MyType::default(), None), combine_values)
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, None), time_incrementer)
            -> unique::<'static>()
            // -> inspect(|x| println!("to_parent: {x:?}"))
            -> map(|(v, ts)| (serialize_to_bytes(v), ts))
            -> map(|(v, ts)| {
                println!("to_parent: {:?}", ts.map(|x| x.elapsed()));
                v
            })
            -> inspect(|x| println!("to_parent bytes: {:?}", x.len())) //Measure the byte size of messages sent over the network
            -> dest_sink(to_parent); //send result to output channel

        to_query = merge();

        from_parent -> to_query;
        from_left -> to_query;
        from_right -> to_query;
        from_local -> to_query;

        to_query
            -> fold::<'tick>((MyType::default(), None), combine_values)
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, None), time_incrementer)
            -> unique::<'static>()
            // -> inspect(|x| println!("to_query: {x:?}"))
            -> map(|(v, ts)| (Bytes::from(serde_json::to_string(&v).unwrap()), ts))
            -> map(|(v, ts)| {
                println!("to_query: {:?}", ts.map(|x| x.elapsed()));
                v
            })
            -> dest_sink(query_responses); //send result to output channel
    };

    let f1_handle = tokio::spawn(f1);
    hydroflow::util::cli::launch_flow(df).await;
    f1_handle.abort();
}
