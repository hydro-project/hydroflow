Simple chat example, with a single central server broadcasting to clients.

To run the example, open 3 terminals.

In one terminal run the server like so:
```
cargo run -p hydroflow --example chat -- --name "_" --role "server" --port 12347 --addr 127.0.0.1
```

In another terminal run the first client:
```
cargo run -p hydroflow --example chat -- --name "alice" --role client --port 12347 --addr 127.0.0.1
```

In the third terminal run the second client:
```
cargo run -p hydroflow --example chat -- --name "bob" --role client --port 12347 --addr 127.0.0.1
```

If you type in the client terminals the messages should appear everywhere.

Adding the `--graph <graph_type>` flag to the end of the command lines above will print out a node-and-edge diagram of the program. Supported values for `<graph_type>` include [mermaid](https://mermaid-js.github.io/) and [dot](https://graphviz.org/doc/info/lang.html).