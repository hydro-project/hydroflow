## Getting Started
This is a template for a Rust project that uses [Hydroflow+](http://github.com/hydro-project/hydroflow) for distributed services. To generate a project, run 

```bash
$ cargo install cargo-generate
$ cargo generate hydro-project/hydroflow-plus-template
```

Once the command completes, you can cd into the project and test the template.
```bash
$ cd <myproject>
$ cargo test
```

## Project Structure
Hydroflow+ uses Stageleft (https://hydro.run/docs/hydroflow_plus/stageleft), and thus requires a special project structure consisting of the main logic (in `flow`) and a auto-generated helper (in `flow_macro`). Other than keeping dependencies in sync between `flow` and `flow_macro`, you should not need to modify `flow_macro` directly.

The template includes two sample programs, `first_ten` and `first_ten_distributed`.

`first_ten` demonstrates how to use Hydroflow+ to create dataflow programs for a single machine, and can be launched by running `cargo run -p flow --bin first_ten`.

`first_ten_distributed` demonstrates how to use Hydroflow+ to create dataflow programs for a distributed system, and can be launched by running `cargo run -p flow --example first_ten_distributed`. Note the use of `--example` here because `src/bin/first_ten_distributed.rs` contains the binary that will be launched for each process, whereas `examples/first_ten_distributed.rs` contains a deployment script for connecting the processes together.

This template also comes with an example of deploying the `first_ten_distributed` flow to Google Cloud. To deploy, you will need to install the [Google Cloud SDK](https://cloud.google.com/sdk/docs/install) and [Terraform](https://developer.hashicorp.com/terraform/install). Then, authenticate with Google Cloud and launch the deployment script with your project ID as an argument:

```bash
$ gcloud auth application-default login
$ cargo run -p flow --example first_ten_distributed_gcp -- YOUR_PROJECT_ID_HERE
```
