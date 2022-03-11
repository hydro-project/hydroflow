# Hydroflow

Hydro's low-level dataflow runtime.

## Dev Setup

This sections explains how to get Hydroflow running, either for development or
usage, even if you are not familiar with Rust development.

### Installing Rust

First you will need to install Rust. We recommend the conventional installation
method, `rustup`, which allows you to easily manage and update Rust versions.

[**Install Rust**](https://www.rust-lang.org/tools/install)

This will install `rustup` and the Rust package manager `cargo` (and the
internally-used `rustc` compiler). `cargo` is Rust's main development tool,
used for building, running, and testing Rust code.

The following `cargo` commands will come in handy:
* `cargo check --all-targets` - Checks the workspace for any compile-time
  errors.
* `cargo build --all-targets` - Builds all projects/tests/benchmarks/examples
  in the workspace.
* `cargo clean` - Cleans the build cache, sometimes needed if the build is
  acting up.
* `cargo test` - Runs tests in the workspace.
* `cargo run -p hydroflow --example <example name>` - Run an example program in
  `hydroflow/examples`.

To learn Rust see the official [Learn Rust page](https://www.rust-lang.org/learn).
Here are some good resources:
* [_The Rust Programming Language_, AKA "The Book"](https://doc.rust-lang.org/book/)
* [Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)
  is a good way to learn Rust's ownership system and its
  implications.

### VS Code Setup

We recommend using VS Code with the `rust-anaylzer` extension (and NOT the
`Rust` extension). To enable the pre-release version of `rust-analyzer`
(required by some new nightly syntax we use, at least until 2022-03-14), click
the "Switch to Pre-Release Version" button next to the uninstall button.

### Hydroflow Setup

The easiest way to get started with Hydroflow is to clone and work in the
repository directly. You should fork the repository if you want to push your
changes.
```bash
git clone git@github.com:hydro-project/hydroflow.git
```
Hydroflow requires nightly Rust, but the repo is already configured for it via
`rust-toolchain.toml`.

We can then open the repo in VS Code or IDE of your choice. `rust-anaylzer`
will provide inline type and error messages, code completion, etc.

### Development

The easiest way to try Hydroflow is with an "example", found in the
[`hydroflow/examples` folder](https://github.com/hydro-project/hydroflow/tree/main/hydroflow/examples).
These examples are include via the [`hydroflow/Cargo.toml` file](https://github.com/hydro-project/hydroflow/blob/main/hydroflow/Cargo.toml),
so make sure to add your example there if you create a new one. The simplest
example is [`graph_reachability`](https://github.com/hydro-project/hydroflow/blob/main/hydroflow/examples/graph_reachability/main.rs).

The Hydroflow repository is set up as a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html),
i.e. a repo containing a bunch of separate packages, `hydroflow` is just the
main one. So if you want to work in a proper separate cargo package, you can
create one and add it into the [root `Cargo.toml`](https://github.com/hydro-project/hydroflow/blob/main/Cargo.toml).
