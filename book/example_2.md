# Simple Example

Lets build on the simplest example to explore some of the operators available
in Hydroflow. You may be familiar with operators such as `.map(...)`,
`.filter(...)`, `.flatten(...)`, etc. from Rust iterators or from other
programming languages, and these are also available in Hydroflow. These
operators are also allowed on both the pull and push sides of subgraphs.

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

            .for_each(|n| println!("Hello {}", n)),
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


