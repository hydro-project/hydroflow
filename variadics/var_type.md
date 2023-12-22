Variadic types macro.

Creates a variadic tuple type from a list of types.

`var_expr!` can be used to define simple types but will result in confusing errors for more
complex types. Use this macro, `var_type!` instead.

```rust
# use std::collections::HashMap;
use variadics::{var_expr, var_type};

// A simple variadic type. Although `var_expr!` would work in this case, it cannot handle
// more complex types i.e. ones with generics.
let list: var_type!(i32, bool, String) = Default::default();

// A more complex type:
let list: var_type!(
    &'static str,
    HashMap<i32, i32>,
    <std::vec::Vec<bool> as IntoIterator>::Item,
) = var_expr!("foo", HashMap::new(), false);
```

The "spread" (or "splat") syntax `...` can be used to concatenate variadics together:
```rust
# use variadics::var_type;
type ListA = var_type!(f32, &'static str);
type ListB = var_type!(i32, bool);
// Spread syntax:
type ListC = var_type!(...ListA, ...ListB, Option::<()>);
// Equals `var_type!(f32, &'static str, i32, bool, Option::<()>)`.
```

Unfortunately, expressions and types cannot be handled using the same macro due to the
undefeated [bastion of the turbofish](https://github.com/rust-lang/rust/blob/7fd15f09008dd72f40d76a5bebb60e3991095a5f/src/test/ui/parser/bastion-of-the-turbofish.rs).