use std::path::PathBuf;

use clap::{Parser, Subcommand};
use pyo3::{
    prelude::*,
    types::{PyList, PyTuple},
};

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

fn deploy(config: PathBuf) -> Result<(), ()> {
    Python::with_gil(|py| {
        let syspath: &PyList = py.import("sys")?.getattr("path")?.downcast::<PyList>()?;
        syspath.insert(0, PathBuf::from(".").canonicalize().unwrap())?;

        let filename = config.canonicalize().unwrap();
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            std::fs::read_to_string(config).unwrap().as_str(),
            filename.to_str().unwrap(),
            "",
        )?
        .getattr("main")?
        .into();

        let wrapper = PyModule::from_code(
            py,
            r#"
import asyncio

async def wrap(inner):
    try:
        return (await inner(), None)
    except asyncio.CancelledError as e:
        import traceback
        traceback.print_exc()
        return (None, e)
    except BaseException as e:
        import traceback
        traceback.print_exc()
        return (None, e)
    except:
        import traceback
        traceback.print_exc()

        return (None, Exception("Unknown error"))

def run(inner):
    event_loop = asyncio.get_event_loop()
    task = event_loop.create_task(wrap(inner))
    should_cancel = False
    try:
        res = event_loop.run_until_complete(task)
    except:
        should_cancel = True

    if should_cancel:
        task.cancel()
        res = event_loop.run_until_complete(task)

    pending = asyncio.all_tasks(loop=event_loop)
    group = asyncio.gather(*pending)
    event_loop.run_until_complete(group)

    event_loop.close()
    return res
"#,
            "wrapper.py",
            "wrapper",
        )?;

        wrapper.call_method1("run", (fun,)).and_then(|v| {
            let v = v.downcast::<PyTuple>()?;
            let _result = v.get_item(0)?;
            let error = v.get_item(1)?;
            if error.is_none() {
                Ok(())
            } else {
                Err(PyErr::from_value(error))
            }
        })
    })
    .map_err(|e: PyErr| {
        Python::with_gil(|py| {
            e.print_and_set_sys_last_vars(py);
        });
    })?;

    Ok(())
}

use python_interface::hydro_cli_rust;

fn main() -> Result<(), ()> {
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
