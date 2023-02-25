use std::sync::Arc;

use async_channel::Receiver;
use pyo3::{exceptions::PyException, prelude::*};
use tokio::sync::RwLock;

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
            let r = underlying.write().await.deploy().await;
            Python::with_gil(|py| {
                r.map_err(|e| PyException::new_err(format!("{}", e)))?;
                Ok(py.None())
            })
        })
        .unwrap()
    }

    fn start<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            underlying.write().await.start().await;
            Python::with_gil(|py| Ok(py.None()))
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
struct PyGCPComputeEngineHost {}

#[pymethods]
impl PyGCPComputeEngineHost {
    #[new]
    fn new(deployment: &PyDeployment, project: String) -> (Self, PyHost) {
        (
            PyGCPComputeEngineHost {},
            PyHost {
                underlying: deployment
                    .underlying
                    .blocking_write()
                    .add_host(|id| crate::core::GCPComputeEngineHost::new(id, project)),
            },
        )
    }
}

#[pyclass(subclass)]
pub struct PyService {
    underlying: Arc<RwLock<dyn crate::core::Service>>,
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
struct PyHydroflowCrate {}

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
        (
            PyHydroflowCrate {},
            PyService {
                underlying: deployment.underlying.blocking_write().add_service(
                    crate::core::HydroflowCrate::new(
                        src.into(),
                        on.underlying.clone(),
                        example,
                        features,
                    ),
                ),
            },
        )
    }

    fn stdout<'p>(self_: PyRef<'p, Self>, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = &self_.as_ref().underlying;
        let underlying = underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut underlying_mut = underlying.write().await;
            let hydro = underlying_mut
                .as_any_mut()
                .downcast_mut::<crate::core::HydroflowCrate>()
                .unwrap();
            Ok(PyReceiver {
                receiver: Arc::new(hydro.stdout().await),
            })
        })
        .unwrap()
    }

    fn stderr<'p>(self_: PyRef<'p, Self>, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = &self_.as_ref().underlying;
        let underlying = underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut underlying_mut = underlying.write().await;
            let hydro = underlying_mut
                .as_any_mut()
                .downcast_mut::<crate::core::HydroflowCrate>()
                .unwrap();
            Ok(PyReceiver {
                receiver: Arc::new(hydro.stderr().await),
            })
        })
        .unwrap()
    }

    fn exit_code<'p>(self_: PyRef<'p, Self>, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = &self_.as_ref().underlying;
        let underlying = underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut underlying_mut = underlying.write().await;
            let hydro = underlying_mut
                .as_any_mut()
                .downcast_mut::<crate::core::HydroflowCrate>()
                .unwrap();
            Ok(hydro.exit_code().await)
        })
        .unwrap()
    }
}

#[pyfunction]
fn create_connection(from: &PyService, from_port: String, to: &PyService, to_port: String) {
    let from = &from.underlying;
    let to = &to.underlying;

    let mut from_mut = from.blocking_write();
    let mut to_mut = to.blocking_write();
    let from_hydro = from_mut
        .as_any_mut()
        .downcast_mut::<crate::core::HydroflowCrate>()
        .unwrap();
    let to_hydro = to_mut
        .as_any_mut()
        .downcast_mut::<crate::core::HydroflowCrate>()
        .unwrap();

    from_hydro.add_outgoing_port(from_port, to, to_port.clone());
    to_hydro.add_incoming_port(to_port, from_hydro);
}

#[pymodule]
pub fn hydro_cli_rust(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyDeployment>()?;
    module.add_class::<PyHost>()?;
    module.add_class::<PyLocalhostHost>()?;
    module.add_class::<PyGCPComputeEngineHost>()?;

    module.add_class::<PyHydroflowCrate>()?;

    module.add_function(wrap_pyfunction!(create_connection, module)?)?;
    Ok(())
}
