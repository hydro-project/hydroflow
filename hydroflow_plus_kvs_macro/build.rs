use std::path::Path;

fn main() {
    stagefright_tool::gen_macro(
        Path::new("../hydroflow_plus_kvs_flow"),
        "hydroflow_plus_kvs",
    );
}
