use std::{
    collections::HashMap,
    path::PathBuf,
    process::Command,
    process::Stdio,
    sync::{Arc, Mutex},
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
            let mut build_args = vec!["build".to_string(), "--release".to_string()];

            if let Some(example) = example.as_ref() {
                build_args.push("--example".to_string());
                build_args.push(example.clone());
            }

            match target_type {
                HostTargetType::Local => {}
                HostTargetType::Linux => {
                    build_args.push("--target".to_string());
                    build_args.push("x86_64-unknown-linux-musl".to_string());
                }
            }

            if let Some(features) = features {
                build_args.push("--features".to_string());
                build_args.push(features.join(","));
            }

            build_args.push("--message-format=json-render-diagnostics".to_string());

            let mut command = Command::new("cargo")
                .args(&build_args)
                .current_dir(&src)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let reader = std::io::BufReader::new(command.stdout.take().unwrap());
            for message in cargo_metadata::Message::parse_stream(reader) {
                match message.unwrap() {
                    cargo_metadata::Message::CompilerArtifact(artifact) => {
                        let is_output = if example.is_some() {
                            artifact.target.kind.contains(&"example".to_string())
                        } else {
                            artifact.target.kind.contains(&"bin".to_string())
                        };

                        if is_output {
                            let path = artifact.executable.unwrap();
                            let path = path.into_string();
                            let data = std::fs::read(path).unwrap();
                            return Arc::new((nanoid!(8), data));
                        }
                    }
                    cargo_metadata::Message::CompilerMessage(msg) => {
                        eprintln!("{}", msg.message.rendered.unwrap())
                    }
                    _ => {}
                }
            }

            if command.wait().unwrap().success() {
                panic!("cargo build succeeded but no binary was emitted")
            } else {
                panic!("failed to build crate")
            }
        })
        .clone()
}
