use std::sync::Arc;

use pyo3::prelude::*;
use tokio::sync::RwLock;

#[pyclass(name = "PyDeployment")]
pub struct PyDeployment {
    underlying: Arc<RwLock<crate::core::Deployment>>,
}

#[pymethods]
impl PyDeployment {
    fn deploy<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            underlying.write().await.deploy().await;
            Ok(Python::with_gil(|py| py.None()))
        })
        .unwrap()
    }
}

#[pyfunction]
fn create_Deployment() -> PyDeployment {
    PyDeployment {
        underlying: Arc::new(RwLock::new(crate::core::Deployment {
            hosts: Vec::new(),
            services: Vec::new(),
        })),
    }
}

#[pyclass(name = "PyHost")]
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

#[pyfunction]
fn create_LocalhostHost(deployment: &PyDeployment) -> PyHost {
    PyHost {
        underlying: deployment
            .underlying
            .blocking_write()
            .add_host(crate::core::LocalhostHost {}),
    }
}

#[pyclass(name = "PyService")]
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

#[pyfunction]
fn create_HydroflowCrate(src: String, on: &PyHost, example: Option<String>, deployment: &PyDeployment) -> PyService {
    PyService {
        underlying: deployment.underlying.blocking_write().add_service(
            crate::core::HydroflowCrate {
                src: src.into(),
                on: on.underlying.clone(),
                example,
                outgoing_ports: Default::default(),
                incoming_ports: Default::default(),
            },
        ),
    }
}

#[pyfunction]
fn create_connection(from: &PyService, from_port: String, to: &PyService, to_port: String) {
    let from = from.underlying.clone();
    let to = to.underlying.clone();

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
        .insert(from_port.clone(), (Arc::downgrade(&to), to_port.clone()));

    to_hydro
        .incoming_ports
        .insert(to_port, (Arc::downgrade(&from), from_port));
}

#[pymodule]
pub fn hydro_cli_rust(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(create_Deployment, module)?)?;
    module.add_function(wrap_pyfunction!(create_LocalhostHost, module)?)?;
    module.add_function(wrap_pyfunction!(create_HydroflowCrate, module)?)?;
    module.add_function(wrap_pyfunction!(create_connection, module)?)?;
    Ok(())
}
