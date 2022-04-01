# Simplest Example

Lets start out with the simplest possible Hydroflow program, which prints out
the numbers in `0..10`.

```rust
use hydroflow::builder::prelude::*;

pub fn main() {
    let mut builder = HydroflowBuilder::new();
    builder.add_subgraph(
        "main",
        (0..10)
            .into_hydroflow()
            .pull_to_push()
            .for_each(|n| println!("Hello {}", n)),
    );

    let mut hydroflow = builder.build();
    hydroflow.tick();
}
```

And the output:
```txt
Hello 0
Hello 1
Hello 2
Hello 3
Hello 4
Hello 5
Hello 6
Hello 7
Hello 8
Hello 9
```

Although this is a trivial program, there's quite a bit going on that might
phase you. Let's go line by line.

```rust,ignore
use hydroflow::builder::prelude::*;
```
This is a prelude import: a convention where all the important dependencies are
re-exported in a single `prelude` module for convenience. In this case it's
everything you need for using Hydroflow via the [Surface API](./architecture.md#apis).
You can check [Hydroflow's rustdocs](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/prelude/index.html)
to see what exactly is imported.

Next, inside the main method we create a new `HydroflowBuilder`:
```rust,ignore
pub main() {
    let mut builder = HydroflowBuilder::new();
    builder.add_subgraph(
        "main",
        // <code for creating the subgraph>
    );
    // ...
}
```
And add a subgraph. We are required to give the subgraph a name, in this case
`"main"`. The "code for creating the subgraph" is:
```rust,ignore
        (0..10)
```
This creates a Rust [`Range` iterator](https://doc.rust-lang.org/std/ops/struct.Range.html).


```rust,ignore
            .into_hydroflow()
```
Converts a Rust [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
for use in Hydroflow via the [`IntoHydroflow`](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/trait.IntoHydroflow.html)
trait.


```rust,ignore
            .pull_to_push()
```
This line is specific to Hydroflow and a bit unusual. It converts the chain
from _pull_ into _push_ which allows us to use push-based methods:


```rust,ignore
            .for_each(|n| println!("Hello {}", n)),
```
Which consumes each element, printing it out as they arrive.

For now, all we need to know is that every Hydroflow subgraph starts as _pull_,
then becomes _push_ after we call `.pull_to_push()`. Some operators can only be
used in the pull side, some can only be used in the push side, and some can be
used in either. The reason behind having separate pull and push-based operators
is explained in the [Architecture](./architecture.html#compiled-layer) section.

Note that these chained method operators do not run any immediate
computations. Instead they provide a blueprint of what the Hydroflow graph
should look like.

```rust,ignore
    let mut hydroflow = builder.build();
    hydroflow.tick();
```
Finally we build the `Hydroflow` instance and run it via the [`tick()` method](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.tick).
Note that `tick()` runs the Hydroflow graph until no more work is immediately
available. In this case running the graph drains the iterator completely, so no
more work will ever be available. But once we add in external inputs such as
network ingress then more work might appear later. The [`tick_stratum()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.tick_stratum),
[`run()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run)
and [`run_async()`] https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_async
methods provide other ways to execute the graph.
