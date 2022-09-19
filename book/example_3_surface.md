# An Example With Streaming Input

In this example we'll introduce the concept of handoffs and external inputs.

```rust
use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut hydroflow = hydroflow_syntax! {
        // code will go here
    };

    hydroflow.run_available();
}
```

We'll start out with the above boilerplate. To add a new external input
channel, we can use the `tokio` library:
```rust, ignore
    let (input_example, example_recv) = tokio::sync::mpsc::unbounded_channel::<usize>();
```
This is a [multiple-producer/single-consumer (`mpsc`) channel](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.unbounded_channel.html), which is usually the appropriate choice for an inbound Hydroflow stream:
think of it as a high-performance "mailbox" that any sender can fill with well-typed data.

The Rust `::<usize>` syntax is affectionately
called the "turbofish" and is how type parameters (generic arguments) are
supplied to generic types and functions. In this case it specifies that this tokio channel
transmits `usize` items.
The returned `example_recv` value can be chained via a [`recv_stream`](./surface_ops#example_recv) 
to build a Hydroflow subgraph just like before. Here is the same program as before, but using the
input channel:
```rust
use hydroflow::hydroflow_syntax;

pub fn main() {
// Create our channel input
let (input_example, example_recv) = tokio::sync::mpsc::unbounded_channel::<usize>();

let mut hydroflow = hydroflow_syntax! {
    recv_stream(example_recv)
    -> filter_map(|n: usize| {
        let n2 = n * n;
        if n2 > 10 {
            Some(n2)
        }
        else {
            None
        }
    })
    -> flat_map(|n| (n..=n+1))
    -> for_each(|n| println!("Ahoj {}", n))
};

println!("A");
input_example.send(0).unwrap();
input_example.send(1).unwrap();
input_example.send(2).unwrap();
input_example.send(3).unwrap();
input_example.send(4).unwrap();
input_example.send(5).unwrap();

hydroflow.run_available();

println!("B");
input_example.send(6).unwrap();
input_example.send(7).unwrap();
input_example.send(8).unwrap();
input_example.send(9).unwrap();

hydroflow.run_available();
}
```
```txt
A
Ahoj 16
Ahoj 17
Ahoj 25
Ahoj 26
B
Ahoj 36
Ahoj 37
Ahoj 49
Ahoj 50
Ahoj 64
Ahoj 65
Ahoj 81
Ahoj 82
```
At the bottom we can see supplying inputs with the tokio `.send()` method; we call Rust's `.unwrap()` 
method to ignore the error messages from `.send()` in this simple case. 
