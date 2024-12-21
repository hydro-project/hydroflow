#![allow(clippy::allow_attributes, missing_docs, reason = "// TODO(mingwei)")]

use std::cell::RefCell;
use std::collections::HashMap;

pub use hydroflow_deploy_integration::*;
use serde::de::DeserializeOwned;

use crate::scheduled::graph::Dfir;

#[macro_export]
macro_rules! launch {
    ($f:expr) => {
        async {
            let ports = $crate::util::deploy::init_no_ack_start().await;
            let flow = $f(&ports);

            println!("ack start");

            $crate::util::deploy::launch_flow(flow).await
        }
    };
}

pub use crate::launch;

pub async fn launch_flow(mut flow: Dfir<'_>) {
    let stop = tokio::sync::oneshot::channel();
    tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        if line.starts_with("stop") {
            stop.0.send(()).unwrap();
        } else {
            eprintln!("Unexpected stdin input: {:?}", line);
        }
    });

    let local_set = tokio::task::LocalSet::new();
    let flow = local_set.run_until(flow.run_async());

    tokio::select! {
        _ = stop.1 => {},
        _ = flow => {}
    }
}

/// Contains runtime information passed by Hydro Deploy to a program,
/// describing how to connect to other services and metadata about them.
pub struct DeployPorts<T = Option<()>> {
    ports: RefCell<HashMap<String, ServerOrBound>>,
    pub meta: T,
}

impl<T> DeployPorts<T> {
    pub fn port(&self, name: &str) -> ServerOrBound {
        self.ports
            .try_borrow_mut()
            .unwrap()
            .remove(name)
            .unwrap_or_else(|| panic!("port {} not found", name))
    }
}

pub async fn init_no_ack_start<T: DeserializeOwned + Default>() -> DeployPorts<T> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    let bind_config = serde_json::from_str::<InitConfig>(trimmed).unwrap();

    // config telling other services how to connect to me
    let mut bind_results: HashMap<String, ServerPort> = HashMap::new();
    let mut binds = HashMap::new();
    for (name, config) in bind_config.0 {
        let bound = config.bind().await;
        bind_results.insert(name.clone(), bound.sink_port());
        binds.insert(name.clone(), bound);
    }

    let bind_serialized = serde_json::to_string(&bind_results).unwrap();
    println!("ready: {bind_serialized}");

    let mut start_buf = String::new();
    std::io::stdin().read_line(&mut start_buf).unwrap();
    let connection_defns = if start_buf.starts_with("start: ") {
        serde_json::from_str::<HashMap<String, ServerPort>>(
            start_buf.trim_start_matches("start: ").trim(),
        )
        .unwrap()
    } else {
        panic!("expected start");
    };

    let mut all_connected = HashMap::new();
    for (name, defn) in connection_defns {
        all_connected.insert(name, ServerOrBound::Server((&defn).into()));
    }

    for (name, defn) in binds {
        all_connected.insert(name, ServerOrBound::Bound(defn));
    }

    DeployPorts {
        ports: RefCell::new(all_connected),
        meta: bind_config
            .1
            .map(|b| serde_json::from_str(&b).unwrap())
            .unwrap_or_default(),
    }
}

pub async fn init<T: DeserializeOwned + Default>() -> DeployPorts<T> {
    let ret = init_no_ack_start::<T>().await;

    println!("ack start");

    ret
}
