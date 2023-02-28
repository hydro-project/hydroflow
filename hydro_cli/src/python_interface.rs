use std::sync::Arc;

use async_channel::Receiver;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use tokio::sync::RwLock;

#[pyclass(extends=PyException)]
#[derive(Clone)]
pub struct AnyhowWrapper {
    pub underlying: Arc<RwLock<Option<anyhow::Error>>>,
}

#[pyclass(name = "PyDeployment")]
pub struct PyDeployment {
    underlying: Arc<RwLock<crate::core::Deployment>>,
}

#[pymethods]
impl PyDeployment {
    #[new]
    fn new() -> Self {
        PyDeployment {
            underlying: Arc::new(RwLock::new(crate::core::Deployment {
                hosts: Vec::new(),
                services: Vec::new(),
            })),
        }
    }

    fn deploy<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            underlying.write().await.deploy().await.map_err(|e| {
                Python::with_gil(|py| {
                    PyErr::from_value(
                        Py::new(
                            py,
                            AnyhowWrapper {
                                underlying: Arc::new(RwLock::new(Some(e))),
                            },
                        )
                        .unwrap()
                        .as_ref(py),
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
pub struct PyHost {
    underlying: Arc<RwLock<dyn crate::core::Host>>,
}

#[pyclass(extends=PyHost, subclass)]
struct PyLocalhostHost {}

#[pymethods]
impl PyLocalhostHost {
    #[new]
    fn new(deployment: &PyDeployment) -> (Self, PyHost) {
        (
            PyLocalhostHost {},
            PyHost {
                underlying: deployment
                    .underlying
                    .blocking_write()
                    .add_host(|id| crate::core::LocalhostHost { id }),
            },
        )
    }
}

#[pyclass(extends=PyHost, subclass)]
struct PyGCPComputeEngineHost {
    underlying: Arc<RwLock<crate::core::GCPComputeEngineHost>>,
}

#[pymethods]
impl PyGCPComputeEngineHost {
    #[new]
    fn new(
        deployment: &PyDeployment,
        project: String,
        machine_type: String,
        image: String,
        region: String,
    ) -> (Self, PyHost) {
        let host = deployment.underlying.blocking_write().add_host(|id| {
            crate::core::GCPComputeEngineHost::new(id, project, machine_type, image, region)
        });

        (
            PyGCPComputeEngineHost {
                underlying: host.clone(),
            },
            PyHost { underlying: host },
        )
    }

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
pub struct PyService {
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

#[pyclass(extends=PyService, subclass)]
struct PyCustomService {
    _underlying: Arc<RwLock<crate::core::CustomService>>,
}

#[pymethods]
impl PyCustomService {
    #[new]
    fn new(deployment: &PyDeployment, on: &PyHost, external_ports: Vec<u16>) -> (Self, PyService) {
        let service =
            deployment
                .underlying
                .blocking_write()
                .add_service(crate::core::CustomService::new(
                    on.underlying.clone(),
                    external_ports,
                ));

        (
            PyCustomService {
                _underlying: service.clone(),
            },
            PyService {
                _underlying: service,
            },
        )
    }
}

#[pyclass(extends=PyService, subclass)]
struct PyHydroflowCrate {
    underlying: Arc<RwLock<crate::core::HydroflowCrate>>,
}

#[pymethods]
impl PyHydroflowCrate {
    #[new]
    fn new(
        deployment: &PyDeployment,
        src: String,
        on: &PyHost,
        example: Option<String>,
        features: Option<Vec<String>>,
    ) -> (Self, PyService) {
        let service =
            deployment
                .underlying
                .blocking_write()
                .add_service(crate::core::HydroflowCrate::new(
                    src.into(),
                    on.underlying.clone(),
                    example,
                    features,
                ));

        (
            PyHydroflowCrate {
                underlying: service.clone(),
            },
            PyService {
                _underlying: service,
            },
        )
    }

    fn stdout<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let underlying = underlying.read().await;
            Ok(PyReceiver {
                receiver: Arc::new(underlying.stdout().await),
            })
        })
        .unwrap()
    }

    fn stderr<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let underlying = underlying.read().await;
            Ok(PyReceiver {
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
}

#[pyfunction]
fn create_connection(
    from: &PyHydroflowCrate,
    from_port: String,
    to: &PyHydroflowCrate,
    to_port: String,
) {
    let from = &from.underlying;
    let to = &to.underlying;

    let mut from_write = from.blocking_write();
    let mut to_write = to.blocking_write();

    from_write.add_outgoing_port(from_port, to, to_port.clone());
    to_write.add_incoming_port(to_port, &from_write);
}

#[pymodule]
pub fn hydro_cli_rust(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<AnyhowWrapper>()?;

    module.add_class::<PyDeployment>()?;

    module.add_class::<PyHost>()?;
    module.add_class::<PyLocalhostHost>()?;
    module.add_class::<PyGCPComputeEngineHost>()?;

    module.add_class::<PyService>()?;
    module.add_class::<PyCustomService>()?;
    module.add_class::<PyHydroflowCrate>()?;

    module.add_function(wrap_pyfunction!(create_connection, module)?)?;
    Ok(())
}
