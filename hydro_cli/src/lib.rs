use std::sync::Arc;

use async_channel::Receiver;
use pyo3::exceptions::PyException;
use pyo3::types::PyDict;
use pyo3::{create_exception, prelude::*, wrap_pymodule};
use tokio::sync::RwLock;

mod cli;
pub mod core;

use crate::core::hydroflow_crate::ClientPortConfig;

create_exception!(hydro_cli_core, AnyhowError, PyException);

#[pyclass]
#[derive(Clone)]
pub struct AnyhowWrapper {
    pub underlying: Arc<RwLock<Option<anyhow::Error>>>,
}

#[pyclass(name = "Deployment")]
pub struct Deployment {
    underlying: Arc<RwLock<crate::core::Deployment>>,
}

#[pymethods]
impl Deployment {
    #[new]
    fn new() -> Self {
        Deployment {
            underlying: Arc::new(RwLock::new(crate::core::Deployment {
                hosts: Vec::new(),
                services: Vec::new(),
            })),
        }
    }

    #[allow(non_snake_case)]
    fn Localhost(&self, py: Python<'_>) -> PyResult<Py<pyo3::PyAny>> {
        Ok(Py::new(
            py,
            PyClassInitializer::from(Host {
                underlying: self
                    .underlying
                    .blocking_write()
                    .add_host(|id| crate::core::LocalhostHost { id }),
            })
            .add_subclass(LocalhostHost {}),
        )?
        .into_py(py))
    }

    #[allow(non_snake_case)]
    fn GCPComputeEngineHost(
        &self,
        py: Python<'_>,
        project: String,
        machine_type: String,
        image: String,
        region: String,
    ) -> PyResult<Py<pyo3::PyAny>> {
        Ok(Py::new(
            py,
            PyClassInitializer::from(Host {
                underlying: self.underlying.blocking_write().add_host(|id| {
                    crate::core::GCPComputeEngineHost::new(id, project, machine_type, image, region)
                }),
            })
            .add_subclass(LocalhostHost {}),
        )?
        .into_py(py))
    }

    #[allow(non_snake_case)]
    fn CustomService(
        &self,
        py: Python<'_>,
        on: &Host,
        external_ports: Vec<u16>,
    ) -> PyResult<Py<pyo3::PyAny>> {
        let service =
            self.underlying
                .blocking_write()
                .add_service(crate::core::CustomService::new(
                    on.underlying.clone(),
                    external_ports,
                ));

        Ok(Py::new(
            py,
            PyClassInitializer::from(Service {
                _underlying: service.clone(),
            })
            .add_subclass(CustomService {
                _underlying: service,
            }),
        )?
        .into_py(py))
    }

    #[allow(non_snake_case)]
    fn HydroflowCrate(
        &self,
        py: Python<'_>,
        src: String,
        on: &Host,
        example: Option<String>,
        features: Option<Vec<String>>,
        args: Option<Vec<String>>,
    ) -> PyResult<Py<pyo3::PyAny>> {
        let service =
            self.underlying
                .blocking_write()
                .add_service(crate::core::HydroflowCrate::new(
                    src.into(),
                    on.underlying.clone(),
                    example,
                    features,
                    args,
                ));

        Ok(Py::new(
            py,
            PyClassInitializer::from(Service {
                _underlying: service.clone(),
            })
            .add_subclass(HydroflowCrate {
                underlying: service,
            }),
        )?
        .into_py(py))
    }

    fn deploy<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            underlying.write().await.deploy().await.map_err(|e| {
                Python::with_gil(|py| {
                    AnyhowError::new_err(
                        Py::new(
                            py,
                            AnyhowWrapper {
                                underlying: Arc::new(RwLock::new(Some(e))),
                            },
                        )
                        .unwrap(),
                    )
                })
            })?;
            Python::with_gil(|py| Ok(py.None()))
        })
        .unwrap()
    }

    fn start<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            underlying.write().await.start().await;
            Ok(Python::with_gil(|py| py.None()))
        })
        .unwrap()
    }
}

#[pyclass(subclass)]
pub struct Host {
    underlying: Arc<RwLock<dyn crate::core::Host>>,
}

#[pyclass(extends=Host, subclass)]
struct LocalhostHost {}

#[pyclass(extends=Host, subclass)]
struct GCPComputeEngineHost {
    underlying: Arc<RwLock<crate::core::GCPComputeEngineHost>>,
}

#[pymethods]
impl GCPComputeEngineHost {
    #[getter]
    fn internal_ip(&self) -> String {
        self.underlying
            .blocking_read()
            .launched
            .as_ref()
            .unwrap()
            .internal_ip
            .clone()
    }

    #[getter]
    fn external_ip(&self) -> Option<String> {
        self.underlying
            .blocking_read()
            .launched
            .as_ref()
            .unwrap()
            .external_ip
            .clone()
    }

    #[getter]
    fn ssh_key_path(&self) -> String {
        self.underlying
            .blocking_read()
            .launched
            .as_ref()
            .unwrap()
            .ssh_key_path()
            .to_str()
            .unwrap()
            .to_string()
    }
}

#[pyclass(subclass)]
pub struct Service {
    _underlying: Arc<RwLock<dyn crate::core::Service>>,
}

#[pyclass]
struct PyReceiver {
    receiver: Arc<Receiver<String>>,
}

#[pymethods]
impl PyReceiver {
    fn next<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let my_receiver = self.receiver.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let underlying = my_receiver.recv();
            Ok(underlying.await.map(Some).unwrap_or(None))
        })
        .unwrap()
    }
}

#[pyclass(extends=Service, subclass)]
struct CustomService {
    _underlying: Arc<RwLock<crate::core::CustomService>>,
}

#[pyclass(extends=Service, subclass)]
struct HydroflowCrate {
    underlying: Arc<RwLock<crate::core::HydroflowCrate>>,
}

fn convert_py_receiver(receiver: PyReceiver) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        // load helper python function as a string
        let module = PyModule::from_code(
            py,
            r#"
async def pyreceiver_to_async_generator(pyreceiver):
    while True:
        res = await pyreceiver.next()
        if res is None:
            break
        else:
            yield res
    "#,
            "converter",
            "converter",
        )?;

        Ok(module
            .call_method1("pyreceiver_to_async_generator", (receiver.into_py(py),))
            .unwrap()
            .into())
    })
}

#[pymethods]
impl HydroflowCrate {
    fn stdout<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let underlying = underlying.read().await;
            convert_py_receiver(PyReceiver {
                receiver: Arc::new(underlying.stdout().await),
            })
        })
        .unwrap()
    }

    fn stderr<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let underlying = underlying.read().await;
            convert_py_receiver(PyReceiver {
                receiver: Arc::new(underlying.stderr().await),
            })
        })
        .unwrap()
    }

    fn exit_code<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let underlying = underlying.read().await;
            Ok(underlying.exit_code().await)
        })
        .unwrap()
    }

    #[getter]
    fn ports(&self) -> HydroflowCratePorts {
        HydroflowCratePorts {
            underlying: self.underlying.clone(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
struct HydroflowCratePorts {
    underlying: Arc<RwLock<crate::core::HydroflowCrate>>,
}

#[pymethods]
impl HydroflowCratePorts {
    fn __getattribute__(&self, name: String) -> PyResult<HydroflowCratePort> {
        Ok(HydroflowCratePort {
            underlying: crate::core::hydroflow_crate::ClientPortConfig::Direct(
                Arc::downgrade(&self.underlying),
                name,
            ),
        })
    }
}

#[pyclass]
#[derive(Clone)]
struct HydroflowCratePort {
    pub(crate) underlying: ClientPortConfig,
}

#[pymethods]
impl HydroflowCratePort {
    fn send_to(&self, to: &HydroflowCratePort) -> PyResult<()> {
        if let ClientPortConfig::Direct(from, from_port) = &self.underlying {
            let from = from.upgrade().unwrap();
            let mut from_write = from.try_write().unwrap();
            from_write
                .add_client_port(&from, from_port.clone(), to.underlying.clone())
                .unwrap();
            Ok(())
        } else {
            panic!("Can only send from a direct port");
        }
    }
}

#[pyfunction]
fn demux(mapping: &PyDict) -> HydroflowCratePort {
    HydroflowCratePort {
        underlying: ClientPortConfig::Demux(
            mapping
                .into_iter()
                .map(|(k, v)| {
                    let k = k.extract::<u32>().unwrap();
                    let v = v.extract::<HydroflowCratePort>().unwrap();
                    (k, v.underlying)
                })
                .collect(),
        ),
    }
}

#[pymodule]
pub fn _core(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add("AnyhowError", py.get_type::<AnyhowError>())?;
    module.add_class::<AnyhowWrapper>()?;

    module.add_class::<Deployment>()?;

    module.add_class::<Host>()?;
    module.add_class::<LocalhostHost>()?;
    module.add_class::<GCPComputeEngineHost>()?;

    module.add_class::<Service>()?;
    module.add_class::<CustomService>()?;
    module.add_class::<HydroflowCrate>()?;
    module.add_class::<HydroflowCratePort>()?;

    module.add_function(wrap_pyfunction!(demux, module)?)?;

    module.add_wrapped(wrap_pymodule!(cli::cli))?;

    Ok(())
}
