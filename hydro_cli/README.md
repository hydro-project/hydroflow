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
The CLI can be used as a command line tool. To see the available commands, run:
```bash
$ hydro --help
```

### Python Package
The CLI can be imported into other Python programs. To use it, import the `hydro` package:
```python
import hydro
```

See `test.hydro.py` for an example of how to use the available APIs.
