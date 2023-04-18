use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use cargo::{
    core::{compiler::BuildConfig, resolver::CliFeatures, Workspace},
    ops::{CompileFilter, CompileOptions, FilterRule, LibRule},
    util::{command_prelude::CompileMode, interning::InternedString},
    Config,
};
use nanoid::nanoid;
use once_cell::sync::{Lazy, OnceCell};

use crate::core::HostTargetType;

type CacheKey = (PathBuf, Option<String>, HostTargetType, Option<Vec<String>>);

pub type BuildResult = Arc<(String, Vec<u8>)>;

static BUILDS: Lazy<Mutex<HashMap<CacheKey, Arc<OnceCell<BuildResult>>>>> =
    Lazy::new(Default::default);

pub fn build_crate(
    src: PathBuf,
    example: Option<String>,
    target_type: HostTargetType,
    features: Option<Vec<String>>,
) -> Arc<(String, Vec<u8>)> {
    let key = (src.clone(), example.clone(), target_type, features.clone());
    let unit_of_work = {
        let mut builds = BUILDS.lock().unwrap();
        builds.entry(key).or_default().clone()
        // Release BUILDS table lock here.
    };
    unit_of_work
        .get_or_init(|| {
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
                CliFeatures::from_command_line(&features.unwrap_or_default(), false, true).unwrap();

            let res = cargo::ops::compile(&workspace, &compile_options).unwrap();
            let binaries = res
                .binaries
                .iter()
                .map(|b| b.path.to_string_lossy())
                .collect::<Vec<_>>();

            if binaries.len() == 1 {
                Arc::new((nanoid!(8), std::fs::read(binaries[0].to_string()).unwrap()))
            } else {
                panic!("expected exactly one binary, got {}", binaries.len())
            }
        })
        .clone()
}
