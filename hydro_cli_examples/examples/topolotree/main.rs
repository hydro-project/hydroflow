use futures::SinkExt;
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

    let mut memory_report = ports
        .remove("memory_report")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_sink();

    let f1 = async move {
        #[cfg(target_os = "linux")]
        loop {
            let x = procinfo::pid::stat_self().unwrap();
            let bytes = x.rss * 1024 * 4;
            memory_report
                .send(Bytes::from(serde_json::to_string(&bytes).unwrap()))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    fn my_merge_function(
        cur: TimestampedValue<MyType>,
        other: TimestampedValue<MyType>,
    ) -> TimestampedValue<MyType> {
        if other.timestamp > cur.timestamp {
            TimestampedValue {
                value: other.value,
                timestamp: other.timestamp,
            }
        } else {
            cur
        }
    }

    fn time_incrementer(
        cur: TimestampedValue<MyType>,
        new_value: MyType,
    ) -> TimestampedValue<MyType> {
        if new_value != cur.value {
            TimestampedValue {
                value: new_value,
                timestamp: cur.timestamp + 1,
            }
        } else {
            cur
        }
    }

    fn binary_op_twitter((key, like): (u64, i32), mut map: HashMap<u64, i32>) -> HashMap<u64, i32> {
        *map.entry(key).or_default() += like;
        map
    }

    fn combine_values(
        (mut a_value, a_tick): (MyType, usize),
        (b_value, b_tick): (MyType, usize),
    ) -> (MyType, usize) {
        for (k, v) in b_value {
            *a_value.entry(k).or_insert(0) += v;
        }

        (a_value, std::cmp::max(a_tick, b_tick))
    }

    type UpdateType = (u64, i32);
    type MyType = HashMap<u64, i32>;

    let df = hydroflow_syntax! {
        from_parent = source_stream(from_parent)
            -> map(|x| deserialize_from_bytes::<TimestampedValue<MyType>>(x.unwrap()).unwrap())
            -> fold::<'static>(
                (TimestampedValue {
                    value: MyType::default(),
                    timestamp: 0
                }, 0),
                |(v, tick), merge| (my_merge_function(v, merge), context.current_tick())
            )
            -> map(|(v, tick)| (v.value, tick))
            -> tee();

        from_left = source_stream(from_left)
            -> map(|x| deserialize_from_bytes::<TimestampedValue<MyType>>(x.unwrap()).unwrap())
            -> fold::<'static>(
                (TimestampedValue {
                    value: MyType::default(),
                    timestamp: 0
                }, 0),
                |(v, tick), merge| (my_merge_function(v, merge), context.current_tick())
            )
            -> map(|(v, tick)| (v.value, tick))
            -> tee();

        from_right = source_stream(from_right)
            -> map(|x| deserialize_from_bytes::<TimestampedValue<MyType>>(x.unwrap()).unwrap())
            -> fold::<'static>(
                (TimestampedValue {
                    value: MyType::default(),
                    timestamp: 0
                }, 0),
                |(v, tick), merge| (my_merge_function(v, merge), context.current_tick())
            )
            -> map(|(v, tick)| (v.value, tick))
            -> tee();

        from_local = source_stream(increment_requests)
            -> map(|x| serde_json::from_str::<IncrementRequest>(&String::from_utf8_lossy(&x.unwrap())).unwrap())
            -> map(|x| (x.tweet_id, x.likes))
            -> fold::<'static>(
                (MyType::default(), 0),
                |prev, req: UpdateType| {
                    (binary_op_twitter(req, prev.0), context.current_tick())
                }
            )
            -> tee();

        to_right = merge();

        from_parent -> to_right;
        from_left -> to_right;
        from_local -> to_right;

        to_right
            -> fold::<'tick>((MyType::default(), 0), combine_values) // This is just adding from_parent, from_left, from_local in one tick, that's what it's 'tick.
            -> filter(|t| t.1 == context.current_tick()) // only produce a value if there was an update this tick
            -> map(|t| t.0)
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, 0), |(v, tick), merge| (time_incrementer(v, merge), context.current_tick())) // This is persisting the logical timestamp, and incrementing it, that's why it's 'static.
            -> filter(|t| t.1 == context.current_tick())
            -> map(|t| t.0)
            -> map(|v| serialize_to_bytes(v))
            -> dest_sink(to_right);

        to_left = merge();

        from_parent -> to_left;
        from_right -> to_left;
        from_local -> to_left;

        to_left
            -> fold::<'tick>((MyType::default(), 0), combine_values)
            -> filter(|t| t.1 == context.current_tick())
            -> map(|t| t.0)
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, 0), |(v, tick), merge| (time_incrementer(v, merge), context.current_tick()))
            -> filter(|t| t.1 == context.current_tick())
            -> map(|t| t.0)
            -> map(|v| serialize_to_bytes(v))
            -> dest_sink(to_left); //send result to output channel

        to_parent = merge();

        from_right -> to_parent;
        from_left -> to_parent;
        from_local -> to_parent;

        to_parent
            -> fold::<'tick>((MyType::default(), 0), combine_values)
            -> filter(|t| t.1 == context.current_tick())
            -> map(|t| t.0)
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, 0), |(v, tick), merge| (time_incrementer(v, merge), context.current_tick()))
            -> filter(|t| t.1 == context.current_tick())
            -> map(|t| t.0)
            -> map(|v| serialize_to_bytes(v))
            -> dest_sink(to_parent); //send result to output channel

        to_query = merge();

        from_parent -> to_query;
        from_left -> to_query;
        from_right -> to_query;
        from_local -> to_query;

        to_query
            -> fold::<'tick>((MyType::default(), 0), combine_values)
            -> filter(|t| t.1 == context.current_tick())
            -> map(|t| t.0)
            -> fold::<'static>((TimestampedValue {
                value: MyType::default(),
                timestamp: 0,
            }, 0), |(v, tick), merge| (time_incrementer(v, merge), context.current_tick()))
            -> filter(|t| t.1 == context.current_tick())
            -> map(|t| t.0)
            -> map(|v| Bytes::from(serde_json::to_string(&v).unwrap()))
            -> dest_sink(query_responses); //send result to output channel
    };

    let f1_handle = tokio::spawn(f1);
    hydroflow::util::cli::launch_flow(df).await;
    f1_handle.abort();
}
