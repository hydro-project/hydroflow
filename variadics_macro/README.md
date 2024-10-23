## `tuple!` Macro

Create a tuple from a Variadic type known at compile time.

Example usage:
```
use variadics::var_expr;
use variadics_macro::tuple;

let tup = var_expr!(1, 2, 3, "four");
let a = tuple!(tup, 4);
assert_eq!(a, (1, 2, 3, "four"));

let tup = var_expr!(1, 2, var_expr!(3));
let b = tuple!(tup, 3);
assert_eq!(b, (1, 2, (3, ())));
```