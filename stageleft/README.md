<h1 class="crate-title">Stageleft</h1>
Stageleft brings the magic of staged programming to Rust, making it easy to write macros with type-safe logic and high-level APIs that can generate efficient code under the hood.

## Example
Stageleft makes it easy to write type-safe code generators. For example, consider a function that raises a number to a power, but the power is known at compile time. Then, we can compile away the power into repeatedly squaring the base. We can implement a staged program for this:

```rust
use stageleft::{q, IntoQuotedOnce, Quoted, RuntimeData};

#[stageleft::entry]
fn raise_to_power(_ctx: &(), value: RuntimeData<i32>, power: u32) -> impl Quoted<i32> {
    if power == 1 {
        q!(value).boxed()
    } else if power % 2 == 0 {
        let half_result = raise_to_power(_ctx, value, power / 2);
        q!({
            let v = half_result;
            v * v
        })
        .boxed()
    } else {
        let half_result = raise_to_power(_ctx, value, power / 2);
        q!({
            let v = half_result;
            (v * v) * value
        })
        .boxed()
    }
}
```

The `q!(...)` macro **quotes** code, which means that it will be spliced into the final generated code. We can take in the unknown base as a runtime parameter (`RuntimeData<i32>`), but the power is known at compile time so we take it as a `u32`. The `_ctx` parameter is unused in this case, because we are returning any borrowed data (see `stageleft::entry` for more details). The `.boxed()` API allows us to return different pieces of spliced code from the same function, and the `impl Quoted<i32>` return type tells the compiler that the function will return a piece of code that evaluates to an `i32`. We can invoke this staged function just like a regular Rust macro:

```rust
let result = raise_to_power!(2, 5);
assert_eq!(result, 1024);
```

But if we expand the macro, we can see that the code has been optimized (simplified for brevity):

```rust
{
    fn expand_staged(value: i32) -> i32 {
        let v = {
            let v = {
                let v = value;
                v * v  // 2^2
            };
            (v * v) * value // 2^5
        };
        v * v // 2^10
    }
    expand_staged(2)
}
```

## Setup
Stageleft requires a particular workspace setup, as any crate that uses Stageleft must have an supporting macro crate (whose contents will be automatically generated). For a crate named `foo`, you will also need a helper crate `foo_macro`.

The main crate `foo` will need the following `Cargo.toml`:
```toml
[package]
// ...

[dependencies]
stageleft = "0.1.0"
foo_macro = { path = "../foo_macro" }

[build-dependencies]
stageleft_tool = "0.1.0"
```

The helper crate should have the following `Cargo.toml`:
```toml
[package]
name = "foo_macro"
// ...

[lib]
proc-macro = true
path = "src/lib.rs"

[features]
default = ["macro"]
macro = []

[dependencies]
// all dependencies of foo

[build-dependencies]
stageleft_tool = "0.1.0"
```

Next, you will need to set up `build.rs` scripts for both of your crates.

In `foo`:
```rust
fn main() {
    stageleft_tool::gen_final!();
}
```

and in `foo_macro`:
```rust
use std::path::Path;

fn main() {
    stageleft_tool::gen_macro(Path::new("../foo"), "foo");
}
```

Finally, you will need to set up the `lib.rs` in these crates.

In `foo`, simply add `stageleft::stageleft_crate!(foo_macro);` at the top of the file.

In `foo_macro`, your `lib.rs` will only need to contain the following:
```rust
stageleft::stageleft_macro_crate!();
```
