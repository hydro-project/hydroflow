use std::path::PathBuf;
use std::sync::Arc;

use perf_options::PerfOptions;

use super::Host;
use crate::ServiceBuilder;

pub(crate) mod build;
pub mod ports;

pub mod service;
pub use service::*;

pub mod perf_options;

#[derive(PartialEq)]
pub enum CrateTarget {
    Default,
    Bin(String),
    Example(String),
}

/// Specifies a crate that uses `hydroflow_cli_integration` to be
/// deployed as a service.
pub struct HydroflowCrate {
    src: PathBuf,
    target: CrateTarget,
    on: Arc<dyn Host>,
    profile: Option<String>,
    perf: Option<PerfOptions>,
    args: Vec<String>,
    display_name: Option<String>,
}

impl HydroflowCrate {
    /// Creates a new `HydroflowCrate` that will be deployed on the given host.
    /// The `src` argument is the path to the crate's directory, and the `on`
    /// argument is the host that the crate will be deployed on.
    pub fn new(src: impl Into<PathBuf>, on: Arc<dyn Host>) -> Self {
        Self {
            src: src.into(),
            target: CrateTarget::Default,
            on,
            profile: None,
            perf: None,
            args: vec![],
            display_name: None,
        }
    }

    /// Sets the target to be a binary with the given name,
    /// equivalent to `cargo run --bin <name>`.
    pub fn bin(mut self, bin: impl Into<String>) -> Self {
        if self.target != CrateTarget::Default {
            panic!("target already set");
        }

        self.target = CrateTarget::Bin(bin.into());
        self
    }

    /// Sets the target to be an example with the given name,
    /// equivalent to `cargo run --example <name>`.
    pub fn example(mut self, example: impl Into<String>) -> Self {
        if self.target != CrateTarget::Default {
            panic!("target already set");
        }

        self.target = CrateTarget::Example(example.into());
        self
    }

    /// Sets the profile to be used when building the crate.
    /// Equivalent to `cargo run --profile <profile>`.
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        if self.profile.is_some() {
            panic!("profile already set");
        }

        self.profile = Some(profile.into());
        self
    }

    pub fn perf(mut self, perf: impl Into<PerfOptions>) -> Self {
        if self.perf.is_some() {
            panic!("perf path already set");
        }

        self.perf = Some(perf.into());
        self
    }

    /// Sets the arguments to be passed to the binary when it is launched.
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(|s| s.into()));
        self
    }

    /// Sets the display name for this service, which will be used in logging.
    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        if self.display_name.is_some() {
            panic!("display_name already set");
        }

        self.display_name = Some(display_name.into());
        self
    }
}

impl ServiceBuilder for HydroflowCrate {
    type Service = HydroflowCrateService;
    fn build(self, id: usize) -> Self::Service {
        let (bin, example) = match self.target {
            CrateTarget::Default => (None, None),
            CrateTarget::Bin(bin) => (Some(bin), None),
            CrateTarget::Example(example) => (None, Some(example)),
        };

        HydroflowCrateService::new(
            id,
            self.src,
            self.on,
            bin,
            example,
            self.profile,
            self.perf,
            None,
            Some(self.args),
            self.display_name,
            vec![],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployment;

    #[tokio::test]
    async fn test_crate_panic() {
        let mut deployment = deployment::Deployment::new();

        let localhost = deployment.Localhost();

        let service = deployment.add_service(
            HydroflowCrate::new("../hydro_cli_examples", localhost.clone())
                .example("panic_program")
                .profile("dev"),
        );

        deployment.deploy().await.unwrap();

        let mut stdout = service.try_read().unwrap().stdout();

        deployment.start().await.unwrap();

        assert_eq!(stdout.recv().await.unwrap(), "hello!");

        assert!(stdout.recv().await.is_none());
    }
}
