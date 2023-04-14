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

### Managing Rust Toolchains
The CLI will use its own copy of `cargo` to build any Rust code. This means that the Rust toolchain used by the CLI is the one on your `$PATH`. If using `rustup` to manage toolchains, you will need to run the deployment with the appropriate toolchain activated. For example, to use the toolchain in `rust-toolchain.toml`, first run:
```bash
$ rustup show active-toolchain
# nightly-2023-04-13-aarch64-apple-darwin (environment override by RUSTUP_TOOLCHAIN)
```

Then, copying the toolchain name, run:
```bash
rustup run nightly-2023-04-13-aarch64-apple-darwin hydro deploy ...
```

## Examples

### Tick + Counter
Run the following in `hydro_cli_examples`:
```bash
rustup run nightly hydro deploy tick.hydro.py -- <location>
```
`<location>`: Either `local` to run on your own machine or `gcp` to launch in the cloud.

This program increments a counter and checks the current tick, once per tick, and outputs the value of the counter every second.

### Echo
Run the following in `hydro_cli_examples`:
```bash
rustup run nightly hydro deploy echo.hydro.py -- <leader_location> <participant_location>
```
`<leader_location>`: Either `local` to run on your own machine or `gcp` to launch the leader in the cloud.  
`<participant_location>`: Either `local` to run on your own machine or `gcp` to launch the participant in the cloud.

The leader uses the current tick as a message and sends it to the participant, which echoes the message. The leader outputs the cumulative number of echoed messages every second.

### Vote
Run the following in `hydro_cli_examples`:
```bash
rustup run nightly hydro deploy vote.hydro.py -- <leader_location> <participant_location> <num_participants>
```
`<leader_location>`: Either `local` to run on your own machine or `gcp` to launch the leader in the cloud.  
`<participant_location>`: Either `local` to run on your own machine or `gcp` to launch the participant in the cloud.  
`<num_participants>`: The number of participants to launch.

The leader uses the current tick as a message and sends it to all participants, which echoes the message. The leader outputs the cumulative number of messages that have been echoed by each participant every second.

### 2PC
Run the following in `hydro_cli_examples`:
```bash
rustup run nightly hydro deploy 2pc.hydro.py -- <leader_location> <participant_location> <num_participants>
```
`<leader_location>`: Either `local` to run on your own machine or `gcp` to launch the leader in the cloud.  
`<participant_location>`: Either `local` to run on your own machine or `gcp` to launch the participant in the cloud.  
`<num_participants>`: The number of participants to launch.

Similar to [Vote](#vote), but there are two rounds.

### Paxos
Run the following in `hydro_cli_examples`:
```bash
rustup run nightly hydro deploy paxos.hydro.py -- <proposer_location> <acceptor_location> <f> <p1a_node_0_timeout> <p1a_other_nodes_timeout> <i_am_leader_resend_timeout> <i_am_leader_check_timeout>
```
`<proposer_location>`: Either `local` to run on your own machine or `gcp` to launch the proposers in the cloud.  
`<acceptor_location>`: Either `local` to run on your own machine or `gcp` to launch the acceptors in the cloud.  
`<f>`: The maximum number of failures. There are f+1 proposers and 2f+1 acceptors.  
`<p1a_node_0_timeout>`: How often proposer 0 sends `p1a` messages for leader election. We overload it to also output the cumulative number of committed messages.  
`<p1a_other_nodes_timeout>`: How often the proposers send `p1a` messages for leader election. Should set to a number far higher than `<p1a_node_0_timeout>` to avoid contention.  
`<i_am_leader_resend_timeout>`: The timeout for the leader to resend an `IAmLeader` message.  
`<i_am_leader_check_timeout>`: The timeout for the leader to check if another `IAmLeader` message has arrived. Should set to a number far higher than `<i_am_leader_resend_timeout>` to avoid preempting correct leaders.

I recommend the following configuration:
```bash
rustup run nightly hydro deploy paxos.hydro.py -- local local 1 1 10 3 8
```

Implements MultiPaxos. The leader outputs the cumulative number of committed messages every second. Frequency can be adjusted with `<p1a_node_0_timeout>`. Can change to output the cumulative number of writes to acceptors every second by modifying the rules that write to the `throughputOut` relation.