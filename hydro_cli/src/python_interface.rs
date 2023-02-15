use std::cell::Cell;
use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

#[pyclass(name = "PyHost")]
pub struct PyHost {
    underlying: Arc<dyn crate::core::Host>,
}

#[pymethods]
impl PyHost {
    fn provision<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            underlying.provision().await;
            Ok(Python::with_gil(|py| py.None()))
        })
        .unwrap()
    }
}

#[pyfunction]
fn create_LocalhostHost() -> PyHost {
    PyHost {
        underlying: Arc::new(crate::core::LocalhostHost {}),
    }
}

// #[pyclass(name = "PyLocalhost")]
// pub struct PyHydroflowCrate {
//     underlying: Cell<crate::core::HydroflowCrate>
// }

// #[pymethods]
// impl PyHydroflowCrate {
//     #[new]
//     fn __new__(src: String, on: ) -> Self {
//         PyLocalhost {
//             underlying: Cell::new(crate::core::LocalhostHost {}),
//         }
//     }
// }

#[pymodule]
pub fn hydro_cli_rust(_py: Python<'_>, foo_module: &PyModule) -> PyResult<()> {
    foo_module.add_function(wrap_pyfunction!(create_LocalhostHost, foo_module)?)?;
    Ok(())
}
