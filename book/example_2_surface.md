# Simple Example
> In this example we will cover some additional standard Hydroflow operators:
> - [`map`](./surface_ops.gen.md#map)
> - [`filter`](./surface_ops.gen.md#filter)
> - [`flatten`](./surface_ops.gen.md#flatten)
> - [`filter_map`](./surface_ops.gen.md#filter_map)
> - [`flat_map`](./surface_ops.gen.md#flat_map)

Lets build on the simplest example to explore some of the operators available
in Hydroflow. You may be familiar with operators such as [`map(...)`](./surface_ops.gen.md#map),
[`filter(...)`](./surface_ops.gen.md#filter), [`flatten`(...)](./surface_ops.gen.md#flatten),
etc. from Rust iterators or from other programming languages, and these are
also available in Hydroflow.

```rust
use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut flow = hydroflow_syntax! {
        recv_iter(0..10)
            -> map(|n| n * n)
            -> filter(|&n| n > 10)
            -> map(|n| (n..=n+1))
            -> flatten()
            -> for_each(|n| println!("Howdy {}", n));
    };

    flow.run_available();
}
```
Let's take this one operator at a time, starting after the `recv_iter` operator we saw in the previous example.

- `-> map(|n| n * n)` transforms each element one-to-one as it flows through the subgraph.
In this case, we square each number. 
- Next, `-> filter(|&n| n > 10)` only keeps any squared numbers that are greater than 10.

- The subsequent `-> map(|n| (n..=n+1))` uses standard Rust syntax to convert each number `n` into a
[`RangeInclusive`](https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html)
\[`n`, `n+1`\]. 

- We then call `-> flatten()` to convert the ranges back
into a stream of the individual numbers which they contain.

- Finally we use the now-familiar `for_each` operator to print each number.

We can also express the same program with more aggressive use of combination operators like
[`filter_map()`](./surface_ops.gen.md#filtermap) and [`flat_map()`](./surface_ops.gen.md#flat_map). Hydroflow will compile these down to the same
machine code:
```rust
# use hydroflow::hydroflow_syntax;
 pub fn main() {
    let mut flow = hydroflow_syntax! {
        recv_iter(0..10)
        -> filter_map(|n| {
            let n2 = n * n;
            if n2 > 10 {
                Some(n2)
            }
            else {
                None
            }
        })
        -> flat_map(|n| (n..=n+1))
        -> for_each(|n| println!("G'day {}", n))
    };

    flow.run_available();
}
```

Results:
```txt
G'day 16
G'day 17
G'day 25
G'day 26
G'day 36
G'day 37
G'day 49
G'day 50
G'day 64
G'day 65
G'day 81
G'day 82
```
