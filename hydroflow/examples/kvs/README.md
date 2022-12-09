Simple single-node key-value store example based on a join of PUTs and GETs. 
Current semantics are purely monotone:
 - PUTs are appended: we remember them all forever
 - GETs are also remembered forever, akin to SUBSCRIBE: once a client issues a GET for key k they will receive a response on the current values of key k (if non-empty) and every future PUT for key k.
 - GETs for empty keys get no acknowledgement, but will receive responses when a subsequent PUT arrives for that key

 Clients accept commands on stdin. Command syntax is as follows:
 - `PUT <key>, <value>`
 - `GET <key>'
 Commands are case-insensitive. All keys and values are treated as `String`s.


To run the example, open 2 terminals.

In one terminal run the server like so:
```
    cargo run -p hydroflow --example kvs -- --role "server" --addr localhost:12346
```

In another terminal run a client:
```
cargo run -p hydroflow --example kvs -- --role client --addr localhost:9090 --server-addr localhost:12346
```

Adding the `--graph <graph_type>` flag to the end of the command lines above will print out a node-and-edge diagram of the program. Supported values for `<graph_type>` include [mermaid](https://mermaid-js.github.io/) and [dot](https://graphviz.org/doc/info/lang.html).
