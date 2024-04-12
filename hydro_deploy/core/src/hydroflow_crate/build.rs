use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use cargo_metadata::diagnostic::Diagnostic;
use nanoid::nanoid;
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;

use crate::progress::ProgressTracker;
use crate::{HostTargetType, LinuxArchitecture};

#[derive(PartialEq, Eq, Hash)]
struct CacheKey {
    src: PathBuf,
    bin: Option<String>,
    example: Option<String>,
    profile: Option<String>,
    target_type: HostTargetType,
    features: Option<Vec<String>>,
}

pub type BuiltCrate = Arc<(String, Vec<u8>, PathBuf)>;

static BUILDS: Lazy<Mutex<HashMap<CacheKey, Arc<OnceCell<BuiltCrate>>>>> =
    Lazy::new(Default::default);

pub async fn build_crate(
    src: impl AsRef<Path>,
    bin: Option<String>,
    example: Option<String>,
    profile: Option<String>,
    target_type: HostTargetType,
    features: Option<Vec<String>>,
) -> Result<BuiltCrate, BuildError> {
    // `fs::canonicalize` prepends windows paths with the `r"\\?\"`
    // https://stackoverflow.com/questions/21194530/what-does-mean-when-prepended-to-a-file-path
    // However, this breaks the `include!(concat!(env!("OUT_DIR"), "/my/forward/slash/path.rs"))`
    // Rust codegen pattern on windows. To help mitigate this happening in third party crates, we
    // instead use `dunce::canonicalize` which is the same as `fs::canonicalize` but avoids the
    // `\\?\` prefix when possible.
    let src = dunce::canonicalize(src).expect("Failed to canonicalize path for build.");

    let key = CacheKey {
        src: src.clone(),
        bin: bin.clone(),
        example: example.clone(),
        profile: profile.clone(),
        target_type,
        features: features.clone(),
    };

    let unit_of_work = {
        let mut builds = BUILDS.lock().unwrap();
        builds.entry(key).or_default().clone()
        // Release BUILDS table lock here.
    };

    unit_of_work
        .get_or_try_init(move || {
            ProgressTracker::rich_leaf("build".to_string(), move |_, set_msg| async move {
                tokio::task::spawn_blocking(move || {
                    let mut command = Command::new("cargo");
                    command.args([
                        "build".to_string(),
                        "--profile".to_string(),
                        profile.unwrap_or("release".to_string()),
                    ]);

                    if let Some(bin) = bin.as_ref() {
                        command.args(["--bin", bin]);
                    }

                    if let Some(example) = example.as_ref() {
                        command.args(["--example", example]);
                    }

                    match target_type {
                        HostTargetType::Local => {}
                        HostTargetType::Linux(LinuxArchitecture::X86_64) => {
                            command.args(["--target", "x86_64-unknown-linux-musl"]);
                        }
                        HostTargetType::Linux(LinuxArchitecture::AARCH64) => {
                            command.args(["--target", "aarch64-unknown-linux-musl"]);
                        }
                    }

                    if let Some(features) = features {
                        command.args(["--features", &features.join(",")]);
                    }

                    command.arg("--message-format=json-diagnostic-rendered-ansi");

                    let mut spawned = command
                        .current_dir(&src)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .stdin(Stdio::null())
                        .spawn()
                        .unwrap();

                    let reader = std::io::BufReader::new(spawned.stdout.take().unwrap());
                    let mut stderr_reader = std::io::BufReader::new(spawned.stderr.take().unwrap());
                    std::thread::spawn(move || loop {
                        let mut buf = String::new();
                        if let Ok(size) = stderr_reader.read_line(&mut buf) {
                            if size == 0 {
                                break;
                            } else {
                                set_msg(buf.trim().to_string());
                            }
                        } else {
                            break;
                        }
                    });

                    let mut diagnostics = Vec::new();
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
                                    let path_buf: PathBuf = path.clone().into();
                                    let path = path.into_string();
                                    let data = std::fs::read(path).unwrap();
                                    return Ok(Arc::new((nanoid!(8), data, path_buf)));
                                }
                            }
                            cargo_metadata::Message::CompilerMessage(msg) => {
                                ProgressTracker::println(msg.message.rendered.as_deref().unwrap());
                                diagnostics.push(msg.message);
                            }
                            _ => {}
                        }
                    }

                    if spawned.wait().unwrap().success() {
                        Err(BuildError::NoBinaryEmitted)
                    } else {
                        Err(BuildError::FailedToBuildCrate(diagnostics))
                    }
                })
                .await
                .map_err(|_| BuildError::TokioJoinError)?
            })
        })
        .await
        .cloned()
}

#[derive(Clone, Debug)]
pub enum BuildError {
    FailedToBuildCrate(Vec<Diagnostic>),
    TokioJoinError,
    NoBinaryEmitted,
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedToBuildCrate(diagnostics) => {
                writeln!(f, "Failed to build crate:")?;
                for diagnostic in diagnostics {
                    write!(f, "{}", diagnostic)?;
                }
            }
            Self::TokioJoinError => {
                write!(f, "Failed to spawn tokio blocking task.")?;
            }
            Self::NoBinaryEmitted => {
                write!(f, "`cargo build` succeeded but no binary was emitted.")?;
            }
        }
        Ok(())
    }
}

impl Error for BuildError {}
