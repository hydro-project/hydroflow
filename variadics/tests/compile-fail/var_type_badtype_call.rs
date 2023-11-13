use variadics::*;

fn main() {
    type _List = var_type!(String, u32, MyIdent(), bool);
}
