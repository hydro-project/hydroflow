# Simplest Example

Lets start out with the simplest possible Hydroflow program, which prints out
the numbers in `0..10`.

```rust
use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut flow = hydroflow_syntax! {
        recv_iter(0..10) -> for_each(|n| println!("Hello {}", n));
    };

    flow.run_available();
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

Although this is a trivial program, it's useful to go through it line by line.
```rust,ignore
use hydroflow::hydroflow_syntax;
```
This import gives you everything you need from hydroflow to write code with the 
[_surface syntax_](./surface_syntax.md), which is the most common way to interact
with Hydroflow.

Next, inside the main method we specify a flow by calling the 
`hydroflow_syntax!` macro. We assign the resulting `Hydroflow` instance to
a mutable variable---we will be changing its status when we run it.
```rust,ignore
# use hydroflow::hydroflow_syntax;
pub fn main() {
    let mut flow = hydroflow_syntax! {
        recv_iter(0..10) -> for_each(|n| println!("Hello {}", n));
    };
```
The flow starts with a [`recv_iter`](./surface_ops.md#recv_iter) operator that emits the 
numbers 1 through 10, and passes them along the arrow `->` operator downstream to a 
[`for_each`](./surface_ops.md#for_each) operator that invokes its closure argument to print each
item passed in.


Finally we run this flow via the [`run_available()` method](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_available).
```rust,ignore
    flow.run_available();
}
```
Note that `run_available()` runs the Hydroflow graph until no more work is immediately
available. In this case running the graph drains the iterator completely, so no
more work will ever be available. But once we add in external inputs such as
network ingress then more work might appear later. The [`run_epoch()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_epoch),
[`run_stratum()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_stratum),
[`run()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run),
and [`run_async()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_async)
methods provide other ways to execute the graph.