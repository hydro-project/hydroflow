---
sidebar_position: 4
---

# Hydroflow's Operators

In our previous examples we made use of some of Hydroflow's operators.
Here we document each operator in more detail. Most of these operators
are based on the Rust equivalents for iterators; see the [Rust documentation](https://doc.rust-lang.org/std/iter/trait.Iterator.html).
<!-- GENERATED "hydroflow_macro/build.rs" -->

| All Operators | | | |
| --- | --- | --- | --- |
| [`anti_join`](#anti_join) | [`batch`](#batch) | [`cross_join`](#cross_join) | [`demux`](#demux) |
| [`dest_file`](#dest_file) | [`dest_sink`](#dest_sink) | [`dest_sink_serde`](#dest_sink_serde) | [`difference`](#difference) |
| [`enumerate`](#enumerate) | [`filter`](#filter) | [`filter_map`](#filter_map) | [`flat_map`](#flat_map) |
| [`flatten`](#flatten) | [`fold`](#fold) | [`for_each`](#for_each) | [`group_by`](#group_by) |
| [`identity`](#identity) | [`initialize`](#initialize) | [`inspect`](#inspect) | [`join`](#join) |
| [`keyed_fold`](#keyed_fold) | [`keyed_reduce`](#keyed_reduce) | [`lattice_batch`](#lattice_batch) | [`lattice_join`](#lattice_join) |
| [`lattice_merge`](#lattice_merge) | [`map`](#map) | [`merge`](#merge) | [`next_stratum`](#next_stratum) |
| [`next_tick`](#next_tick) | [`null`](#null) | [`persist`](#persist) | [`reduce`](#reduce) |
| [`repeat_fn`](#repeat_fn) | [`repeat_iter`](#repeat_iter) | [`repeat_iter_external`](#repeat_iter_external) | [`sort`](#sort) |
| [`sort_by`](#sort_by) | [`source_file`](#source_file) | [`source_interval`](#source_interval) | [`source_iter`](#source_iter) |
| [`source_json`](#source_json) | [`source_stdin`](#source_stdin) | [`source_stream`](#source_stream) | [`source_stream_serde`](#source_stream_serde) |
| [`tee`](#tee) | [`unique`](#unique) | [`unzip`](#unzip) |

## `anti_join`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 2">exactly 2</span> | `-> [<input_port>]anti_join() ->` | <span title="exactly 1">exactly 1</span> |Blocking |

> Input port names: `pos` (streaming), `neg` (blocking)  

<!-- GENERATED "hydroflow_lang/src/graph/ops/anti_join.rs" -->
> 2 input streams the first of type (K, T), the second of type K,
> with output type (K, T)

For a given tick, computes the anti-join of the items in the input
streams, returning items in the `pos` input that do not have matching keys
in the `neg` input.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print ("elephant", 3)
source_iter(vec![("dog", 1), ("cat", 2), ("elephant", 3)]) -> [pos]diff;
source_iter(vec!["dog", "cat", "gorilla"]) -> [neg]diff;
diff = anti_join() -> for_each(|v: (_, _)| println!("{}, {}", v.0, v.1));
// elephant 3
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `batch`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> batch(A, B) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/batch.rs" -->
> 1 input stream, 1 output stream

> Arguments: First argument is the maximum batch size that batch() will buffer up before completely releasing the batch.
 The second argument is the receive end of a tokio channel that signals when to release the batch downstream.

Given a [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
created in Rust code, `batch`
is passed the receive end of the channel and when receiving any element
will pass through all received inputs to the output unchanged.

```rust
    let (tx, rx) = hydroflow::util::unbounded_channel::<()>();

    // Will print 0, 1, 2, 3, 4 each on a new line just once.
    let mut df = hydroflow::hydroflow_syntax! {
        repeat_iter(0..5) -> batch(10, rx) -> for_each(|x| { println!("{x}"); });
    };

    tx.send(()).unwrap();

    df.run_available();
```


## `cross_join`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 2">exactly 2</span> | `-> [<input_port>]cross_join() ->` | <span title="exactly 1">exactly 1</span> |Streaming |

> Input port names: `0` (streaming), `1` (streaming)  

<!-- GENERATED "hydroflow_lang/src/graph/ops/cross_join.rs" -->
> 2 input streams of type S and T, 1 output stream of type (S, T)

Forms the cross-join (Cartesian product) of the items in the input streams, returning all
tupled pairs.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print all 4 pairs of emotion and animal
source_iter(vec!["happy", "sad"]) -> [0]my_join;
source_iter(vec!["dog", "cat"]) -> [1]my_join;
my_join = cross_join() -> for_each(|(v1, v2)| println!("({}, {})", v1, v2));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```

`cross_join` can also be provided with one or two generic lifetime persistence arguments
in the same was as [`join`](#join), see [`join`'s documentation](#join) for more info.

`cross_join` also accepts one type argument that controls how the join state is built up. This (currently) allows switching between a SetUnion and NonSetUnion implementation.
For example:
```hydroflow,ignore
join::<HalfSetJoinState>();
join::<HalfMultisetJoinState>();
```

```rust
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<&str>();
let mut flow = hydroflow::hydroflow_syntax! {
    my_join = cross_join::<'tick>();
    source_iter(["hello", "bye"]) -> [0]my_join;
    source_stream(input_recv) -> [1]my_join;
    my_join -> for_each(|(s, t)| println!("({}, {})", s, t));
};
input_send.send("oakland").unwrap();
flow.run_tick();
input_send.send("san francisco").unwrap();
flow.run_tick();
```
Prints only `"(hello, oakland)"` and `"(bye, oakland)"`. The `source_iter` is only included in
the first tick, then forgotten.


## `demux`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> demux(A)[<output_port>] ->` | <span title="at least 2">at least 2</span> |Streaming |

> Output port names: Variadic, as specified in arguments.

<!-- GENERATED "hydroflow_lang/src/graph/ops/demux.rs" -->
> Arguments: A Rust closure, the first argument is a received item and the
> second argument is a variadic [`var_args!` tuple list](https://hydro-project.github.io/hydroflow/doc/hydroflow/macro.var_args.html)
> where each item name is an output port.

Takes the input stream and allows the user to determine what elemnt(s) to
deliver to any number of output streams.

> Note: Downstream operators may need explicit type annotations.

> Note: The [`Pusherator`](https://hydro-project.github.io/hydroflow/doc/pusherator/trait.Pusherator.html)
> trait is automatically imported to enable the [`.give(...)` method](https://hydro-project.github.io/hydroflow/doc/pusherator/trait.Pusherator.html#tymethod.give).

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
my_demux = source_iter(1..=100) -> demux(|v, var_args!(fzbz, fizz, buzz, vals)|
    match (v % 3, v % 5) {
        (0, 0) => fzbz.give(v),
        (0, _) => fizz.give(v),
        (_, 0) => buzz.give(v),
        (_, _) => vals.give(v),
    }
);
my_demux[fzbz] -> for_each(|v| println!("{}: fizzbuzz", v));
my_demux[fizz] -> for_each(|v| println!("{}: fizz", v));
my_demux[buzz] -> for_each(|v| println!("{}: buzz", v));
my_demux[vals] -> for_each(|v| println!("{}", v));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `dest_file`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> dest_file(A, B)` | <span title="exactly 0">exactly 0</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/dest_file.rs" -->
> 0 input streams, 1 output stream

> Arguments: (1) An [`AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)`<`[`Path`](https://doc.rust-lang.org/nightly/std/path/struct.Path.html)`>`
for a file to write to, and (2) a bool `append`.

Consumes `String`s by writing them as lines to a file. The file will be created if it doesn't
exist. Lines will be appended to the file if `append` is true, otherwise the file will be
truncated before lines are written.

Note this operator must be used within a Tokio runtime.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter(1..=10) -> map(|n| format!("Line {}", n)) -> dest_file("dest.txt", false);
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `dest_sink`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> dest_sink(A)` | <span title="exactly 0">exactly 0</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/dest_sink.rs" -->
> Arguments: An [async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).

Consumes items by sending them to an [async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).
A `Sink` is a thing into which values can be sent, asynchronously. For example, sending items
into a bounded channel.

Note this operator must be used within a Tokio runtime.

```rust
# #[tokio::main(flavor = "current_thread")]
# async fn main() {
// In this example we use a _bounded_ channel for our `Sink`. This is for demonstration only,
// instead you should use [`hydroflow::util::unbounded_channel`]. A bounded channel results in
// `Hydroflow` buffering items internally instead of within the channel. (We can't use
// unbounded here since unbounded channels are synchonous to write to and therefore not
// `Sink`s.)
let (send, recv) = tokio::sync::mpsc::channel::<usize>(5);
// `PollSender` adapts the send half of the bounded channel into a `Sink`.
let send = tokio_util::sync::PollSender::new(send);

let mut flow = hydroflow::hydroflow_syntax! {
    source_iter(0..10) -> dest_sink(send);
};
// Call `run_async()` to allow async events to propegate, run for one second.
tokio::time::timeout(std::time::Duration::from_secs(1), flow.run_async())
    .await
    .expect_err("Expected time out");

let mut recv = tokio_stream::wrappers::ReceiverStream::new(recv);
// Only 5 elements received due to buffer size.
// (Note that if we were using a multi-threaded executor instead of `current_thread` it would
// be possible for more items to be added as they're removed, resulting in >5 collected.)
let out: Vec<_> = hydroflow::util::ready_iter(&mut recv).collect();
assert_eq!(&[0, 1, 2, 3, 4], &*out);
# }
```

`Sink` is different from [`AsyncWrite`](https://docs.rs/futures/latest/futures/io/trait.AsyncWrite.html).
Instead of discrete values we send arbitrary streams of bytes into an `AsyncWrite` value. For
example, writings a stream of bytes to a file, a socket, or stdout.

To handle those situations we can use a codec from [`tokio_util::codec`](crate::tokio_util::codec).
These specify ways in which the byte stream is broken into individual items, such as with
newlines or with length delineation.

If we only want to write a stream of bytes without delineation we can use the [`BytesCodec`](crate::tokio_util::codec::BytesCodec).

In this example we use a [`duplex`](crate::tokio::io::duplex) as our `AsyncWrite` with a
`BytesCodec`.

```rust
# #[tokio::main]
# async fn main() {
use bytes::Bytes;
use tokio::io::AsyncReadExt;

// Like a channel, but for a stream of bytes instead of discrete objects.
let (asyncwrite, mut asyncread) = tokio::io::duplex(256);
// Now instead handle discrete byte strings by length-encoding them.
let sink = tokio_util::codec::FramedWrite::new(asyncwrite, tokio_util::codec::BytesCodec::new());

let mut flow = hydroflow::hydroflow_syntax! {
    source_iter([
        Bytes::from_static(b"hello"),
        Bytes::from_static(b"world"),
    ]) -> dest_sink(sink);
};
tokio::time::timeout(std::time::Duration::from_secs(1), flow.run_async())
    .await
    .expect_err("Expected time out");

let mut buf = Vec::<u8>::new();
asyncread.read_buf(&mut buf).await.unwrap();
assert_eq!(b"helloworld", &*buf);
# }
```


## `dest_sink_serde`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> dest_sink_serde(A)` | <span title="exactly 0">exactly 0</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/dest_sink_serde.rs" -->
> Arguments: A [serializing async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).

Consumes (payload, addr) pairs by serializing the payload and sending the resulting pair to an [async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).

Note this operator must be used within a Tokio runtime.
```rust
async fn serde_out() {
    let addr = hydroflow::util::ipv4_resolve("localhost:9000".into()).unwrap();
    let (outbound, inbound, _) = hydroflow::util::bind_udp_bytes(addr).await;
    let remote = hydroflow::util::ipv4_resolve("localhost:9001".into()).unwrap();
    let mut flow = hydroflow::hydroflow_syntax! {
        source_iter(vec![("hello".to_string(), 1), ("world".to_string(), 2)])
            -> map (|m| (m, remote)) -> dest_sink_serde(outbound);
    };
    flow.run_available();
}
```


## `difference`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 2">exactly 2</span> | `-> [<input_port>]difference() ->` | <span title="exactly 1">exactly 1</span> |Blocking |

> Input port names: `pos` (streaming), `neg` (blocking)  

<!-- GENERATED "hydroflow_lang/src/graph/ops/difference.rs" -->
> 2 input streams of the same type T, 1 output stream of type T

For a given tick, forms the set difference of the items in the input
streams, returning items in the `pos` input that are not found in the
`neg` input.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print "elephant"
source_iter(vec!["dog", "cat", "elephant"]) -> [pos]diff;
source_iter(vec!["dog", "cat", "gorilla"]) -> [neg]diff;
diff = difference() -> for_each(|v| println!("{}", v));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `enumerate`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> enumerate() ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/enumerate.rs" -->
> 1 input stream of type `T`, 1 output stream of type `(usize, T)`

For each item passed in, enumerate it with its index: `(0, x_0)`, `(1, x_1)`, etc.

`enumerate` can also be provided with one generic lifetime persistence argument, either
`'tick` or `'static`, to specify if indexing resets. If `'tick` is specified, indexing will
restart at zero at the start of each tick. Otherwise `'static` (the default) will never reset
and count monotonically upwards.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter(vec!["hello", "world"]) -> enumerate()
    -> for_each(|(i, x)| println!("{}: {}", i, x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `filter`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> filter(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/filter.rs" -->
Filter outputs a subsequence of the items it receives at its input, according to a
Rust boolean closure passed in as an argument.

The closure receives a reference `&T` rather than an owned value `T` because filtering does
not modify or take ownership of the values. If you need to modify the values while filtering
use [`filter_map`](#filter_map) instead.

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter(vec!["hello", "world"]) -> filter(|x| x.starts_with('w'))
    -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `filter_map`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> filter_map(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/filter_map.rs" -->
> 1 input stream, 1 output stream

An operator that both filters and maps. It yields only the items for which the supplied closure returns `Some(value)`.

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter(vec!["1", "hello", "world", "2"]) -> filter_map(|s| s.parse().ok())
    -> for_each(|x: usize| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `flat_map`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> flat_map(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/flat_map.rs" -->
> 1 input stream, 1 output stream

> Arguments: A Rust closure that handles an iterator

For each item `i` passed in, treat `i` as an iterator and map the closure to that
iterator to produce items one by one. The type of the input items must be iterable.

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print out each character of each word on a separate line
source_iter(vec!["hello", "world"]) -> flat_map(|x| x.chars())
    -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `flatten`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> flatten() ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/flatten.rs" -->
> 1 input stream, 1 output stream

For each item `i` passed in, treat `i` as an iterator and produce its items one by one.
The type of the input items must be iterable.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print the numbers 1-6 without any nesting
source_iter(vec![[1, 2], [3, 4], [5, 6]]) -> flatten()
-> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `fold`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> fold(A, B) ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/fold.rs" -->
> 1 input stream, 1 output stream

> Arguments: an initial value, and a closure which itself takes two arguments:
an 'accumulator', and an element. The closure returns the value that the accumulator should have for the next iteration.

Akin to Rust's built-in fold operator. Folds every element into an accumulator by applying a closure,
returning the final result.

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

`fold` can also be provided with one generic lifetime persistence argument, either
`'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
within the same tick. With `'static`, values will be remembered across ticks and will be
aggregated with pairs arriving in later ticks. When not explicitly specified persistence
defaults to `'static`.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print `Reassembled vector [1,2,3,4,5]`
source_iter([1,2,3,4,5])
    -> fold::<'tick>(Vec::new(), |mut accum, elem| {
        accum.push(elem);
        accum
    })
    -> for_each(|e| println!("Ressembled vector {:?}", e));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `for_each`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> for_each(A)` | <span title="exactly 0">exactly 0</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/for_each.rs" -->
> 1 input stream, 0 output streams

> Arguments: a Rust closure

Iterates through a stream passing each element to the closure in the
argument.

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
    source_iter(vec!["Hello", "World"])
        -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `group_by`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> group_by(A, B) ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/group_by.rs" -->
An alias for [`keyed_fold`](#keyed_fold).


## `identity`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> identity() ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/identity.rs" -->
> 1 input stream of type T, 1 output stream of type T

For each item passed in, pass it out without any change.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print "hello" and "world" on separate lines (in either order)
source_iter(vec!["hello", "world"]) -> identity()
    -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```

You can also supply a type parameter `identity::<MyType>()` to specify what items flow thru the
the pipeline. This can be useful for helping the compiler infer types.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// Use type parameter to ensure items are `i32`s.
source_iter(0..10) -> identity::<i32>() -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `initialize`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `initialize() ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/initialize.rs" -->
> 0 input streams, 1 output stream

> Arguments: None.

Emits a single unit `()` at the start of the first tick.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
initialize() -> for_each(|()| println!("This only runs one time!"));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `inspect`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> inspect(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/inspect.rs" -->
> Arguments: A single closure `FnMut(&Item)`.

An operator which allows you to "inspect" each element of a stream without
modifying it. The closure is called on a reference to each item. This is
mainly useful for debugging as in the example below, and it is generally an
anti-pattern to provide a closure with side effects.

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter([1, 2, 3, 4]) -> inspect(|&x| println!("{}", x)) -> null();
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `join`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 2">exactly 2</span> | `-> [<input_port>]join() ->` | <span title="exactly 1">exactly 1</span> |Streaming |

> Input port names: `0` (streaming), `1` (streaming)  

<!-- GENERATED "hydroflow_lang/src/graph/ops/join.rs" -->
> 2 input streams of type <(K, V1)> and <(K, V2)>, 1 output stream of type <(K, (V1, V2))>

Forms the equijoin of the tuples in the input streams by their first (key) attribute. Note that the result nests the 2nd input field (values) into a tuple in the 2nd output field.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print `(hello, (world, cleveland))`
source_iter(vec![("hello", "world"), ("stay", "gold")]) -> [0]my_join;
source_iter(vec![("hello", "cleveland")]) -> [1]my_join;
my_join = join() -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```

`join` can also be provided with one or two generic lifetime persistence arguments, either
`'tick` or `'static`, to specify how join data persists. With `'tick`, pairs will only be
joined with corresponding pairs within the same tick. With `'static`, pairs will be remembered
across ticks and will be joined with pairs arriving in later ticks. When not explicitly
specified persistence defaults to `static.

When two persistence arguments are supplied the first maps to port `0` and the second maps to
port `1`.
When a single persistence argument is supplied, it is applied to both input ports.
When no persistence arguments are applied it defaults to `'static` for both.

The syntax is as follows:
```hydroflow,ignore
join(); // Or
join::<'static>();

join::<'tick>();

join::<'static, 'tick>();

join::<'tick, 'static>();
// etc.
```

Join also accepts one type argument that controls how the join state is built up. This (currently) allows switching between a SetUnion and NonSetUnion implementation.
For example:
```hydroflow,ignore
join::<HalfSetJoinState>();
join::<HalfMultisetJoinState>();
```

### Examples

```rust
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
let mut flow = hydroflow::hydroflow_syntax! {
    source_iter([("hello", "world")]) -> [0]my_join;
    source_stream(input_recv) -> [1]my_join;
    my_join = join::<'tick>() -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
};
input_send.send(("hello", "oakland")).unwrap();
flow.run_tick();
input_send.send(("hello", "san francisco")).unwrap();
flow.run_tick();
```
Prints out `"(hello, (world, oakland))"` since `source_iter([("hello", "world")])` is only
included in the first tick, then forgotten.

---

```rust
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
let mut flow = hydroflow::hydroflow_syntax! {
    source_iter([("hello", "world")]) -> [0]my_join;
    source_stream(input_recv) -> [1]my_join;
    my_join = join::<'static>() -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
};
input_send.send(("hello", "oakland")).unwrap();
flow.run_tick();
input_send.send(("hello", "san francisco")).unwrap();
flow.run_tick();
```
Prints out `"(hello, (world, oakland))"` and `"(hello, (world, san francisco))"` since the
inputs are peristed across ticks.


## `keyed_fold`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> keyed_fold(A, B) ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/keyed_fold.rs" -->
> 1 input stream of type `(K, V1)`, 1 output stream of type `(K, V2)`.
The output will have one tuple for each distinct `K`, with an accumulated value of type `V2`.

If the input and output value types are the same and do not require initialization then use
[`keyed_reduce`](#keyed_reduce).

> Arguments: two Rust closures. The first generates an initial value per group. The second
itself takes two arguments: an 'accumulator', and an element. The second closure returns the
value that the accumulator should have for the next iteration.

A special case of `fold`, in the spirit of SQL's GROUP BY and aggregation constructs. The input
is partitioned into groups by the first field, and for each group the values in the second
field are accumulated via the closures in the arguments.

> Note: The closures have access to the [`context` object](surface_flows.md#the-context-object).

`keyed_fold` can also be provided with one generic lifetime persistence argument, either
`'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
within the same tick. With `'static`, values will be remembered across ticks and will be
aggregated with pairs arriving in later ticks. When not explicitly specified persistence
defaults to `'static`.

`keyed_fold` can also be provided with two type arguments, the key type `K` and aggregated
output value type `V2`. This is required when using `'static` persistence if the compiler
cannot infer the types.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter([("toy", 1), ("toy", 2), ("shoe", 11), ("shoe", 35), ("haberdashery", 7)])
    -> keyed_fold(|| 0, |old: &mut u32, val: u32| *old += val)
    -> for_each(|(k, v)| println!("Total for group {} is {}", k, v));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```

Example using `'tick` persistence:
```rust
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
let mut flow = hydroflow::hydroflow_syntax! {
    source_stream(input_recv)
        -> keyed_fold::<'tick, &str, String>(String::new, |old: &mut _, val| {
            *old += val;
            *old += ", ";
        })
        -> for_each(|(k, v)| println!("({:?}, {:?})", k, v));
};

input_send.send(("hello", "oakland")).unwrap();
input_send.send(("hello", "berkeley")).unwrap();
input_send.send(("hello", "san francisco")).unwrap();
flow.run_available();
// ("hello", "oakland, berkeley, san francisco, ")

input_send.send(("hello", "palo alto")).unwrap();
flow.run_available();
// ("hello", "palo alto, ")
```


## `keyed_reduce`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> keyed_reduce(A) ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/keyed_reduce.rs" -->
> 1 input stream of type `(K, V)`, 1 output stream of type `(K, V)`.
The output will have one tuple for each distinct `K`, with an accumulated (reduced) value of
type `V`.

If you need the accumulated value to have a different type, use [`keyed_fold`](#keyed_fold).

> Arguments: one Rust closures. The closure takes two arguments: an `&mut` 'accumulator', and
an element. Accumulator should be updated based on the element.

A special case of `fold`, in the spirit of SQL's GROUP BY and aggregation constructs. The input
is partitioned into groups by the first field, and for each group the values in the second
field are accumulated via the closures in the arguments.

> Note: The closures have access to the [`context` object](surface_flows.md#the-context-object).

`keyed_reduce` can also be provided with one generic lifetime persistence argument, either
`'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
within the same tick. With `'static`, values will be remembered across ticks and will be
aggregated with pairs arriving in later ticks. When not explicitly specified persistence
defaults to `'static`.

`keyed_reduce` can also be provided with two type arguments, the key and value type. This is
required when using `'static` persistence if the compiler cannot infer the types.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter([("toy", 1), ("toy", 2), ("shoe", 11), ("shoe", 35), ("haberdashery", 7)])
    -> keyed_reduce(|old: &mut u32, val: u32| *old += val)
    -> for_each(|(k, v)| println!("Total for group {} is {}", k, v));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```

Example using `'tick` persistence:
```rust
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
let mut flow = hydroflow::hydroflow_syntax! {
    source_stream(input_recv)
        -> keyed_reduce::<'tick, &str>(|old: &mut _, val| *old = std::cmp::max(*old, val))
        -> for_each(|(k, v)| println!("({:?}, {:?})", k, v));
};

input_send.send(("hello", "oakland")).unwrap();
input_send.send(("hello", "berkeley")).unwrap();
input_send.send(("hello", "san francisco")).unwrap();
flow.run_available();
// ("hello", "oakland, berkeley, san francisco, ")

input_send.send(("hello", "palo alto")).unwrap();
flow.run_available();
// ("hello", "palo alto, ")
```


## `lattice_batch`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> lattice_batch(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/lattice_batch.rs" -->
> 1 input stream, 1 output stream

> Arguments: The one and only argument is the receive end of a tokio channel that signals when to release the batch downstream.

Given a [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
created in Rust code, `lattice_batch`
is passed the receive end of the channel and when receiving any element
will pass through all received inputs to the output unchanged.

```rust
    let (tx, rx) = hydroflow::util::unbounded_channel::<()>();

    // Will print 0, 1, 2, 3, 4 each on a new line just once.
    let mut df = hydroflow::hydroflow_syntax! {
        repeat_iter(0..5)
            -> map(|x| hydroflow::lattices::Max::new(x))
            -> lattice_batch::<hydroflow::lattices::Max<usize>>(rx)
            -> for_each(|x| { println!("{x:?}"); });
    };

    tx.send(()).unwrap();

    df.run_available();
```


## `lattice_join`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 2">exactly 2</span> | `-> [<input_port>]lattice_join() ->` | <span title="exactly 1">exactly 1</span> |Streaming |

> Input port names: `0` (streaming), `1` (streaming)  

<!-- GENERATED "hydroflow_lang/src/graph/ops/lattice_join.rs" -->
> 2 input streams of type <(K, V1)> and <(K, V2)>, 1 output stream of type <(K, (V1, V2))>

Performs a group-by with lattice-merge aggregate function on LHS and RHS inputs and then forms the
equijoin of the tuples in the input streams by their first (key) attribute. Note that the result nests the 2nd input field (values) into a tuple in the 2nd output field.

You must specify the LHS and RHS lattice types, they cannot be inferred.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print `(key, (2, 1))`
my_join = lattice_join::<hydroflow::lattices::Max<usize>, hydroflow::lattices::Max<usize>>();
source_iter(vec![("key", hydroflow::lattices::Max::new(0)), ("key", hydroflow::lattices::Max::new(2))]) -> [0]my_join;
source_iter(vec![("key", hydroflow::lattices::Max::new(1))]) -> [1]my_join;
my_join -> for_each(|(k, (v1, v2))| println!("({}, ({:?}, {:?}))", k, v1, v2));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```

`lattice_join` can also be provided with one or two generic lifetime persistence arguments, either
`'tick` or `'static`, to specify how join data persists. With `'tick`, pairs will only be
joined with corresponding pairs within the same tick. With `'static`, pairs will be remembered
across ticks and will be joined with pairs arriving in later ticks. When not explicitly
specified persistence defaults to `static.

When two persistence arguments are supplied the first maps to port `0` and the second maps to
port `1`.
When a single persistence argument is supplied, it is applied to both input ports.
When no persistence arguments are applied it defaults to `'static` for both.

The syntax is as follows:
```hydroflow,ignore
lattice_join::<MaxRepr<usize>, MaxRepr<usize>>(); // Or
lattice_join::<'static, MaxRepr<usize>, MaxRepr<usize>>();

lattice_join::<'tick, MaxRepr<usize>, MaxRepr<usize>>();

lattice_join::<'static, 'tick, MaxRepr<usize>, MaxRepr<usize>>();

lattice_join::<'tick, 'static, MaxRepr<usize>, MaxRepr<usize>>();
// etc.
```

### Examples

```rust
use hydroflow::lattices::Max;

let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(usize, Max<usize>)>();
let (out_tx, mut out_rx) = hydroflow::util::unbounded_channel::<(usize, (Max<usize>, Max<usize>))>();

let mut df = hydroflow::hydroflow_syntax! {
    my_join = lattice_join::<'tick, Max<usize>, Max<usize>>();
    source_iter([(7, Max::new(2)), (7, Max::new(1))]) -> [0]my_join;
    source_stream(input_recv) -> [1]my_join;
    my_join -> for_each(|v| out_tx.send(v).unwrap());
};
input_send.send((7, Max::new(5))).unwrap();
df.run_tick();
let out: Vec<_> = hydroflow::util::collect_ready(&mut out_rx);
assert_eq!(out, vec![(7, (Max::new(2), Max::new(5)))]);
```


## `lattice_merge`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> lattice_merge() ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/lattice_merge.rs" -->
> 1 input stream, 1 output stream

> Generic parameters: A [`LatticeRepr`](https://hydro-project.github.io/hydroflow/doc/hydroflow/lang/lattice/trait.LatticeRepr.html)
type.

A specialized operator for merging lattices together into a accumulated value. Like [`fold()`](#fold)
but specialized for lattice types. `lattice_merge::<MyLattice>()` is equivalent to `fold(Default::default, hydroflow::lattices::Merge::merge_owned)`.

`lattice_merge` can also be provided with one generic lifetime persistence argument, either
`'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
within the same tick. With `'static`, values will be remembered across ticks and will be
aggregated with pairs arriving in later ticks. When not explicitly specified persistence
defaults to `'static`.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter([1,2,3,4,5])
    -> map(hydroflow::lattices::Max::new)
    -> lattice_merge::<'static, hydroflow::lattices::Max<usize>>()
    -> for_each(|x: hydroflow::lattices::Max<usize>| println!("Least upper bound: {}", x.0));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `map`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> map(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/map.rs" -->
> 1 input stream, 1 output stream

> Arguments: A Rust closure
For each item passed in, apply the closure to generate an item to emit.

If you do not want to modify the item stream and instead only want to view
each item use the [`inspect`](#inspect) operator instead.

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter(vec!["hello", "world"]) -> map(|x| x.to_uppercase())
    -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `merge`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="at least 0">at least 2</span> | `-> [<input_port>]merge() ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/merge.rs" -->
> *n* input streams of the same type, 1 output stream of the same type

Merges an arbitrary number of input streams into a single stream. Each input sequence is a subsequence of the output, but no guarantee is given on how the inputs are interleaved.

Since `merge` has multiple input streams, it needs to be assigned to
a variable to reference its multiple input ports across statements.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_iter(vec!["hello", "world"]) -> my_merge;
source_iter(vec!["stay", "gold"]) -> my_merge;
source_iter(vec!["don\'t", "give", "up"]) -> my_merge;
my_merge = merge() -> map(|x| x.to_uppercase())
    -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `next_stratum`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> next_stratum() ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/next_stratum.rs" -->
Delays all elements which pass through to the next stratum (in the same
tick).


## `next_tick`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> next_tick() ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/next_tick.rs" -->
Delays all elements which pass through to the next tick. In short,
execution of a hydroflow graph runs as a sequence of distinct "ticks".
Non-monotonic operators compute their output in terms of each tick so
execution doesn't have to block, and it is up to the user to coordinate
data between tick executions to achieve the desired result.

An tick may be divided into multiple _strata_, see the [`next_stratum()`](#next_stratum)
operator.

In the example below `next_tick()` is used alongside `difference()` to
ignore any items in the current tick that already appeared in the previous
tick.
```rust
// Outputs 1 2 3 4 5 6 (on separate lines).
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<usize>();
let mut flow = hydroflow::hydroflow_syntax! {
    inp = source_stream(input_recv) -> tee();
    diff = difference() -> for_each(|x| println!("{}", x));
    inp -> [pos]diff;
    inp -> next_tick() -> [neg]diff;
};

for x in [1, 2, 3, 4] {
    input_send.send(x).unwrap();
}
flow.run_tick();

for x in [3, 4, 5, 6] {
    input_send.send(x).unwrap();
}
flow.run_tick();
```


## `null`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="at least 0 and at most 1">at least 0 and at most 1</span> | `null()` | <span title="at least 0 and at most 1">at least 0 and at most 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/null.rs" -->
> unbounded number of input streams of any type, unbounded number of output streams of any type.

As a source, generates nothing. As a sink, absorbs anything with no effect.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print `1, 2, 3, 4, 5, 6, a, b, c` across 9 lines
null() -> for_each(|_: ()| panic!());
source_iter([1,2,3]) -> map(|i| println!("{}", i)) -> null();
null_src = null();
null_sink = null();
null_src[0] -> for_each(|_: ()| panic!());
// note: use `for_each()` (or `inspect()`) instead of this:
source_iter([4,5,6]) -> map(|i| println!("{}", i)) -> [0]null_sink;
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `persist`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> persist() ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/persist.rs" -->
Stores each item as it passes through, and replays all item every tick.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// Normally `source_iter(...)` only emits once, but with `persist()` will replay the `"hello"`
// on every tick. This is equivalent to `repeat_iter(["hello"])`.
source_iter(["hello"])
    -> persist()
    -> for_each(|item| println!("{}: {}", context.current_tick(), item));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```

`persist()` can be used to introduce statefulness into stateless pipelines. For example this
join only stores data for single `'tick`. The `persist()` operator introduces statefulness
across ticks. This can be useful for optimization transformations within the hydroflow
compiler.
```rust
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
let mut flow = hydroflow::hydroflow_syntax! {
    repeat_iter([("hello", "world")]) -> [0]my_join;
    source_stream(input_recv) -> persist() -> [1]my_join;
    my_join = join::<'tick>() -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
};
input_send.send(("hello", "oakland")).unwrap();
flow.run_tick();
input_send.send(("hello", "san francisco")).unwrap();
flow.run_tick();
// (hello, (world, oakland))
// (hello, (world, oakland))
// (hello, (world, san francisco))
```


## `reduce`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> reduce(A) ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/reduce.rs" -->
> 1 input stream, 1 output stream

> Arguments: a closure which itself takes two arguments:
an ‘accumulator’, and an element. The closure returns the value that the accumulator should have for the next iteration.

Akin to Rust's built-in reduce operator. Folds every element into an accumulator by applying a closure,
returning the final result.

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print 120 (i.e., 1*2*3*4*5)
source_iter([1,2,3,4,5])
    -> reduce(|mut accum, elem| {
        accum *= elem;
        accum
    })
    -> for_each(|e| println!("{}", e));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `repeat_fn`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `repeat_fn(A, B) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/repeat_fn.rs" -->
> 0 input streams, 1 output stream

> Arguments: A batch size per tick, and a zero argument closure to produce each item in the stream.
Similar to `repeat_iter`, but generates the items by calling the supplied closure instead of cloning them from an input iter

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
    repeat_fn(10, || 7) -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `repeat_iter`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `repeat_iter(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/repeat_iter.rs" -->
> 0 input streams, 1 output stream

> Arguments: An iterable Rust object.
Similar to `source_iter`, but delivers all elements from the iterable object
on every tick (where `source_iter` only delivers on the first tick).


```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
    repeat_iter(vec!["Hello", "World"])
        -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `repeat_iter_external`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `repeat_iter_external(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/repeat_iter_external.rs" -->
Same as [`repeat_iter`](#repeat_iter) but treats the iter as external, meaning this operator
will trigger the start of new ticks in order to repeat, causing spinning-like behavior.


## `sort`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> sort() ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/sort.rs" -->
Takes a stream as input and produces a sorted version of the stream as output.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print 1, 2, 3 (in order)
source_iter(vec![2, 3, 1])
    -> sort()
    -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```

`sort` can also be provided with one generic lifetime persistence argument, either
`'tick` or `'static`, to specify how data persists. The default is `'tick`. With `'tick` only
the values will only be collected within a single tick will be sorted and emitted. With
`'static`, values will be remembered across ticks and will be repeatedly emitted each tick (in
order).

```rust
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<usize>();
let mut flow = hydroflow::hydroflow_syntax! {
    source_stream(input_recv)
        -> sort::<'static>()
        -> for_each(|n| println!("{}", n));
};

input_send.send(6).unwrap();
input_send.send(3).unwrap();
input_send.send(4).unwrap();
flow.run_available();
// 3, 4, 6

input_send.send(1).unwrap();
input_send.send(7).unwrap();
flow.run_available();
// 1, 3, 4, 6, 7
```


## `sort_by`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> sort_by(A) ->` | <span title="exactly 1">exactly 1</span> |Blocking |


<!-- GENERATED "hydroflow_lang/src/graph/ops/sort_by.rs" -->
Takes a stream as input and produces a version of the stream as output
sorted according to the key extracted by the closure.

> Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print (1, 'z'), (2, 'y'), (3, 'x') (in order)
source_iter(vec![(2, 'y'), (3, 'x'), (1, 'z')])
    -> sort_by(|(k, _v)| k)
    -> for_each(|x| println!("{:?}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `source_file`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `source_file(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/source_file.rs" -->
> 0 input streams, 1 output stream

> Arguments: An [`AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)`<`[`Path`](https://doc.rust-lang.org/nightly/std/path/struct.Path.html)`>`
for a file to read as json.

Reads the referenced file one line at a time. The line will NOT include the line ending.

Will panic if the file could not be read, or if the file contains bytes that are not valid UTF-8.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_file("Cargo.toml") -> for_each(|line| println!("{}", line));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `source_interval`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `source_interval(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/source_interval.rs" -->
> 0 input streams, 1 output stream

> Arguments: A [`Duration`](https://doc.rust-lang.org/stable/std/time/struct.Duration.html) for this interval.

Emits [Tokio time `Instant`s](https://docs.rs/tokio/1/tokio/time/struct.Instant.html) on a
repeated interval. The first tick completes imediately. Missed ticks will be scheduled as soon
as possible, and the `Instant` will be the missed time, not the late time.

Note that this requires the hydroflow instance be run within a [Tokio `Runtime`](https://docs.rs/tokio/1/tokio/runtime/struct.Runtime.html).
The easiest way to do this is with a [`#[tokio::main]`](https://docs.rs/tokio/1/tokio/attr.main.html)
annotation on `async fn main() { ... }`.

```rust
use std::time::Duration;

use hydroflow::hydroflow_syntax;

#[tokio::main]
async fn main() {
    let mut hf = hydroflow_syntax! {
        source_interval(Duration::from_secs(1))
            -> for_each(|time| println!("This runs every second: {:?}", time));
    };

    // Will print 4 times (fencepost counting).
    tokio::time::timeout(Duration::from_secs_f32(3.5), hf.run_async())
        .await
        .expect_err("Expected time out");

    // Example output:
    // This runs every second: Instant { t: 27471.704813s }
    // This runs every second: Instant { t: 27472.704813s }
    // This runs every second: Instant { t: 27473.704813s }
    // This runs every second: Instant { t: 27474.704813s }
}
```


## `source_iter`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `source_iter(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/source_iter.rs" -->
> 0 input streams, 1 output stream

> Arguments: An iterable Rust object.
Takes the iterable object and delivers its elements downstream
one by one.

Note that all elements are emitted during the first tick.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
    source_iter(vec!["Hello", "World"])
        -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `source_json`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `source_json(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/source_json.rs" -->
> 0 input streams, 1 output stream

> Arguments: An [`AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)`<`[`Path`](https://doc.rust-lang.org/nightly/std/path/struct.Path.html)`>`
for a file to read as json. This will emit the parsed value one time.

`source_json` may take one generic type argument, the type of the value to be parsed, which must implement [`Deserialize`](https://docs.rs/serde/latest/serde/de/trait.Deserialize.html).

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
source_json("example.json") -> for_each(|json: hydroflow::serde_json::Value| println!("{:#?}", json));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `source_stdin`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `source_stdin() ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/source_stdin.rs" -->
> 0 input streams, 1 output stream

> Arguments: port number

`source_stdin` receives a Stream of lines from stdin
and emits each of the elements it receives downstream.

```rust
let mut flow = hydroflow::hydroflow_syntax! {
    source_stdin() -> map(|x| x.unwrap().to_uppercase())
        -> for_each(|x| println!("{}", x));
};
flow.run_async();
```


## `source_stream`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `source_stream(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/source_stream.rs" -->
> 0 input streams, 1 output stream

> Arguments: The receive end of a tokio channel

Given a [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
created in Rust code, `source_stream`
is passed the receive endpoint of the channel and emits each of the
elements it receives downstream.

```rust
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<&str>();
let mut flow = hydroflow::hydroflow_syntax! {
    source_stream(input_recv) -> map(|x| x.to_uppercase())
        -> for_each(|x| println!("{}", x));
};
input_send.send("Hello").unwrap();
input_send.send("World").unwrap();
flow.run_available();
```


## `source_stream_serde`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 0">exactly 0</span> | `source_stream_serde(A) ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/source_stream_serde.rs" -->
> 0 input streams, 1 output stream

> Arguments: [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)

Given a [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
of (serialized payload, addr) pairs, deserializes the payload and emits each of the
elements it receives downstream.

```rust
async fn serde_in() {
    let addr = hydroflow::util::ipv4_resolve("localhost:9000".into()).unwrap();
    let (outbound, inbound, _) = hydroflow::util::bind_udp_bytes(addr).await;
    let mut flow = hydroflow::hydroflow_syntax! {
        source_stream_serde(inbound) -> map(Result::unwrap) -> map(|(x, a): (String, std::net::SocketAddr)| x.to_uppercase())
            -> for_each(|x| println!("{}", x));
    };
    flow.run_available();
}
```


## `tee`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> tee()[<output_port>] ->` | <span title="at least 0">at least 2</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/tee.rs" -->
> 1 input stream, *n* output streams

Takes the input stream and delivers a copy of each item to each output.
> Note: Downstream operators may need explicit type annotations.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
my_tee = source_iter(vec!["Hello", "World"]) -> tee();
my_tee -> map(|x: &str| x.to_uppercase())
    -> for_each(|x| println!("{}", x));
my_tee -> map(|x: &str| x.to_lowercase())
    -> for_each(|x| println!("{}", x));
my_tee -> for_each(|x: &str| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


## `unique`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> unique() ->` | <span title="exactly 1">exactly 1</span> |Streaming |


<!-- GENERATED "hydroflow_lang/src/graph/ops/unique.rs" -->
Takes one stream as input and filters out any duplicate occurrences. The output
contains all unique values from the input.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
// should print 1, 2, 3 (in any order)
source_iter(vec![1, 1, 2, 3, 2, 1, 3])
    -> unique()
    -> for_each(|x| println!("{}", x));
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```

`unique` can also be provided with one generic lifetime persistence argument, either
`'tick` or `'static`, to specify how data persists. The default is `'static`.
With `'tick`, uniqueness is only considered within the current tick, so across multiple ticks
duplicate values may be emitted.
With `'static`, values will be remembered across ticks and no duplicates will ever be emitted.

```rust
let (input_send, input_recv) = hydroflow::util::unbounded_channel::<usize>();
let mut flow = hydroflow::hydroflow_syntax! {
    source_stream(input_recv)
        -> unique::<'tick>()
        -> for_each(|n| println!("{}", n));
};

input_send.send(3).unwrap();
input_send.send(3).unwrap();
input_send.send(4).unwrap();
input_send.send(3).unwrap();
flow.run_available();
// 3, 4

input_send.send(3).unwrap();
input_send.send(5).unwrap();
flow.run_available();
// 3, 5
// Note: 3 is emitted again.
```


## `unzip`
| Inputs | Syntax | Outputs | Flow |
| ------ | ------ | ------- | ---- |
| <span title="exactly 1">exactly 1</span> | `-> unzip()[<output_port>] ->` | <span title="exactly 2">exactly 2</span> |Streaming |

> Output port names: `0`, `1`  

<!-- GENERATED "hydroflow_lang/src/graph/ops/unzip.rs" -->
> 1 input stream of pair tuples `(A, B)`, 2 output streams

Takes the input stream of pairs and unzips each one, delivers each item to
its corresponding side.

```rust
# #[allow(unused_imports)] use hydroflow::{var_args, var_expr};
# #[allow(unused_imports)] use hydroflow::pusherator::Pusherator;
# let __rt = hydroflow::tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
# __rt.block_on(async {
# let mut __hf = hydroflow::hydroflow_syntax! {
my_unzip = source_iter(vec![("Hello", "Foo"), ("World", "Bar")]) -> unzip();
my_unzip[0] -> for_each(|x| println!("0: {}", x)); // Hello World
my_unzip[1] -> for_each(|x| println!("1: {}", x)); // Foo Bar
# };
# for _ in 0..100 {
#     hydroflow::tokio::task::yield_now().await;
#     if !__hf.run_tick() {
#         // No work done.
#         break;
#     }
# }
# })
```


