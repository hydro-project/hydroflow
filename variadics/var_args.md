Variadic patterns macro.

Used to [pattern-match](https://doc.rust-lang.org/reference/patterns.html) or "unpack" variadic
tuples. This is used for function arguments, as well as in `match`, `if/while let ...`,
`let ... else`, and `for` expressions.

Although it may somtimes be possible to use `var_expr!` in place of this macro, doing so may
cause confusing errors.

```rust
use variadics::{var_args, var_expr, var_type};

fn my_fn(var_args!(a, b, c): var_type!(usize, &str, bool)) {
    println!("{} {} {}", a, b, c);
}
my_fn(var_expr!(12, "hello", false));
```

```rust
use variadics::{var_args, var_expr};

let val = var_expr!(true, Some("foo"), 2);
if let var_args!(true, Some(item), 0..=3) = val {
    println!("{}", item);
} else {
    unreachable!();
}
```

```rust
# use variadics::{var_args, var_expr};
match var_expr!(true, Some(100), 5) {
    var_args!(false, _, _) => unreachable!(),
    var_args!(true, None, _) => unreachable!(),
    var_args!(true, Some(0..=10), _) => unreachable!(),
    var_args!(true, Some(a), b) => println!("{} {}", a, b),
}
```

The "spread" (or "splat") syntax `...` can be used to unpack the tail of a variadic. Note that
unlike with the other macros, this macro (`var_args!`) only allows the spread syntax on the
final argument.
```rust
# use variadics::{var_args, var_expr};
let var_args!(a, b, ...list_c) = var_expr!("hi", 100, 0.5, false);
assert_eq!(var_expr!(0.5, false), list_c);
```