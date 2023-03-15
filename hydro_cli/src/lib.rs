use std::sync::Arc;

use async_channel::Receiver;
use bytes::Bytes;
use futures::SinkExt;
use hydroflow_cli_integration::{Connected, ConnectedBidi, DynSink};
use pyo3::exceptions::PyException;
use pyo3::types::{PyBytes, PyDict};
use pyo3::{create_exception, prelude::*, wrap_pymodule};
use tokio::sync::RwLock;

mod cli;
pub mod core;

use crate::core::hydroflow_crate::ports::HydroflowSource;

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
            underlying: Arc::new(RwLock::new(crate::core::Deployment::default())),
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
        network: GCPNetwork,
    ) -> PyResult<Py<pyo3::PyAny>> {
        Ok(Py::new(
            py,
            PyClassInitializer::from(Host {
                underlying: self.underlying.blocking_write().add_host(|id| {
                    crate::core::GCPComputeEngineHost::new(
                        id,
                        project,
                        machine_type,
                        image,
                        region,
                        network.underlying,
                    )
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
                underlying: service,
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

#[pyclass]
#[derive(Clone)]
struct GCPNetwork {
    underlying: Arc<RwLock<crate::core::gcp::GCPNetwork>>,
}

#[pymethods]
impl GCPNetwork {
    #[new]
    fn new(project: String) -> Self {
        GCPNetwork {
            underlying: Arc::new(RwLock::new(crate::core::gcp::GCPNetwork {
                project,
                vpc_id: None,
            })),
        }
    }
}

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
    underlying: Arc<RwLock<crate::core::CustomService>>,
}

#[pymethods]
impl CustomService {
    fn client_port(&self) -> CustomClientPort {
        CustomClientPort {
            underlying: Arc::new(RwLock::new(
                crate::core::custom_service::CustomClientPort::new(Arc::downgrade(
                    &self.underlying,
                )),
            )),
        }
    }
}

#[pyclass]
#[derive(Clone)]
struct CustomClientPort {
    pub(crate) underlying: Arc<RwLock<crate::core::custom_service::CustomClientPort>>,
}

#[pymethods]
impl CustomClientPort {
    fn send_to(&mut self, to: &HydroflowCratePort) {
        self.underlying
            .try_write()
            .unwrap()
            .send_to(to.underlying.clone());
    }

    fn server_port<'p>(&self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let underlying = underlying.read().await;
            Ok(ServerPort {
                underlying: Some(underlying.server_port().await),
            })
        })
        .unwrap()
    }
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
            underlying: crate::core::hydroflow_crate::ports::HydroflowPortConfig::Direct(
                Arc::downgrade(&self.underlying),
                name,
            ),
        })
    }
}

#[pyclass]
#[derive(Clone)]
struct HydroflowCratePort {
    pub(crate) underlying: crate::core::hydroflow_crate::ports::HydroflowPortConfig,
}

#[pymethods]
impl HydroflowCratePort {
    fn send_to(&mut self, to: &HydroflowCratePort) {
        self.underlying.send_to(to.underlying.clone());
    }
}

#[pyfunction]
fn demux(mapping: &PyDict) -> HydroflowCratePort {
    HydroflowCratePort {
        underlying: crate::core::hydroflow_crate::ports::HydroflowPortConfig::Demux(
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

#[pyclass]
struct ServerPort {
    underlying: Option<hydroflow_cli_integration::ServerOrBound>,
}

#[pymethods]
impl ServerPort {
    fn sink<'p>(&mut self, py: Python<'p>) -> &'p pyo3::PyAny {
        let underlying = self.underlying.take().unwrap();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            Ok(ConnectedSink {
                underlying: Arc::new(RwLock::new(
                    underlying.connect::<ConnectedBidi>().await.take_sink(),
                )),
            })
        })
        .unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
struct ConnectedSink {
    underlying: Arc<RwLock<DynSink<Bytes>>>,
}

#[pymethods]
impl ConnectedSink {
    fn send<'p>(&mut self, data: Py<PyBytes>, py: Python<'p>) -> &'p PyAny {
        let underlying = self.underlying.clone();
        let bytes = Bytes::from(data.as_bytes(py).to_vec());
        pyo3_asyncio::tokio::future_into_py(py, async move {
            underlying.write().await.send(bytes).await?;
            Ok(())
        })
        .unwrap()
    }
}

#[pymodule]
pub fn _core(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    ctrlc::set_handler(move || {}).unwrap();

    module.add("AnyhowError", py.get_type::<AnyhowError>())?;
    module.add_class::<AnyhowWrapper>()?;

    module.add_class::<Deployment>()?;

    module.add_class::<Host>()?;
    module.add_class::<LocalhostHost>()?;

    module.add_class::<GCPNetwork>()?;
    module.add_class::<GCPComputeEngineHost>()?;

    module.add_class::<Service>()?;
    module.add_class::<CustomService>()?;
    module.add_class::<HydroflowCrate>()?;
    module.add_class::<HydroflowCratePort>()?;

    module.add_class::<ServerPort>()?;
    module.add_class::<ConnectedSink>()?;

    module.add_function(wrap_pyfunction!(demux, module)?)?;

    module.add_wrapped(wrap_pymodule!(cli::cli))?;

    Ok(())
}
