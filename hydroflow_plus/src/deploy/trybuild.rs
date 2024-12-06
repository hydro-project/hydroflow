use std::fs;
use std::path::PathBuf;

use hydroflow_lang::graph::{partition_graph, HydroflowGraph};
use sha2::{Digest, Sha256};
use stageleft::internal::quote;
use syn::visit_mut::VisitMut;
use trybuild_internals_api::cargo::{self, Metadata};
use trybuild_internals_api::env::Update;
use trybuild_internals_api::run::{PathDependency, Project};
use trybuild_internals_api::{dependencies, features, path, Runner};

use super::trybuild_rewriters::{ReplaceCrateNameWithStaged, ReplaceCrateWithOrig};

static IS_TEST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn init_test() {
    IS_TEST.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn clean_name_hint(name_hint: &str) -> String {
    name_hint
        .replace("::", "__")
        .replace(" ", "_")
        .replace(",", "_")
        .replace("<", "_")
        .replace(">", "")
        .replace("(", "")
        .replace(")", "")
}

pub fn create_graph_trybuild(
    graph: HydroflowGraph,
    extra_stmts: Vec<syn::Stmt>,
    name_hint: &Option<String>,
) -> (String, (PathBuf, PathBuf, Option<Vec<String>>)) {
    let source_dir = cargo::manifest_dir().unwrap();
    let source_manifest = dependencies::get_manifest(&source_dir).unwrap();
    let crate_name = &source_manifest.package.name.to_string().replace("-", "_");

    let is_test = IS_TEST.load(std::sync::atomic::Ordering::Relaxed);

    let mut generated_code = compile_graph_trybuild(graph, extra_stmts);

    ReplaceCrateNameWithStaged {
        crate_name: crate_name.clone(),
    }
    .visit_file_mut(&mut generated_code);

    let mut inlined_staged = stageleft_tool::gen_staged_trybuild(
        &path!(source_dir / "src" / "lib.rs"),
        crate_name.clone(),
        is_test,
    );

    ReplaceCrateWithOrig {
        crate_name: crate_name.clone(),
    }
    .visit_file_mut(&mut inlined_staged);

    let source = prettyplease::unparse(&syn::parse_quote! {
        #generated_code

        #[allow(
            unused,
            ambiguous_glob_reexports,
            clippy::suspicious_else_formatting,
            unexpected_cfgs,
            reason = "generated code"
        )]
        pub mod __staged {
            #inlined_staged
        }
    });

    let hash = format!("{:X}", Sha256::digest(&source))
        .chars()
        .take(8)
        .collect::<String>();

    let bin_name = if let Some(name_hint) = &name_hint {
        format!("{}_{}", clean_name_hint(name_hint), &hash)
    } else {
        hash
    };

    let trybuild_created = create_trybuild(&source, &bin_name, is_test).unwrap();
    (bin_name, trybuild_created)
}

pub fn compile_graph_trybuild(graph: HydroflowGraph, extra_stmts: Vec<syn::Stmt>) -> syn::File {
    let partitioned_graph = partition_graph(graph).expect("Failed to partition (cycle detected).");

    let mut diagnostics = Vec::new();
    let tokens = partitioned_graph.as_code(
        &quote! { hydroflow_plus::hydroflow },
        true,
        quote!(),
        &mut diagnostics,
    );

    let source_ast: syn::File = syn::parse_quote! {
        #![allow(unused_imports, unused_crate_dependencies, missing_docs, non_snake_case)]
        use hydroflow_plus::*;

        #[allow(unused)]
        fn __hfplus_runtime<'a>(__hydroflow_plus_trybuild_cli: &'a hydroflow_plus::hydroflow::util::deploy::DeployPorts<hydroflow_plus::deploy::HydroflowPlusMeta>) -> hydroflow_plus::hydroflow::scheduled::graph::Hydroflow<'a> {
            #(#extra_stmts)*
            #tokens
        }

        #[tokio::main]
        async fn main() {
            let ports = hydroflow_plus::hydroflow::util::deploy::init_no_ack_start().await;
            let flow = __hfplus_runtime(&ports);
            println!("ack start");
            hydroflow_plus::hydroflow::util::deploy::launch_flow(flow).await;
        }
    };
    source_ast
}

pub fn create_trybuild(
    source: &str,
    bin: &str,
    is_test: bool,
) -> Result<(PathBuf, PathBuf, Option<Vec<String>>), trybuild_internals_api::error::Error> {
    let Metadata {
        target_directory: target_dir,
        workspace_root: workspace,
        packages,
    } = cargo::metadata()?;

    let source_dir = cargo::manifest_dir()?;
    let mut source_manifest = dependencies::get_manifest(&source_dir)?;

    if !is_test {
        source_manifest.dev_dependencies.clear();
    }

    let mut features = features::find();

    let path_dependencies = source_manifest
        .dependencies
        .iter()
        .filter_map(|(name, dep)| {
            let path = dep.path.as_ref()?;
            if packages.iter().any(|p| &p.name == name) {
                // Skip path dependencies coming from the workspace itself
                None
            } else {
                Some(PathDependency {
                    name: name.clone(),
                    normalized_path: path.canonicalize().ok()?,
                })
            }
        })
        .collect();

    let crate_name = source_manifest.package.name.clone();
    let project_dir = path!(target_dir / "hfplus_trybuild" / crate_name /);
    fs::create_dir_all(&project_dir)?;

    let project_name = format!("{}-hfplus-trybuild", crate_name);
    let mut manifest = Runner::make_manifest(
        &workspace,
        &project_name,
        &source_dir,
        &packages,
        &[],
        source_manifest,
    )?;

    manifest.features.remove("stageleft_devel");

    if let Some(enabled_features) = &mut features {
        enabled_features
            .retain(|feature| manifest.features.contains_key(feature) || feature == "default");

        manifest
            .features
            .get_mut("default")
            .iter_mut()
            .for_each(|v| {
                v.retain(|f| f != "stageleft_devel");
            });
    }

    let project = Project {
        dir: project_dir,
        source_dir,
        target_dir,
        name: project_name,
        update: Update::env()?,
        has_pass: false,
        has_compile_fail: false,
        features,
        workspace,
        path_dependencies,
        manifest,
        keep_going: false,
    };

    let manifest_toml = toml::to_string(&project.manifest)?;
    fs::write(path!(project.dir / "Cargo.toml"), manifest_toml)?;

    fs::create_dir_all(path!(project.dir / "src" / "bin"))?;

    let out_path = path!(project.dir / "src" / "bin" / format!("{bin}.rs"));
    if !out_path.exists() || fs::read_to_string(&out_path)? != source {
        fs::write(
            path!(project.dir / "src" / "bin" / format!("{bin}.rs")),
            source,
        )?;
    }
    // TODO(shadaj): garbage collect this directory occasionally

    let workspace_cargo_lock = path!(project.workspace / "Cargo.lock");
    if workspace_cargo_lock.exists() {
        let _ = fs::copy(workspace_cargo_lock, path!(project.dir / "Cargo.lock"));
    } else {
        let _ = cargo::cargo(&project).arg("generate-lockfile").status();
    }

    let workspace_dot_cargo_config_toml = path!(project.workspace / ".cargo" / "config.toml");
    if workspace_dot_cargo_config_toml.exists() {
        let dot_cargo_folder = path!(project.dir / ".cargo");
        fs::create_dir_all(&dot_cargo_folder)?;

        let _ = fs::copy(
            workspace_dot_cargo_config_toml,
            path!(dot_cargo_folder / "config.toml"),
        );
    }

    Ok((
        project.dir.as_ref().into(),
        path!(project.target_dir / "hfplus_trybuild"),
        project.features,
    ))
}
