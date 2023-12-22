use variadics::*;

fn main() {
    type List = var_type!(String, u32, ...f64, bool);
    fn check(_: List) {}
}
