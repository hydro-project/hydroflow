Simple single-node key-value store example based on a join of PUTs and GETs. 
Current semantics are:
 - PUT overwrites old values
 - GETs are remembered forever, akin to SUBSCRIBE: once a client issues a GET for key k they will receive a response on the current value of key k (if non-empty) and every future PUT for key k.
 - GETs for empty keys currently fail silently.

 Clients accept commands on stdin. Command syntax is as follows:
 - `PUT <key>, <value>`
 - `GET <key>'
 Commands are case-insensitive. All keys and values are treated as `String`s.


To run the example, open 2 terminals.

In one terminal run the server like so:
```
    cargo run -p hydroflow --example kvs -- --role "server" --port 12346 --addr 127.0.0.1
```

In another terminal run a client:
```
cargo run -p hydroflow --example kvs -- --role client --port 9090 --addr 127.0.0.1 --server-port 12346 --server-addr 127.0.0.1
```

Adding the `--graph <graph_type>` flag to the end of the command lines above will print out a node-and-edge diagram of the program. Supported values for `<graph_type>` include [mermaid](https://mermaid-js.github.io/) and [dot](https://graphviz.org/doc/info/lang.html).
