# Networked Services 2: Chat Server
> In this example we cover:
> * Multiple message types and the `demux` operator.
> * A broadcast pattern via the `cross_join` operator.
> * One-time bootstrapping pipelines
> * A "gated buffer" pattern via `cross_join` with a single-object input.

Our previous [echo server](./example_7_echo_server.md) example was admittedly simplistic.  In this example, we'll build something a bit more useful: a simple chat server. We will again have two roles: a `Client` and a `Server`. `Clients` will register their presence with the `Server`, which maintains a list of clients. Each `Client` sends messages to the `Server`, which will then broadcast those messages to all other clients. 

## main.rs
The `main.rs` file here is very similar to that of the echo server, just with two new command-line arguments: one for a "nickname" in the chatroom, and another optional argument for printing a dataflow graph if desired.

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
#[derive(Clone, ArgEnum, Debug)]
enum GraphType {
    Mermaid,
    Dot,
    Json,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(long)]
    name: String,
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    port: u16,
    #[clap(long)]
    addr: String,
    #[clap(long)]
    server_addr: Option<String>,
    #[clap(long)]
    server_port: Option<u16>,
    #[clap(arg_enum, long)]
    graph: Option<GraphType>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let server_str = opts.server_addr.clone();

    match opts.role {
        Role::Client => {
            let client_str = opts.client_addr.clone().unwrap();
            println!(
                "Client is bound to {}, connecting to Server at {}",
                client_str.clone(),
                server_str.clone()
            );
            let (outbound, inbound) = bind_udp_socket(client_str).await;
            run_client(
                outbound,
                inbound,
                ipv4_resolve(server_str.clone()),
                opts.name.clone(),
                opts.graph.clone(),
            )
            .await;
        }
        Role::Server => {
            println!("Listening on {}", server_str.clone());
            let (outbound, inbound) = bind_udp_socket(server_str).await;

            run_server(outbound, inbound, opts.graph.clone()).await;
        }
    }
}
```

## protocol.rs
Our protocol file here defines three message types. Note how we use a single Rust `enum` to represent all varieties of message types; this allows us to handle `Message`s of different types with a single  Rust network channel. We will
use the `demux` operator to separate out these different message types on the receiving end. 

The `ConnectRequest` and `ConnectResponse` messages have no payload; 
the address of the sender and the type of the message will be sufficient information. The `ChatMsg` message type has a `nickname` field, a `message` field, and a `ts` 
field for the timestamp. Once again we use the `chrono` crate to represent timestamps.
```rust,ignore
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    ConnectRequest,
    ConnectResponse,
    ChatMsg {
        nickname: String,
        message: String,
        ts: DateTime<Utc>,
    },
}
```

## server.rs
The chat server is nearly as simple as the echo server. The main differences are (a) we need to handle multiple message types, 
(b) we need to keep track of the list of clients, and (c) we need to broadcast messages to all clients. 

After a short prelude, we have the Hydroflow code near the top of `run_server()`. It begins by defining `outbound_chan` as a `merge`d destination sink for network messages. Then we get to the
more interesting `inbound_chan` definition. 

The `inbound` channel is a source stream that will carry many
types of `Message`s. We use the `[demux](./surface_ops.gen.md#demux)` operator to partition the stream objects into three channels. The `clients` channel 
will carry the addresses of clients that have connected to the server. The `msgs` channel will carry the `ChatMsg` messages that clients send to the server. 
The `errs` channel will carry any other messages that clients send to the server. Note the structure of the `demux` operator: it takes a closure on 
`(Message, SocketAddr)` pairs, and a variadic tuple of output channel names—in this case `clients`, `msgs`, and `errs`. The closure is basically a big
Rust pattern [`match`](https://doc.rust-lang.org/book/ch06-02-match.html), with one arm for each output channel name given in the variadic tuple. Note 
that the different output channels can have different-typed messages!

```rust, ignore
use crate::GraphType;
use hydroflow::util::{UdpSink, UdpStream};

use crate::protocol::Message;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, graph: Option<GraphType>) {
    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // NW channels
        outbound_chan = merge() -> dest_sink_serde(outbound);
        inbound_chan = source_stream_serde(inbound)
            ->  demux(|(msg, addr), tl!(clients, msgs, errs)|
                    match msg {
                        Message::ConnectRequest => clients.give(addr),
                        Message::ChatMsg {..} => msgs.give(msg),
                        _ => errs.give(msg),
                    }
                );
        clients = inbound_chan[clients] -> tee();
        inbound_chan[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));
```

The remainder of the server consists of two independent pipelines. The first pipeline is one line long, 
and is responsible for acknowledging requests from `clients`: it takes the address of the incoming `Message::ConnectRequest` 
and sends a `ConnectResponse` back to that address. The second pipeline is responsible for broadcasting 
all chat messages to all clients. This all-to-all pairing corresponds to the notion of a cartesian product
or `[cross_join](./surface_ops.gen.md#cross_join)` in Hydroflow. The `cross_join` operator takes two input 
channels and produces a single output channel with a tuple for each pair of inputs, in this case it produces
`(Message, SocketAddr)` pairs. Conveniently, that is exactly the structure needed for sending to the `outbound_chan` sink!
We call the cross-join pipeline `broadcast` because it effectively broadcasts all messages to all clients.

Finally, the server closes with the Rust code to optionally print the dataflow graph.

```rust, ignore
       // Pipeline 1: Acknowledge client connections
        clients[0] -> map(|addr| (Message::ConnectResponse, addr)) -> [0]outbound_chan;

        // Pipeline 2: Broadcast messages to all clients
        broadcast = cross_join() -> [1]outbound_chan;
        inbound_chan[msgs] -> [0]broadcast;
        clients[1] -> [1]broadcast;
    };

    if let Some(graph) = graph {
        let serde_graph = df
            .serde_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                println!("{}", serde_graph.to_mermaid());
            }
            GraphType::Dot => {
                println!("{}", serde_graph.to_dot())
            }
            GraphType::Json => {
                unimplemented!();
                // println!("{}", serde_graph.to_json())
            }
        }
    }

    df.run_async().await.unwrap();
}
```

The mermaid graph for the server is below. The three branches of the `demux` are very clear toward the top. Note also the `tee` of the `clients` channel
for both `ClientResponse` and broadcasting, and the `merge` of all outbound messages into `dest_sink_serde`.

```mermaid
flowchart TB
    subgraph "sg_1v1 stratum 0"
        7v1["7v1 <tt>op_7v1: map(| addr | (Message :: ConnectResponse, addr))</tt>"]
        8v1["8v1 <tt>op_8v1: cross_join()</tt>"]
        1v1["1v1 <tt>op_1v1: merge()</tt>"]
        2v1["2v1 <tt>op_2v1: dest_sink_serde(outbound)</tt>"]
    end
    subgraph "sg_2v1 stratum 0"
        3v1["3v1 <tt>op_3v1: source_stream_serde(inbound)</tt>"]
        4v1["4v1 <tt>op_4v1: demux(| (m, a), tl! (clients, msgs, errs) | match m<br>{<br>    Message :: ConnectRequest =&gt; clients.give(a), Message :: ChatMsg { .. } =&gt;<br>    msgs.give(m), _ =&gt; errs.give(m),<br>})</tt>"]
        5v1["5v1 <tt>op_5v1: tee()</tt>"]
        6v1["6v1 <tt>op_6v1: for_each(| m | println! (&quot;Received unexpected message type: {:?}&quot;, m))</tt>"]
    end

    9v1{"handoff"}
    10v1{"handoff"}
    11v1{"handoff"}

    1v1-->2v1
    3v1-->4v1
    4v1-->5v1
    4v1-->6v1
    4v1-->10v1
    5v1-->9v1
    5v1-->11v1
    7v1-->1v1
    8v1-->1v1
    9v1-->7v1
    10v1-->8v1
    11v1-->8v1
```

## client.rs
The chat client is not very different from the echo server client, with two small additions and two 
new design patterns. The additions are 
(1) it has a Rust helper routine `pretty_print_msg` for formatting output, and (2) it sends a
`ConnectRequest` message to the server upon invocation. The new design patterns are (a) a pipeline that runs once 
as a "bootstrap" in the first epoch, and (b) the use of
`cross_join` as a "gated buffer" to postpone sending messages.

The prelude of the file is the same as the echo server client, with the addition of three crates for 
handling dataflow graph display, `chrono` timestamps and `colored` output. This is followed by the 
`pretty_print_msg` function, which is fairly self-explanatory.

```rust,ignore
use crate::protocol::Message;
use chrono::prelude::*;
use hydroflow::hydroflow_syntax;
use hydroflow::util::{UdpSink, UdpStream};
use std::net::SocketAddr;

use crate::GraphType;
use chrono::Utc;
use colored::Colorize;

fn pretty_print_msg(msg: Message) {
    if let Message::ChatMsg {
        nickname,
        message,
        ts,
    } = msg
    {
        println!(
            "{} {}: {}",
            ts.with_timezone(&Local)
                .format("%b %-d, %-I:%M:%S")
                .to_string()
                .truecolor(126, 126, 126)
                .italic(),
            nickname.green().italic(),
            message,
        );
    }
}
```

This brings us to the `run_client` function. As in `run_server` we begin with a standard pattern of a `merge`d `outbound_chan`, 
and a `demux`ed `inbound_chan`. The client handles only two inbound `Message` types: `Message::ConnectResponse` and `Message::ChatMsg`.

 
```rust,ignore
pub(crate) async fn run_client(
    outbound: UdpSink,
    inbound: UdpStream,
    server_addr: SocketAddr,
    name: String,
    graph: Option<GraphType>,
) {
    println!("Client live!");

    let mut hf = hydroflow_syntax! {
        // set up channels
        outbound_chan = merge() -> dest_sink_serde(outbound);
        inbound_chan = source_stream_serde(inbound) -> map(|(m, _)| m)
            ->  demux(|m, tl!(acks, msgs, errs)|
                    match m {
                        Message::ConnectResponse => acks.give(m),
                        Message::ChatMsg {..} => msgs.give(m),
                        _ => errs.give(m),
                    }
                );
        inbound_chan[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));
```
The core logic of the client consists of three dataflow pipelines shown below. 

1. The first pipeline is the "bootstrap" alluded to above.
It starts with `source_iter` operator on a single, opaque "unit" (`()`) value. This value is available when the client begins, which means 
this pipeline runs immediately on startup, and generates a single `ConnectRequest` message which is sent to the server.

2. The second pipeline reads from `source_stdin` and sends messages to the server. It differs from our echo-server example in the use of a `cross_join`
with `inbound_chan[acks]`. In principle, this cross-join is like that of the server: it forms pairs between all messages and all servers that send a `ConnectResponse` ack. 
In principle that means that the client is broadcasting each message to all servers.
In practice, however, the client establishes at most one connection to a server. Hence over time, this pipeline starts with zero `ConnectResponse`s and is sending no messages; 
subsequently it receives a single `ConnectResponse` and starts sending messages. The `cross_join` is thus effectively a buffer for messages, and a "gate" on that buffer that opens 
when the client receives its sole `ConnectResponse`.

3. The final pipeline simple pretty-prints the messages received from the server.

```rust,ignore
        // send a single connection request on startup
        source_iter([()]) -> map(|_m| (Message::ConnectRequest, server_addr)) -> [0]outbound_chan;

        // take stdin and send to server as a msg
        // the join serves to buffer msgs until the connection request is acked
        msg_send = cross_join() -> map(|(msg, _)| (msg, server_addr)) -> [1]outbound_chan;
        lines = source_stdin()
          -> map(|l| Message::ChatMsg {
                    nickname: name.clone(),
                    message: l.unwrap(),
                    ts: Utc::now()})
          -> [0]msg_send;
        inbound_chan[acks] -> [1]msg_send;

        // receive and print messages
        inbound_chan[msgs] -> for_each(pretty_print_msg);
    };

    // optionally print the dataflow graph
    if let Some(graph) = graph {
        let serde_graph = hf
            .serde_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                println!("{}", serde_graph.to_mermaid());
            }
            GraphType::Dot => {
                println!("{}", serde_graph.to_dot())
            }
            GraphType::Json => {
                unimplemented!();
            }
        }
    }

    hf.run_async().await.unwrap();
}
```

The client's mermaid graph looks a bit different than the server's, mostly because it routes some data to
the screen rather than to an outbound network channel.
```mermaid
flowchart TB
    subgraph "sg_1v1 stratum 0"
        7v1["7v1 <tt>op_7v1: source_iter([()])</tt>"]
        8v1["8v1 <tt>op_8v1: map(| _m | (Message :: ConnectRequest, server_addr))</tt>"]
        11v1["11v1 <tt>op_11v1: source_stdin()</tt>"]
        12v1["12v1 <tt>op_12v1: map(| l | Message :: ChatMsg<br>{ nickname : name.clone(), message : l.unwrap(), ts : Utc :: now() })</tt>"]
        9v1["9v1 <tt>op_9v1: cross_join()</tt>"]
        10v1["10v1 <tt>op_10v1: map(| (msg, _) | (msg, server_addr))</tt>"]
        1v1["1v1 <tt>op_1v1: merge()</tt>"]
        2v1["2v1 <tt>op_2v1: dest_sink_serde(outbound)</tt>"]
    end
    subgraph "sg_2v1 stratum 0"
        3v1["3v1 <tt>op_3v1: source_stream_serde(inbound)</tt>"]
        4v1["4v1 <tt>op_4v1: map(| (m, _) | m)</tt>"]
        5v1["5v1 <tt>op_5v1: demux(| m, tl! (acks, msgs, errs) | match m<br>{<br>    Message :: ConnectResponse =&gt; acks.give(m), Message :: ChatMsg { .. } =&gt;<br>    msgs.give(m), _ =&gt; errs.give(m),<br>})</tt>"]
        6v1["6v1 <tt>op_6v1: for_each(| m | println! (&quot;Received unexpected message type: {:?}&quot;, m))</tt>"]
        13v1["13v1 <tt>op_13v1: for_each(pretty_print_msg)</tt>"]
    end

    14v1{"handoff"}

    1v1-->2v1
    3v1-->4v1
    4v1-->5v1
    5v1-->6v1
    5v1-->14v1
    5v1-->13v1
    7v1-->8v1
    8v1-->1v1
    9v1-->10v1
    10v1-->1v1
    11v1-->12v1
    12v1-->9v1
    14v1-->9v1
```

## Running the example
As described in `hydroflow/hydroflow/example/chat/README.md`, we can run the server in one terminal, and run clients in additional terminals.
The client and server need to agree on `server-addr` or this won't work!

Fire up the server in terminal 1:
```console
cargo run -p hydroflow --example chat -- --name "_" --role server --server-addr 127.0.0.1:12347
% ```

Start client "alice" in terminal 2 and type some messages, and you'll see them 
echoed back to you. This will appear in colored fonts in most terminals
(but unfortunately not in this markdown-based book!)
```html
% cargo run -p hydroflow --example chat -- --name "alice" --role client --client-addr 127.0.0.1:9090 --server-addr 127.0.0.1:12347
Client is bound to 127.0.0.1:9090, connecting to Server at 127.0.0.1:12347
Client live!
Hello (hello hello) ... is there anybody in here?
Dec 13, 12:04:34 alice: Hello (hello hello) ... is there anybody in here?
Just nod if you can hear me.
Dec 13, 12:04:58 alice: Just nod if you can hear me.
Is there anyone home?
Dec 13, 12:05:01 alice: Is there anyone home?
```

Now start client "bob" in terminal 3, and notice how he instantly receives the backlog of Alice's messages from the server's `cross_join`. 
(The messages may not be printed in the same order as they were timestamped! The `cross_join` operator is not guaranteed to preserve order, nor
is the udp network. Fixing these issues requires extra client logic that we leave as an exercise to the reader.)
```console
% cargo run -p hydroflow --example chat -- --name "bob" --role client --client-addr 127.0.0.1:9091 --server-addr 127.0.0.1:12347
Client is bound to 127.0.0.1:9091, connecting to Server at 127.0.0.1:12347
Client live!
Dec 13, 12:05:01 alice: Is there anyone home?
Dec 13, 12:04:58 alice: Just nod if you can hear me.
Dec 13, 12:04:34 alice: Hello (hello hello) ... is there anybody in here?
```
Now in terminal 3, Bob can respond:
```console
*nods*
Dec 13, 12:05:05 bob: *nods*
```
and if we go back to terminal 2 we can see that Alice gets the message too:
```console
Dec 13, 12:05:05 bob: *nods*
```