# An Example With Streaming Input

In this example we'll introduce the concept of handoffs and external inputs.

```rust
use hydroflow::builder::prelude::*;

pub fn main() {
    let mut builder = HydroflowBuilder::new();

    // code will go here

    let mut hydroflow = builder.build();
    hydroflow.run_available();
}
```

We'll start out with the above boilerplate. To add a new external input
channel, we can call the [`builder.add_channel_input()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/struct.HydroflowBuilder.html#method.add_channel_input)
method. The method requires several type parameters as well as a human-friendly
name for debugging, and returns a tuple of an input for feeding in items and a
output for receiving items into the subgraph.

```rust,ignore
let (input_example, example_recv) =
    builder.add_channel_input::<_, Option<usize>, VecHandoff<usize>>(
        "My example input"
    );
```

The Rust `::<_, Option<usize>, VecHandoff<usize>>` syntax is affectionately
called the "turbofish" and is how type parameters (generic arguments) are
supplied to generic types and functions. In this case the first type argument
is the label type, and by supplying an underscore `_` we leave it up to the
compiler to infer the type. The second and third type parameters are more
important.

The second type parameter, `Option<usize>`, specifies what we will be sending
into the input channel. Even though the input will be for `usize`s, we specify
`Option`s instead. This extra layer of abstraction allows us to pass in both
individual elements (via [`Option`](https://doc.rust-lang.org/stable/std/option/enum.Option.html)
or [`Single`](https://hydro-project.github.io/hydroflow/doc/hydroflow/lang/collections/struct.Single.html))
or full iterators via [`Iter`](https://hydro-project.github.io/hydroflow/doc/hydroflow/lang/collections/struct.Iter.html).

This syntax is a little verbose and clunky, and is subject to change in the
future.

Finally, the third type parameter is the _handoff_ type. A handoff is a buffer
which stores elements as they wait to be processed. Handoffs are mainly used as
buffers between subgraphs, but in this case it is used to store the external
inputs until the subgraph is ready to process them.
Currently [`VecHandoff<T>`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/handoff/struct.VecHandoff.html)
is the main and only handoff type, but in the future we may implement more
specialized handoffs.

The returned `example_recv` value can be chained on the build a Hydroflow
subgraph just like before. Here is the same program as before, but using the
input channel:

```rust
use hydroflow::builder::prelude::*;

pub fn main() {
    let mut builder = HydroflowBuilder::new();

    // Create our channel input.
    let (input_example, example_recv) =
        builder.add_channel_input::<_, Option<usize>, VecHandoff<usize>>(
            "My example input"
        );

    builder.add_subgraph(
        "main",
        example_recv
            .flatten() // Huh?!
            .map(|n| n * n)
            .filter(|&n| n > 10)
            .pull_to_push()
            .flat_map(|n| (n..=n + 1))
            .for_each(|n| println!("Ahoj {}", n)),
    );

    let mut hydroflow = builder.build();

    println!("A");
    input_example.give(Some(0));
    input_example.give(Some(1));
    input_example.give(Some(2));
    input_example.give(Some(3));
    input_example.give(Some(4));
    input_example.give(Some(5));
    input_example.flush();

    hydroflow.run_available();

    println!("B");
    input_example.give(Some(6));
    input_example.give(Some(7));
    input_example.give(Some(8));
    input_example.give(Some(9));
    input_example.flush();

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
At the bottom we can see supplying inputs with `.give(Option)`. And
importantly, make sure to call `.flush()`!

Unlike the previous example, there's an extra `.flatten()` directly after
starting with `example_recv`. This is because the handoff actually returns its
entire buffer as one element (a [`VecDeque<T>`](https://doc.rust-lang.org/std/collections/struct.VecDeque.html))
rather than returning each element individually. This lets us process the data
as a batch if needed, but most of the time we'll just call `.flatten()` to
convert back into individual elements.
