//! Build script to generate operator book docs.

use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use hydroflow_lang::graph::ops::OPERATORS;

fn main() -> Result<(), Box<dyn Error>> {
    let mut book_output_path = PathBuf::new();
    book_output_path.push(std::env::var("CARGO_MANIFEST_DIR")?);
    book_output_path.push("../book/surface_ops.gen.md");

    let mut write = BufWriter::new(File::create(book_output_path)?);
    for op in OPERATORS {
        writeln!(write, "- {}", op.name)?;
    }

    Ok(())
}
