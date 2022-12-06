# Graph UnReachability

To run:
```
cargo run -p hydroflow --example graph_unreachability
```

Adding the `-- --graph <graph_type>` flag to the end of the command line above will print out a node-and-edge diagram of the program. Supported values for `<graph_type>` include [`mermaid`](https://mermaid-js.github.io/), [`dot`](https://graphviz.org/doc/info/lang.html) and `json`.