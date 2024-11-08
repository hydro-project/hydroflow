//! Build script to generate operator book docs.

use std::env::VarError;
use std::fmt::Write as _FmtWrite;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Result, Write};
use std::path::{Path, PathBuf};

use hydroflow_lang::graph::ops::{PortListSpec, OPERATORS};
use hydroflow_lang::graph::PortIndexValue;
use itertools::Itertools;
use quote::ToTokens;

const FILENAME: &str = "surface_ops_gen.md";

fn book_file(filename: impl AsRef<Path>) -> PathBuf {
    let mut pathbuf = PathBuf::new();
    pathbuf.push(std::env!("CARGO_MANIFEST_DIR"));
    pathbuf.push("../docs/docs/hydroflow/syntax/");
    pathbuf.push(filename);
    pathbuf
}

fn book_file_writer(filename: impl AsRef<Path>) -> Result<BufWriter<File>> {
    let pathbuf = book_file(filename);
    Ok(BufWriter::new(File::create(pathbuf)?))
}

fn write_operator_docgen(op_name: &str, write: &mut impl Write) -> Result<()> {
    let doctest_path = PathBuf::from_iter([
        std::env!("CARGO_MANIFEST_DIR"),
        "../docs/docgen",
        &*format!("{}.md", op_name),
    ]);
    let mut read_string = String::new();
    File::open(doctest_path)?.read_to_string(&mut read_string)?;
    write!(write, "{}", read_string.split("<!--").map(|t| t.replace("<", "\\<")).join("<!--"))?;
    Ok(())
}

fn update_book() -> Result<()> {
    let mut ops: Vec<_> = OPERATORS.iter().collect();
    // operators that have their name start with "_" are internal compiler operators, we should sort those after all the user-facing ops.
    // but underscore by default sorts before all [A-z]
    ops.sort_by(|a, b| {
        a.name
            .starts_with('_')
            .cmp(&b.name.starts_with('_'))
            .then_with(|| a.name.cmp(b.name))
    });

    let mut write = book_file_writer(FILENAME)?;
    writeln!(write, "{}", PREFIX)?;
    writeln!(
        write,
        "<!-- GENERATED {:?} -->",
        file!().replace(std::path::MAIN_SEPARATOR, "/")
    )?;

    // Table of contents.
    {
        let category_operators = OPERATORS
            .iter()
            .flat_map(|op| {
                op.categories
                    .iter()
                    .copied()
                    .map(|category| (category, op.name))
            })
            .into_group_map();
        let categories = category_operators
            .keys()
            .copied()
            .sorted()
            .collect::<Vec<_>>();

        for category in categories {
            writeln!(write, "**{}:**", category.name())?;
            writeln!(
                write,
                "{}  ",
                category_operators
                    .get(&category)
                    .unwrap()
                    .iter()
                    .map(|&op_name| format!("[`{0}`](#{0})", op_name))
                    .join(", ")
            )?;
            writeln!(write, "{}", category.description())?;
            writeln!(write)?;
        }
    }
    writeln!(write, "---")?;
    writeln!(write)?;

    // Individual operator doc entries
    for &op in ops.iter() {
        writeln!(write, "## `{}`", op.name)?;

        writeln!(write, "| Inputs | Syntax | Outputs | Flow |")?;
        writeln!(write, "| ------ | ------ | ------- | ---- |")?;
        let metadata_str = format!(
            "| <span title={:?}>{}</span> | `{}{}({}){}` | <span title={:?}>{}</span> |",
            op.hard_range_inn.human_string(),
            op.soft_range_inn.human_string(),
            if op.soft_range_inn.contains(&0) {
                ""
            } else if op.soft_range_inn.contains(&1) {
                "-> "
            } else {
                "-> [<input_port>]"
            },
            op.name,
            ('A'..)
                .take(op.num_args)
                .fold(String::new(), |mut s, c| {
                    write!(&mut s, "{}, ", c).unwrap();
                    s
                })
                .strip_suffix(", ")
                .unwrap_or(""),
            if op.soft_range_out.contains(&0) {
                ""
            } else if op.soft_range_out.contains(&1) {
                " ->"
            } else {
                "[<output_port>] ->"
            },
            op.hard_range_out.human_string(),
            op.soft_range_out.human_string(),
        );

        let mut blocking = false;
        let input_port_names = op.ports_inn.map(|ports_inn_fn| (ports_inn_fn)());
        let input_str_maybe = match input_port_names {
            None => {
                blocking = (op.input_delaytype_fn)(&PortIndexValue::Elided(None)).is_some();
                None
            }
            Some(PortListSpec::Fixed(port_names)) => Some(format!(
                "> Input port names: {}  ",
                port_names
                    .into_iter()
                    .fold(String::new(), |mut s, idx| {
                        let port_ix = idx.clone().into();
                        let flow_str = if (op.input_delaytype_fn)(&port_ix).is_some() {
                            blocking = true;
                            "blocking"
                        } else {
                            "streaming"
                        };
                        write!(&mut s, "`{}` ({}), ", idx.into_token_stream(), flow_str).unwrap();
                        s
                    })
                    .strip_suffix(", ")
                    .unwrap_or("&lt;EMPTY&gt;")
            )),
            Some(PortListSpec::Variadic) => {
                Some("> Input port names: Variadic, as specified in arguments.".to_string())
            }
        };
        let mut output_str_maybe = None;
        if let Some(f) = op.ports_out {
            output_str_maybe = Some(match (f)() {
                PortListSpec::Fixed(port_names) => format!(
                    "> Output port names: {}  ",
                    port_names
                        .into_iter()
                        .fold(String::new(), |mut s, idx| {
                            write!(&mut s, "`{}`, ", idx.into_token_stream()).unwrap();
                            s
                        })
                        .strip_suffix(", ")
                        .unwrap_or("&lt;EMPTY&gt;")
                ),
                PortListSpec::Variadic => {
                    "> Output port names: Variadic, as specified in arguments.".to_string()
                }
            })
        }
        let flow_str = if blocking { "Blocking" } else { "Streaming" };
        writeln!(write, "{}{} |", metadata_str, flow_str)?;
        writeln!(write)?;
        if let Some(input_str) = input_str_maybe {
            writeln!(write, "{}", input_str)?;
        }
        if let Some(output_str) = output_str_maybe {
            writeln!(write, "{}", output_str)?;
        }
        writeln!(write)?;

        if let Err(err) = write_operator_docgen(op.name, &mut write) {
            eprintln!("Docgen error: {}", err);
        }
        writeln!(write)?;
        writeln!(write)?;
    }

    Ok(())
}

fn main() {
    if Err(VarError::NotPresent) != std::env::var("CARGO_CFG_HYDROFLOW_GENERATE_DOCS") {
        if let Err(err) = update_book() {
            eprintln!("hydroflow_macro/build.rs error: {:?}", err);
        }
    }
}

const PREFIX: &str = "\
---
sidebar_position: 4
---

# Hydroflow's Operators

In our previous examples we made use of some of Hydroflow's operators.
Here we document each operator in more detail. Most of these operators
are based on the Rust equivalents for iterators; see the [Rust documentation](https://doc.rust-lang.org/std/iter/trait.Iterator.html).";
