use std::path::Path;

fn main() {
    stageleft_tool::gen_macro(Path::new("../hydroflow_plus_kvs"), "hydroflow_plus_kvs");
}
