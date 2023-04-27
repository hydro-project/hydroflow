# Hydro CLI
The Hydro CLI is a library and command line tool for provisioning machines and launching distributed Hydro programs on them. Currently, only macOS/Linux are supported.

## Installation
First, create a virtual environment and activate it:
```bash
$ python3 -m venv .venv
$ source .venv/bin/activate
```
Note: You will need to run `source .venv/bin/activate` in the `hydro_cli` directory every time you open a new terminal.

Then, install Maturin:
```bash
$ pip install maturin
$ cargo install maturin # alternative that works outside the venv
```

Then, build the CLI for local use:
```bash
$ maturin develop
```
Note: You will need to run `maturin develop` every time you make changes to Hydroflow code.

You can begin running examples in `hydro_cli_examples`, each with their own instructions in the [Examples](#examples) section.

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
