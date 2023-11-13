use variadics::*;

fn main() {
    let var_args!(a, ...b, c) = var_expr!(1, 2.0, "three", false);
}