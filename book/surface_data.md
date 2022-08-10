# Data Sources and Sinks in Rust
Any useful flow requires us to define sources of data, either generated computationally or received from 
and outside environment via I/O.

## `recv_iter`
A flow can receive data from a Rust collection object via the `recv_iter` operator, which takes the 
iterable collection as an argument and passes the items down the flow. 
For example, here we iterate through a vector of `usize` items and push them down the flow:
```rust,ignore
    recv_iter(vec![0, 1]) -> ...
```
The Hello, World example above uses this construct.

## `recv_stream`
More commonly, a flow is handling external data that comes in asynchronously via Rust's _tokio_ library.
For this we need to allocate tokio _channels_ that allow Rust code to send data into the Hydroflow inputs. 
The code below creates a channel for data of (Rust) type `(usize, usize)`:
```rust,ignore
    let (input_send, input_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
```
Now in Rust we can now push data into the channel via _tokio_'s methods. E.g. for testing
we can do it explicitly as follows:
```rust,ignore
    input_send.send((0, 1)).unwrap()
```
And in our Hydroflow syntax we can receive the data from the channel using the `recv_stream` syntax and
pass it along a flow:
```rust,ignore
    recv_stream(input_recv) -> ...
```

To put this together, let's revisit our Hello, World example from above with data sent 
in from outside the flow:
```rust
# use hydroflow::hydroflow_syntax;
let (input_send, input_recv) = tokio::sync::mpsc::unbounded_channel::<&str>();
let mut flow = hydroflow_syntax! {
    recv_stream(input_recv) -> map(|x| x.to_uppercase())
        -> for_each(|x| println!("{}", x));
};
input_send.send("Hello").unwrap();
input_send.send("World").unwrap();
flow.run_available();
```