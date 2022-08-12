use std::path::PathBuf;

fn main() {
    rust_sitter_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
