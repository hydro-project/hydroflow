# Simple Example

Lets build on the simplest example to explore some of the operators available
in Hydroflow. You may be familiar with operators such as [`.map(...)`](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/surface/trait.BaseSurface.html#method.map),
[`.filter(...)`](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/surface/trait.BaseSurface.html#method.filter),
[`.flatten(...)`](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/surface/trait.BaseSurface.html#method.flatten),
etc. from Rust iterators or from other programming languages, and these are
also available in Hydroflow.

```rust
use hydroflow::builder::prelude::*;

pub fn main() {
    let mut builder = HydroflowBuilder::new();
    builder.add_subgraph(
        "main",
        (0..10)
            .into_hydroflow()

            .map(|n| n * n)
            .filter(|&n| n > 10)

            .pull_to_push()

            .map(|n| (n..=n+1))
            .flatten()

            .for_each(|n| println!("Howdy {}", n)),
    );

    let mut hydroflow = builder.build();
    hydroflow.tick();
}
```
`.map()` transforms each element one-to-one as it flows through the subgraph.
In this case, we square each number. Then `.filter()` only keeps any squared
numbers which are greater than 10.

After the pull to push. `.map()` converts each number into a
[`RangeInclusive`](https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html)
of it and the next number. We then call `.flatten()` to conver the range
iterators into the numbers which they contains.

We can also express the same program with the combination operators
[`.filter_map()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/surface/trait.BaseSurface.html#method.filter_map)
and [`.flat_map()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/surface/trait.BaseSurface.html#method.flat_map).
```rust
use hydroflow::builder::prelude::*;

pub fn main() {
    let mut builder = HydroflowBuilder::new();
    builder.add_subgraph(
        "main",
        (0..10)
            .into_hydroflow()
            .filter_map(|n| {
                let n2 = n * n;
                if n2 > 10 {
                    Some(n2)
                }
                else {
                    None
                }
            })
            .pull_to_push()
            .flat_map(|n| (n..=n+1))
            .for_each(|n| println!("G'day {}", n)),
    );

    let mut hydroflow = builder.build();
    hydroflow.tick();
}
```

Note all of these operators work on both pull and push sides of a subgraph.
[`BaseSurface`](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/surface/trait.BaseSurface.html#)
contains the full list of such operators.
