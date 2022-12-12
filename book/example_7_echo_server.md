# Networked Services 1: EchoServer
> In this example we cover:
> * A standard project template for building networked Hydroflow services.
> * Rust's `clap` crate for command-line options
> * Defining message types
> * Using network sources and sinks with built-in serde.
> * The `source_stdin` source
> * Long-running services via `run_async`

Our examples up to now have been simple single-node programs, to get us comfortable with Hydroflow's
surface syntax. But the whole point of Hydroflow is to help us write distributed programs or services that run on a cluster of machines!

In this example we'll build the "hello, world" of distributed systems -- a simple echo server. It will listen on a UDP port,
and send back a copy of any message it receives, with a timestamp. We will also build a client to 
accept strings from the command line, send them to the echo server, and print responses.

Full code for this example can be found in `hydroflow/hydroflow/examples/echoserver`. This example can 
serve as a template for many networked Hydroflow services.  

Generally the directory structure we're advocating is:
```txt
project/README.md       # documentation
project/main.rs         # main function
project/protocol.rs     # message types exchanged between roles
project/helpers.rs      # helper functions used by all roles
project/roleA.rs        # service definition for role A
project/roleB.rs        # service definition for role B
```
In this example, the roles we'll be using are `Client` and `Server`, but you can imagine different roles depending on the structure of your service or application.

## main.rs
We start with a `main` function that parses command-line options, and invokes the appropriate
role-specific service.
After a prelude of imports, we start by defining an `enum` for the `Role`s that the service supports. 

```rust, ignore
use clap::{ArgEnum, Parser};
use client::run_client;
use hydroflow::tokio;
use hydroflow::util::{bind_udp_socket, ipv4_resolve};
use server::run_server;

mod client;
mod protocol;
mod server;

#[derive(Clone, ArgEnum, Debug)]
enum Role {
    Client,
    Server,
}
```

Following that, we use Rust's [`clap`](https://docs.rs/clap/latest/clap/) (Command Line Argument Parser) crate to parse command-line options:

```rust,ignore
#[derive(Parser, Debug)]
struct Opts {
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    addr: Option<String>,
    #[clap(long)]
    server_addr: String,
}
```
This sets up 3 command-line options: `role`, `addr`, and `server_addr`. The `addr` option is optional, but the `role` and `server_addr` options are required. The `clap` crate will parse the command-line options and populate the `Opts` struct with the values.

This brings us to the `main` function itself. It is prefaced by a `#[tokio::main]` attribute, which is a macro that sets up the tokio runtime. This is necessary because Hydroflow uses the tokio runtime for asynchronous execution.  

```rust,ignore
#[tokio::main]
async fn main() {
    // parse command line arguments
    let opts = Opts::parse();
```

After parsing the command line arguments, we get into invoking the client or server code. Before we do so, we set up some Rust-based networking. Specifically, in both cases we will need to allocate a UDP socket that is used for both sending and receiving messages. We do this by calling the async `bind_udp_socket` function, which is defined in `hydroflow/src/net.rs`. It is an async `future`, so requires appending `.await`; the function returns a pair of type `(UdpSink, UdpSource)`. These are the types that we'll use in Hydroflow to send and receive messages. (Note: your IDE might expand out the `UdpSink` and `UdpSource` traits to their more verbose definitions. This is fine; you can ignore for now.)

For the server case, all that's left is to invoke `run_server` and pass it the network information. Note that the server is also an asynchronous program, so we append `.await` to that call as well. The program will block on this call until the server is done (which should only happen when it fails).
```rust,ignore
    // depending on the role, pass in arguments to the right function
    match opts.role {
        Role::Server => {
            // allocate `outbound` and `inbound` sockets
            let (outbound, inbound) = bind_udp_socket(opts.server_addr.clone()).await;
            // run the server
            run_server(outbound, inbound, opts.graph.clone()).await;
        }
```

In the client case, we need one more piece of information passed down: the address of the server. We get this by calling the `ipv4_resolve` function, which is also defined in `src/net.rs`. This function takes a string and returns a `SocketAddr` type, which is the type that the `UdpSink` and `UdpSource` traits expect. Invoking `run_client` is similar to the server case, except that we pass in the server address as well.
```rust,ignore
        Role::Client => {
            // resolve the server's IP address
            let server_addr = ipv4_resolve(opts.server_addr.clone());
            // allocate `outbound` and `inbound` sockets
            let (outbound, inbound) = bind_udp_socket(opts.addr.clone().unwrap()).await;
            // run the client
            run_client(outbound, inbound, server_addr).await;
        }
    }
}
```

## protocol.rs
As a design pattern, it is natural in distributed Hydroflow programs to define various message types in a `protocol.rs` file with structures shared for use by all the Hydroflow logic across roles. In this simple example, we define only one message type: `EchoMsg`, and a simple struct with two fields: `payload` and `ts` (timestamp). The `payload` field is a string, and the `ts` field is a `DateTime<Utc>`, which is a type from the [`chrono`](https://docs.rs/chrono/latest/chrono/) crate. Note the various derived traits on `EchoMsg` -- these are required for structs that we send over the network.

```rust,ignore
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct EchoMsg {
    pub payload: String,
    pub ts: DateTime<Utc>,
}
```

# server.rs
Things get interesting when we look at the `run_server` function. This function is the main entry point for the server. It takes as arguments the `outbound` and `inbound` sockets, and the `graph` type. The `outbound` and `inbound` sockets are the same ones that we allocated in `main.rs`. The `graph` type is an enum that we defined in `main.rs`, and is used to control whether Hydroflow emits a dataflow diagram.

After printing a cheery message, we get the surface syntax for the server, consisting of three short pipelines:

```rust
use crate::protocol::EchoMsg;
use chrono::prelude::*;
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};
use std::net::SocketAddr;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream) {
    println!("Server live!");


   let mut flow: Hydroflow = hydroflow_syntax! {
        // Inbound channel sharing
        inbound_chan = source_stream_serde(inbound) -> tee();

        // Logic
        inbound_chan[0] -> for_each(|(m, a): (EchoMsg, SocketAddr)| println!("Got {:?} from {:?}", m, a));
        inbound_chan[1] -> map(|(EchoMsg { payload, .. }, addr)| (EchoMsg { payload, ts: Utc::now() }, addr))
            -> dest_sink_serde(outbound);
    };
```
Lets take these one at a time. 

The first pipeline, `inbound_chan` uses a source operator we have not seen before, [`source_stream_serde()`](./surface_ops.gen.md#source_stream_serde). This is a streaming source like `source_stream`, but for network streams. It takes a `UdpSource` as an argument, and has a particular output type: a stream of `(T, SocketAddr)` pairs where `T` is some type that implements the `Serialize` and `Deserialize` traits, and `SocketAddr` is the network address of the sender of the item. In this case, `T` is `EchoMsg`, which we defined in `protocol.rs`, and the `SocketAddr` is the address of the client that sent the message. We pipe the result into a `tee()` for reuse.

The second pipeline is a simple `for_each` to print the messages received at the server.

The third and final pipeline constructs a response `EchoMsg` with the local timestamp copied in. It then pipes the result into a sink operator we have not seen before, [`dest_sink_serde()`](./surface_ops.gen.md#dest_sink_serde). This is a sink operator like `dest_sink`, but for network streams. It takes a `UdpSink` as an argument, and requires a particular input type: a stream of `(T, SocketAddr)` pairs where `T` is some type that implements the `Serialize` and `Deserialize` traits, and `SocketAddr` is the network address of the destination. In this case, `T` is once again `EchoMsg`, and the `SocketAddr` is the address of the client that sent the original message.

The remaining line of code runs the server. The `run_async()` function is a method on the `Hydroflow` type. It is an async function, so we append `.await` to the call. The program will block on this call until the server is done.

```rust,ignore
    // run the server
    flow.run_async().await;
}
```

## client.rs
The client is as simple as the server, consisting only of two pipelines. The first uses another new source operator [`source_stdin()`](./surface_ops.gen.md#source_stdin), which does what you might expect: streams lines of text as they arrive from `stdin` (i.e. as they are typed into a terminal). It then uses a `map` to construct an `EchoMsg` with the current timestamp. The result is piped into a sink operator [`dest_sink_serde()`](./surface_ops.gen.md#dest_sink_serde), which sends the message to the server. The second operator is a `for_each` that prints the messages echoed back from the server.

```rust,ignore
use crate::protocol::EchoMsg;
use chrono::prelude::*;
use hydroflow::hydroflow_syntax;
use hydroflow::util::{UdpSink, UdpStream};
use std::net::SocketAddr;

pub(crate) async fn run_client(outbound: UdpSink, inbound: UdpStream, server_addr: SocketAddr) {
    println!("Attempting to connect to server at {:?}", server_addr);
    println!("Client live!");

let mut flow = hydroflow_syntax! {
    // take stdin and send to server as an Echo::Message
    source_stdin() -> map(|l| (EchoMsg{ payload: l.unwrap(), ts: Utc::now(), }, server_addr) )
        -> dest_sink_serde(outbound);

    // receive and print messages
    source_stream_serde(inbound) -> for_each(|(m, _a): (EchoMsg, SocketAddr) | println!("{:?}", m));
};
```

## Running the example
As described in `hydroflow/hydroflow/example/echoserver/README.md`, we can run the server in one terminal, and the client in another. The server will print the messages it receives, and the client will print the messages it receives back from the server. The client and servers' `--server-addr' arguments need to match or this won't work!

Fire up the server in terminal 1:
```console
% cargo run -p hydroflow --example echoserver -- --role server --server-addr localhost:12347
```

Then start the client in terminal 2 and type some messages!
```console
% cargo run -p hydroflow --example echoserver -- --role client --addr localhost:9090 --server-addr localhost:12347
Attempting to connect to server at 127.0.0.1:12347
Client live!
This is a test 
EchoMsg { payload: "This is a test", ts: 2022-12-12T23:42:13.053293Z }
This is the rest!
EchoMsg { payload: "This is the rest!", ts: 2022-12-12T23:42:20.181371Z }
```

And have a look back at the server console!
```console
Server live!
Got EchoMsg { payload: "This is a test", ts: 2022-12-12T23:42:13.049499Z } from 127.0.0.1:9090
Got EchoMsg { payload: "This is the rest!", ts: 2022-12-12T23:42:20.179337Z } from 127.0.0.1:9090
```
