Variadic expressions (values) macro.

Creates a variadic tuple value from a list of expressions.

Create a variadic tuple value:
```rust
use variadics::var_expr;

let list = var_expr!(10, false, "foo");

assert_eq!(list, (10, (false, ("foo", ()))),)
```

Although this can be used as a pattern to unpack tuples, [`var_args!`] should be used instead:
```
# use variadics::*;
// Ok...
let var_expr!(a, b, c) = var_expr!(10, false, "foo");
// Better:
let var_args!(a, b, c) = var_expr!(10, false, "foo");

assert_eq!(a, 10);
assert_eq!(b, false);
assert_eq!(c, "foo");
```

The "spread" (or "splat") syntax `...` can be used to concatenate variadics together:
```rust
# use variadics::var_expr;
let list_a = var_expr!(0.5, "foo");
let list_b = var_expr!(-5, false);
// Spread syntax:
let list_c = var_expr!(...list_a, ...list_b, "bar");
// Equals `var_expr!(0.5, "foo", -5, false, "bar)`.
```