# Hydroflow's Built-in Operators

In our previous examples we made use of some of Hydroflow's built-in operators.
Here we document each operators in more detail. Most of these operators
are based on the Rust equivalents for iterators; see the [Rust documentation](https://doc.rust-lang.org/std/iter/trait.Iterator.html).

> *NOTE* Would be nice to format more like Rust docs, but I couldn't find that source as a template.

## filter

> 1 input stream, 1 output stream

> Arguments: A Rust closure that returns a boolean

Filter outputs a subsequence of the items it receives at its input, according to a
Rust boolean closure passed in as an argument

```rust
recv_iter(vec!["hello", "world"]) -> filter(|x| x == &"hello")
    -> for_each(|x| println!("{}", x));
```

## map

> 1 input stream, 1 output stream

> Arguments: A Rust closure
For each item passed in, apply the closure to generate an item to emit.

```rust
recv_iter(vec!["hello", "world"]) -> map(|x| x.to_uppercase())
    -> for_each(|x| println!("{}", x));
```

## flat_map

> 1 input stream, 1 output stream

> Arguments: A Rust closure that handles an iterator

For each item `i` passed in, treat `i` as an iterator and map the closure to that
iterator to produce items one by one. the type of the input items must be iterable.

```rust
recv_iter(vec!["hello", "world"]) -> flat_map(|x| x.chars())
    -> for_each(|x| println!("{}", x));
```

## filter_map

> 1 input stream, 1 output stream

An operator that both filters and maps. It yields only the items for which the supplied closure returns `Some(value)`.

```rust
recv_iter(vec!["1", "hello", "world", "2"]) -> filter_map(|s| s.parse().ok())
    -> for_each(|x| println!("{}", x));
```

## merge

> *n* input streams of the same type, 1 output stream of the same type

Merges an arbitrary number of input streams into a single stream. Each input sequence is a subsequence of the output, but no guarantee is given on how the inputs are interleaved.

Since `merge` has multiple input streams, it needs to be assigned to
a variable to reference its multiple input ports across statements.

```rust
my_merge = merge();
recv_iter(vec!["hello", "world"]) -> [0]my_merge;
recv_iter(vec!["stay", "gold"]) -> [1]my_merge;
recv_iter(vec!["don\'t", "give", "up"]) -> [2]my_merge;
my_merge -> map(|x| x.to_uppercase()) 
    -> for_each(|x| println!("{}", x));
```

## join

> 2 input streams of type <(K, V1)> and <(K, V2)>, 1 output stream of type <(K, (V1, V2))>

Forms the equijoin of the tuples in the input streams by their first (key) attribute. Note that the result nests the 2nd input field (values) into a tuple in the 2nd output field.

```rust
recv_iter(vec![("hello", "world"), ("stay", "gold")]) -> [0]my_join;
recv_iter(vec![("hello", "cleveland")]) -> [1]my_join;
my_join -> for_each(|(k, (v1, v2))| println!("({}, ({}, {})", k, v1, v2));
```

## tee

> 1 input stream, *n* output streams

Takes the input stream and delivers a copy of each item to each output.
> Note: Downstream operators may need explicit type annotations.

```rust
my_tee = recv_iter(vec!["Hello", "World"]) -> tee();
my_tee[0] -> map(|x: &str| x.to_uppercase())
    -> for_each(|x| println!("{}", x));
my_tee[1] -> map(|x: &str| x.to_lowercase())
    -> for_each(|x| println!("{}", x));
my_tee[2] -> for_each(|x: &str| println!("{}", x));
```

## recv_stream

> 0 input streams, 1 output stream

> Arguments: The receive end of a tokio channel

Given a tokio channel created in Rust code, `recv_stream` 
is passed the receive endpoint of the channel and emits each of the
elements it receives downstream.

```rust
let (input_send, input_recv) = tokio::sync::mpsc::unbounded_channel::<&str>();
let mut flow = hydroflow_syntax! {
    recv_stream(input_recv) -> map(|x| x.to_uppercase()) 
        -> for_each(|x| println!("{}", x));
};
input_send.send("Hello").unwrap();
input_send.send("World").unwrap();
flow.run_available();
```

## recv_iter

> 0 input streams, 1 output stream

> Arguments: An iterable Rust object.
Takes the iterable object and delivers its elements downstream
one by one.

```rust
    recv_iter(vec!["Hello", "World"])
        -> for_each(|x| println!("{}", x));
```

## for_each

> 1 input stream, 0 output streams

> Arguments: a Rust closure

Iterates through a stream passing each element to the closure in the
argument.

```rust
    recv_iter(vec!["Hello", "World"])
        -> for_each(|x| println!("{}", x));
```