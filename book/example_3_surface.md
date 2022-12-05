# An Example With Streaming Input

In this example we'll introduce the concept of handoffs and external inputs.

```rust
use hydroflow::hydroflow_syntax;
use serde::{Deserialize, Serialize};
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct UsizeMessage{
    payload: usize
};

pub fn main() {
    let mut hydroflow = hydroflow_syntax! {
        // code will go here
    };

    hydroflow.run_available();
}
```

We'll start out with the above boilerplate. Note that we've added a few things:
- the statement beginning `pub struct` defines a very simple struct, `UsizeMessage`,
that we will use as the type for our external inputs. 
- the preceding line, `use serde::{Deserialize, Serialize};`, allows us to serialize/deserialize (serde) data.
We will not use this in our example, but it's extremely common in distributed Hydroflow programs.
- the `#derive[...]` line before that derives certain basic traits
we will need for any struct to work in Hydroflow.

We could have skipped all three of these lines and simply used `usize` in place of `UsizeMessage` throughout this example. 
We use a custom struct here because it's typical: distributed programs typically deal with 
network inputs that have more complex structure and require 
serde support.



To add a new external input
channel, we can use the `hydroflow::util::unbounded_channel()` function:
```rust, ignore
    let (input_example, example_recv) = hydroflow::util::unbounded_channel::<UsizeMessage>();
```
Under the covers, this is a [multiple-producer/single-consumer (`mpsc`) channel](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.unbounded_channel.html), which is usually the appropriate choice for an inbound Hydroflow stream:
think of it as a high-performance "mailbox" that any sender can fill with well-typed data.

The Rust `::<UsizeMessage>` syntax uses what is affectionately
called the "turbofish", which is how type parameters (generic arguments) are
supplied to generic types and functions. In this case it specifies that this tokio channel
transmits items of type `UsizeMessage`.
The returned `example_recv` value can be used via a [`recv_stream`](./surface_ops.gen.md#recv_stream)
to build a Hydroflow subgraph just like before. Here is the same program as before, but using the
input channel:
```rust
use hydroflow::hydroflow_syntax;
use serde::{Deserialize, Serialize};
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct UsizeMessage {
    payload: usize,
}

pub fn main() {
   // Create our channel input
    let (input_example, example_recv) = hydroflow::util::unbounded_channel::<UsizeMessage>();

    let mut flow = hydroflow_syntax! {
         recv_stream(example_recv)
        -> filter_map(|n: UsizeMessage| {
            let n2 = n.payload * n.payload;
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
    input_example.send(UsizeMessage { payload: 1 }).unwrap();
    input_example.send(UsizeMessage { payload: 0 }).unwrap();
    input_example.send(UsizeMessage { payload: 2 }).unwrap();
    input_example.send(UsizeMessage { payload: 3 }).unwrap();
    input_example.send(UsizeMessage { payload: 4 }).unwrap();
    input_example.send(UsizeMessage { payload: 5 }).unwrap();

    flow.run_available();

    println!("B");
    input_example.send(UsizeMessage { payload: 6 }).unwrap();
    input_example.send(UsizeMessage { payload: 7 }).unwrap();
    input_example.send(UsizeMessage { payload: 8 }).unwrap();
    input_example.send(UsizeMessage { payload: 9 }).unwrap();
    flow.run_available();
};
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
At the bottom we can see how to programatically supply `UsizeMessage`-typed inputs with the tokio 
`.send()` method; we call Rust's `.unwrap()` method to ignore the error messages from 
`.send()` in this simple case. 
