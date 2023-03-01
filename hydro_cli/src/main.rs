use std::{fmt::Display, path::PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};
use pyo3::{prelude::*, types::PyList};

pub mod core;

#[allow(non_snake_case)]
mod python_interface;
use python_interface::{hydro_cli_rust, AnyhowWrapper};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// deploys
    Deploy {
        #[arg(value_name = "FILE")]
        config: PathBuf,
    },
}

fn async_wrapper_module(py: Python) -> Result<&PyModule, PyErr> {
    PyModule::from_code(
        py,
        include_str!("async_wrapper.py"),
        "wrapper.py",
        "wrapper",
    )
}

#[derive(Debug)]
struct PyErrWithTraceback {
    err_display: String,
    traceback: String,
}

impl std::error::Error for PyErrWithTraceback {}

impl Display for PyErrWithTraceback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.err_display)?;
        write!(f, "{}", self.traceback)
    }
}

fn deploy(config: PathBuf) -> anyhow::Result<()> {
    Python::with_gil(|py| -> anyhow::Result<()> {
        let syspath: &PyList = py
            .import("sys")
            .and_then(|s| s.getattr("path"))
            .and_then(|p| Ok(p.downcast::<PyList>()?))?;

        syspath.insert(0, PathBuf::from(".").canonicalize().unwrap())?;

        let filename = config.canonicalize().unwrap();
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            std::fs::read_to_string(config).unwrap().as_str(),
            filename.to_str().unwrap(),
            "",
        )
        .with_context(|| format!("failed to load deployment script: {}", filename.display()))?
        .getattr("main")
        .context("expected deployment script to define a `main` function, but one was not found")?
        .into();

        let wrapper = async_wrapper_module(py)?;
        match wrapper.call_method1("run", (fun,)) {
            Ok(_) => Ok(()),
            Err(err) => {
                let traceback = err
                    .traceback(py)
                    .context("traceback was expected but none found")
                    .and_then(|tb| Ok(tb.format()?))?
                    .trim()
                    .to_string();

                if err.is_instance_of::<AnyhowWrapper>(py) {
                    let wrapper = err.into_py(py).extract::<AnyhowWrapper>(py)?;
                    let underlying = wrapper.underlying;
                    let mut underlying = underlying.blocking_write();
                    Err(underlying.take().unwrap()).context(format!("RustException\n{}", traceback))
                } else {
                    Err(PyErrWithTraceback {
                        err_display: format!("{}", err),
                        traceback,
                    }
                    .into())
                }
            }
        }
    })?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    pyo3_asyncio::tokio::init(builder);

    pyo3::append_to_inittab!(hydro_cli_rust);

    let args = Cli::parse();
    if let Some(cmd) = args.command {
        match cmd {
            Commands::Deploy { config } => {
                deploy(config)?;
            }
        }
    }

    Ok(())
}
