use std::path::PathBuf;

use async_channel::Sender;
use cargo::{
    core::{compiler::BuildConfig, resolver::CliFeatures, Workspace},
    ops::{CompileFilter, CompileOptions, FilterRule, LibRule},
    util::{command_prelude::CompileMode, interning::InternedString},
    Config,
};
use tokio::task::JoinHandle;

use crate::core::HostTargetType;

type CacheKey = (PathBuf, Option<String>, HostTargetType, Option<Vec<String>>);

static BUILD_CACHE: once_cell::sync::Lazy<
    std::sync::Mutex<std::collections::HashMap<CacheKey, PathBuf>>,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));
static IN_FLIGHT_BUILDS: once_cell::sync::Lazy<
    std::sync::Mutex<std::collections::HashMap<CacheKey, Vec<Sender<PathBuf>>>>,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

pub fn build_crate(
    src: PathBuf,
    example: Option<String>,
    target_type: HostTargetType,
    features: Option<Vec<String>>,
) -> JoinHandle<PathBuf> {
    let key = (
        src.clone(),
        example.clone(),
        target_type,
        features.clone(),
    );
    let mut in_flight = IN_FLIGHT_BUILDS.lock().unwrap();
    if let Some(listeners) = in_flight.get_mut(&key) {
        let (sender, receiver) = async_channel::unbounded();
        listeners.push(sender);
        tokio::task::spawn_blocking(move || receiver.recv_blocking().unwrap())
    } else {
        let cache = BUILD_CACHE.lock().unwrap();
        if let Some(handle) = cache.get(&key) {
            let res = handle.clone();
            tokio::task::spawn_blocking(move || res)
        } else {
            let (sender, receiver) = async_channel::unbounded();
            in_flight.insert(key.clone(), vec![sender]);
            drop(in_flight);
            drop(cache);

            tokio::task::spawn_blocking(move || {
                let config = Config::default().unwrap();
                config.shell().set_verbosity(cargo::core::Verbosity::Normal);

                let workspace = Workspace::new(&src, &config).unwrap();

                let mut compile_options = CompileOptions::new(&config, CompileMode::Build).unwrap();
                compile_options.filter = CompileFilter::Only {
                    all_targets: false,
                    lib: LibRule::Default,
                    bins: FilterRule::Just(vec![]),
                    examples: FilterRule::Just(vec![example.unwrap()]),
                    tests: FilterRule::Just(vec![]),
                    benches: FilterRule::Just(vec![]),
                };

                compile_options.build_config = BuildConfig::new(
                    &config,
                    None,
                    false,
                    &(match target_type {
                        HostTargetType::Local => vec![],
                        HostTargetType::Linux => vec!["x86_64-unknown-linux-musl".to_string()],
                    }),
                    CompileMode::Build,
                )
                .unwrap();

                compile_options.build_config.requested_profile = InternedString::from("release");
                compile_options.cli_features =
                    CliFeatures::from_command_line(&features.unwrap_or_default(), false, true)
                        .unwrap();

                let res = cargo::ops::compile(&workspace, &compile_options).unwrap();
                let binaries = res
                    .binaries
                    .iter()
                    .map(|b| b.path.to_string_lossy())
                    .collect::<Vec<_>>();

                if binaries.len() == 1 {
                    let mut in_flight = IN_FLIGHT_BUILDS.lock().unwrap();
                    let listeners = in_flight.get_mut(&key).unwrap();
                    for listener in listeners {
                        listener
                            .send_blocking(binaries[0].to_string().into())
                            .unwrap();
                    }

                    let mut cache = BUILD_CACHE.lock().unwrap();
                    cache.insert(key, binaries[0].to_string().into());
                } else {
                    panic!("expected exactly one binary, got {}", binaries.len())
                }
            });

            tokio::task::spawn_blocking(move || receiver.recv_blocking().unwrap())
        }
    }
}
