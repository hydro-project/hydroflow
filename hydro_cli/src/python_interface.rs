use std::sync::Arc;

use async_channel::Receiver;
use pyo3::prelude::*;
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
            underlying.write().await.deploy().await;
            Ok(Python::with_gil(|py| py.None()))
        })
        .unwrap()
    }
}

#[pyclass(subclass)]
pub struct PyHost {
    underlying: Arc<RwLock<dyn crate::core::Host>>,
}

#[pymethods]
impl PyHost {
    fn provision<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            underlying.write().await.provision().await;
            Ok(Python::with_gil(|py| py.None()))
        })
        .unwrap()
    }
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
                    .add_host(crate::core::LocalhostHost {}),
            },
        )
    }
}

#[pyclass(subclass)]
pub struct PyService {
    underlying: Arc<RwLock<dyn crate::core::Service>>,
}

#[pymethods]
impl PyService {
    fn deploy<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            underlying.write().await.deploy().await;
            Ok(Python::with_gil(|py| py.None()))
        })
        .unwrap()
    }
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
    ) -> (Self, PyService) {
        (
            PyHydroflowCrate {},
            PyService {
                underlying: deployment.underlying.blocking_write().add_service(
                    crate::core::HydroflowCrate {
                        src: src.into(),
                        on: on.underlying.clone(),
                        example,
                        outgoing_ports: Default::default(),
                        incoming_ports: Default::default(),
                        launched_binary: None,
                    },
                ),
            },
        )
    }

    fn stdout(self_: PyRef<'_, Self>) -> PyResult<PyReceiver> {
        let underlying = &self_.as_ref().underlying;
        let mut underlying_mut = underlying.blocking_write();
        let hydro = underlying_mut
            .as_any_mut()
            .downcast_mut::<crate::core::HydroflowCrate>()
            .unwrap();
        Ok(PyReceiver {
            receiver: Arc::new(hydro.stdout()),
        })
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

    from_hydro
        .outgoing_ports
        .insert(from_port.clone(), (Arc::downgrade(to), to_port.clone()));

    to_hydro
        .incoming_ports
        .insert(to_port, (Arc::downgrade(from), from_port));
}

#[pymodule]
pub fn hydro_cli_rust(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyDeployment>()?;
    module.add_class::<PyHost>()?;
    module.add_class::<PyLocalhostHost>()?;

    module.add_class::<PyHydroflowCrate>()?;

    module.add_function(wrap_pyfunction!(create_connection, module)?)?;
    Ok(())
}
