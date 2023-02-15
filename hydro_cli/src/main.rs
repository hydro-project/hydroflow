use std::path::PathBuf;

use clap::{Parser, Subcommand};
use pyo3::{prelude::*, types::PyList};

pub mod core;

#[allow(non_snake_case)]
mod python_interface;

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

async fn deploy(config: PathBuf) -> Result<(), PyErr> {
    let res: PyResult<_> = Python::with_gil(|py| {
        let syspath: &PyList = py.import("sys")?.getattr("path")?.downcast::<PyList>()?;
        syspath.insert(0, PathBuf::from(".").canonicalize().unwrap())?;

        let filename = config.canonicalize().unwrap();
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            std::fs::read_to_string(config).unwrap().as_str(),
            &filename.to_str().unwrap(),
            "",
        )?
        .getattr("main")?
        .into();

        // call object without any arguments
        Ok(pyo3_asyncio::tokio::into_future(
            &(fun.call0(py)?).as_ref(py),
        )?)
    });

    let fn_future = res.unwrap();
    fn_future.await?;

    Ok(())
}

use python_interface::hydro_cli_rust;

fn main() {
    async fn main() -> PyResult<()> {
        let args = Cli::parse();
        if let Some(cmd) = args.command {
            match cmd {
                Commands::Deploy { config } => {
                    deploy(config).await?;
                }
            }
        }

        Ok(())
    }

    pyo3::append_to_inittab!(hydro_cli_rust);

    pyo3::prepare_freethreaded_python();

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();

    pyo3_asyncio::tokio::init(builder);
    pyo3::Python::with_gil(|py| {
        pyo3_asyncio::tokio::run(py, main()).unwrap_or_else(|e| e.print_and_set_sys_last_vars(py));
    });
}
