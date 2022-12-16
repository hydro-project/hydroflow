## Getting Started
This is a template for a Rust project that uses [Hydroflow](http://github.com/hydro-project/hydroflow) for distributed services. To install, simply run 
```bash
cargo generate hydro-project/hydroflow-template
```
You will be prompted to name your project. Once the command completes, you can cd into the project and build the template.
```bash
cd <myproject>
cargo build
```

## Launching the Template Application
The template assumes that there are distinct "roles" (classes of service) that can be launched via the same executable. 
By default, the template provides a `server` and `client` role; these are easily overridden in the `Opts` struct in `src/main.rs`.
Command line arguments allow you to launch with a specific role (`--role`) and an address to bind to (`--addr`). 
If you don't wish to choose an address or port number, you can simply omit the `--addr` argument and the service 
will bind to a random port on localhost. If you want separate executables, you can use this template multiple times to generate
a separate project for each executable.

By default, the template also allows you to optionally specify the address of a remote server (`--server-addr`) and 
a type of dataflow graph to be emitted (`--graph`); 
these can be removed from the `Opts` struct in `src/main.rs` if they are not needed. 


To launch a service instance manually, it is sufficient to run the following command:
```console
% cargo run -- --role <role>
```
where `<role>` is the role of the service (e.g. `server` or `client`). In our client-server setup, 
the client role must also be provided with the address of the server to connect to (`--server-addr`).

For testing its usually helpful to run multiple instances in separate terminals.
Once your code seems to be working correctly, the [hydroplane](https://github.com/hydro-project/hydroplane) project 
provides a framework for launching and managing multiple instances of a service either locally or in a distributed environment.

### Launching the Unmodified Template Project
The provided code implements an echo server and client. To run it unmodified, open 2 terminals.

In one terminal run the server like so:
```console
% cargo run -- --role server --addr localhost:12346
```

In another terminal run a client like so:
```console
% cargo run -- --role client --server-addr localhost:12346
```
The client listens on stdin, and sends (newline-delimited) messages that it receives to the server.
The client also prints any messages it receives to stdout.
Meanwhile, the server waits for messages, which it echoes back to the sender. 

The template also includes an optional command-line argument to print out a dataflow graph of the hydroflow code.
Adding the `--graph <graph_type>` flag to the end of the command lines above will print out a node-and-edge diagram of the chosen service. 
Supported values for `<graph_type>` include [mermaid](https://mermaid-js.github.io/) and [dot](https://graphviz.org/doc/info/lang.html).

## Structure of the Template Project
The `src` directory contains the following files:

```txt
src/main.rs         - This file contains the `main` function, which handles command-line arguments and launches the appropriate service.
src/protocol.rs     - This file contains the `Message` enum that defines the messages that can be sent between instances.
src/helpers.rs      - This file contains helper functions that are invoked from Hydroflow code in multiple services
src/<role>.rs       - The code for a service with the given role. Default files are provided for `server` and `client`.
```

The `src/main.rs` file is where the command-line arguments are parsed and the appropriate service is launched.
It also contains the `Opts` struct, which uses the [clap](https://docs.rs/clap/latest/clap/) crate to 
specify the command-line arguments that are accepted by the template.
It is possible to change the command-line arguments by modifying the `Opts` struct.
Before launching the service, the `main` function binds to the specified address and prints out the address that was bound to.

The `src/protocol` file contains the enum `Message`, which can include messages with very different structures. 
The `Message` must provide the `Serialize` and `Deserialize` traits, which are used by the [serde](https://docs.serde.rs/serde/) crate.
In the template, the `Message` enum includes an `Echo` message that has a (`String`) payload and timestamp; it also includes
`Heartbeat` and `HeartbeatAck` messages that carry no information other than their type. (Messages are delivered with the sender 
address attached, so these empty message types can be useful.) 

The `src/helpers.rs` file contains any helper functions that are needed. In our example this is just a function to print
a dataflow graph representation of the hydroflow code to stdout.

Each service file comes with a skeleton Hydroflow spec that provides an inbound communication channel and an outbound communication channel, both 
bound to the specified address and port. The channels are named `inbound_chan` and `outbound_chan`, and are accessed in hydroflow code using the 
`source_serde` and `sink_serde` operators respectively. The single address/port pair is sufficient in general to support multiple
different `Message` types across multiple services and instances. Upon receipt, messages are handled by the appropriate code using hydroflow's `demux`
operator. It is also possible to open more channels to segregate traffic to different IP addresses or ports, simply by copying the 
patterns that define and use the `inbound` and `outbound` channels.

Each service file also includes code to generate the dataflow graph for the service, if the `--graph` flag is provided on the command line.
The ASCII spec for the graph is printed to stdout on launch.

## Communication Patterns
No particular communication pattern is assumed by Hydroflow. The unmodified template is designed to be used in a "star topology": 
multiple independent clients talking to a single server. However, the template can be easily modified to support other topologies. 
Additional examples are provided in the [hydroflow](https://github.com/hydro-project/hydroflow) repository in the `hydroflow/examples` directory.

## Where do you go from here?
This template is intended to be a starting point for your own project. You'll undoubtedly want to change it.

In our experience, when starting a Hydroflow project we recommend a four-step approach:

1. **Roles**: Identify the roles that your services will play (in the `Opts` struct in `src/main.rs`)
2. **Messages**: Define the basic message types that services will send to each other (in the `Message` enum in `src/protocol.rs`).
3. **Print Received Messages**: Utilize the template logic at each service that prints out messages received. 
4. **Exercise Sending Patterns**:  Make sure the right messages get to the right recipients! Write simple logic to send out messages in all the message patterns you expect to see (in the `src/<role>.rs` files).
5. **Service Programming**: Begin writing the actual logic for each service, with plenty of `for_each(|m| println!("{:?}", m))` operators 
peppered throughout!

Have fun!
