# An Example With Streaming Input
> In this example we will cover:
> - the input `channel` concept, which streams data in from outside the Hydroflow spec
> - the [`recv_stream`](./surface_ops.gen.md#recv_stream) operator that brings channel input into Hydroflow
> - Rust syntax to programmatically send data to a (local) channel

In our previous examples, data came from within the Hydroflow spec, via Rust iterators and the [`source_iter`](./surface_ops.gen.md#source_iter) operator. In most cases, however, data comes from outside the Hydroflow spec. In this example, we'll see a simple version of this idea, with data being generated on the same machine and sent into the channel programmatically via Rust.

We start with a skeleton much like before:

```rust
use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut hydroflow = hydroflow_syntax! {
        // code will go here
    };

    hydroflow.run_available();
}
```

To add a new external input
channel, we can use the `hydroflow::util::unbounded_channel()` function in Rust before we declare the Hydroflow spec:
```rust, ignore
    let (input_example, example_recv) = hydroflow::util::unbounded_channel::<usize>();
```
Under the covers, this is a [multiple-producer/single-consumer (`mpsc`) channel](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.unbounded_channel.html) provided by Rust's [tokio](https://docs.rs/tokio/latest/tokio) library, which is usually the appropriate choice for an inbound Hydroflow stream.
Think of it as a high-performance "mailbox" that any sender can fill with well-typed data.

The Rust `::<usize>` syntax uses what is affectionately
called the "turbofish", which is how type parameters (generic arguments) are
supplied to generic types and functions. In this case it specifies that this tokio channel
transmits items of type `usize`.
The returned `example_recv` value can be used via a [`recv_stream`](./surface_ops.gen.md#recv_stream)
to build a Hydroflow subgraph just like before. Here is the same program as before, but using the
input channel:
```rust
use hydroflow::hydroflow_syntax;

pub fn main() {
   // Create our channel input
    let (input_example, example_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut flow = hydroflow_syntax! {
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
    input_example.send(1).unwrap();
    input_example.send(0).unwrap();
    input_example.send(2).unwrap();
    input_example.send(3).unwrap();
    input_example.send(4).unwrap();
    input_example.send(5).unwrap();

    flow.run_available();

    println!("B");
    input_example.send(6).unwrap();
    input_example.send(7).unwrap();
    input_example.send(8).unwrap();
    input_example.send(9).unwrap();
    flow.run_available();
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
At the bottom we can see how to programatically supply `usize`-typed inputs with the tokio 
`.send()` method.  We call Rust's `.unwrap()` method to ignore the error messages from 
`.send()` in this simple case.  In later examples we'll see how to 
allow for data coming in over a network.
