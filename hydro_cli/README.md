# Hydro CLI
The Hydro CLI is a library and command line tool for provisioning machines and launching distributed Hydro programs on them. Currently, only macOS/Linux are supported.

## Installation
First, create a virtual environment and activate it:
```bash
$ python3 -m venv .venv
$ source .venv/bin/activate
```

Then, install Maturin:
```bash
$ pip install maturin
$ cargo install maturin # alternative that works outside the venv
```

Then, build the CLI for local use:
```bash
$ maturin develop
```

## Usage
The CLI is a Python package that can be imported into other Python programs. It can also be used as a command line tool.

### Command Line Tool
After building the CLI, it will be automatically available on your path in the virtual environment. To see the available commands, run:
```bash
$ hydro --help
```

### Python Package
The deployment API can be imported into other Python programs. To use it, import the `hydro` package:
```python
import hydro
```

See `test.hydro.py` for an example of how to use the available APIs.

### Managing Rust Toolchains
The CLI will use its own copy of `cargo` to build any Rust code. This means that the Rust toolchain used by the CLI is the one on your `$PATH`. If using `rustup` to manage toolchains, you will need to run the deployment with the appropriate toolchain activated. For example, to use the toolchain in `rust-toolchain.toml`, first run:
```bash
$ rustup show active-toolchain
# nightly-2023-03-01-aarch64-apple-darwin (environment override by RUSTUP_TOOLCHAIN)
```

Then, copying the toolchain name, run:
```bash
rustup run nightly-2023-03-01-aarch64-apple-darwin hydro deploy ...
```
