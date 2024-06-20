use std::path::Path;

fn main() {
    stageleft_tool::gen_macro(Path::new("../flow"), "flow");
}
