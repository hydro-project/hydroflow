use std::collections::HashMap;

use async_process::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TerraformBatch {
    pub terraform: TerraformConfig,
    pub resource: HashMap<String, HashMap<String, serde_json::Value>>,
    pub output: HashMap<String, TerraformOutput>,
}

impl Default for TerraformBatch {
    fn default() -> TerraformBatch {
        TerraformBatch {
            terraform: TerraformConfig {
                required_providers: HashMap::new(),
            },
            resource: HashMap::new(),
            output: HashMap::new(),
        }
    }
}

impl TerraformBatch {
    pub async fn provision(self) -> TerraformResult {
        let dothydro_folder = std::env::current_dir().unwrap().join(".hydro");
        std::fs::create_dir_all(&dothydro_folder).unwrap();
        let deployment_folder = tempfile::tempdir_in(dothydro_folder).unwrap();

        if self.terraform.required_providers.is_empty()
            && self.resource.is_empty()
            && self.output.is_empty()
        {
            return TerraformResult {
                outputs: HashMap::new(),
                deployment_folder,
            };
        }

        std::fs::write(
            deployment_folder.path().join("main.tf.json"),
            serde_json::to_string(&self).unwrap(),
        )
        .unwrap();

        if !Command::new("terraform")
            .current_dir(deployment_folder.path())
            .arg("init")
            .spawn()
            .unwrap()
            .status()
            .await
            .expect("Failed to spawn terraform init command")
            .success()
        {
            panic!("Failed to initialize terraform");
        }

        let mut result = TerraformResult {
            outputs: HashMap::new(),
            deployment_folder,
        };

        if !Command::new("terraform")
            .current_dir(result.deployment_folder.path())
            .arg("apply")
            .arg("-auto-approve")
            .spawn()
            .unwrap()
            .status()
            .await
            .expect("Failed to spawn terraform apply command")
            .success()
        {
            panic!("Failed to apply terraform");
        }

        let output = Command::new("terraform")
            .current_dir(result.deployment_folder.path())
            .arg("output")
            .arg("-json")
            .output()
            .await
            .expect("Failed to spawn terraform output command");

        result.outputs = serde_json::from_slice(&output.stdout).unwrap();

        result
    }
}

#[derive(Serialize, Deserialize)]
pub struct TerraformConfig {
    pub required_providers: HashMap<String, TerraformProvider>,
}

#[derive(Serialize, Deserialize)]
pub struct TerraformProvider {
    pub source: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct TerraformOutput {
    pub value: String,
}

pub struct TerraformResult {
    pub outputs: HashMap<String, TerraformOutput>,
    pub deployment_folder: tempfile::TempDir,
}

impl Drop for TerraformResult {
    fn drop(&mut self) {
        println!(
            "Destroying terraform deployment at {}",
            self.deployment_folder.path().display()
        );
        if !std::process::Command::new("terraform")
            .current_dir(&self.deployment_folder)
            .arg("destroy")
            .arg("-auto-approve")
            .spawn()
            .expect("Failed to spawn terraform destroy command")
            .wait()
            .expect("Failed to destroy terraform deployment")
            .success()
        {
            panic!("Failed to destroy terraform deployment");
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TerraformResultOutput {
    value: String,
}
