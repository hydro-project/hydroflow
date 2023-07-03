## Paoxs
This is a Paxos implementation with stable leaders.

### To Run the code:
Look in the file `members.json` to find the addresses of the proposers and acceptors. 
For each proposer, launch a process on the node with a matching IP address as follows.
Here we assume the proposer's IP address is `localhost` and ports `12300`, `12301`, and `12302` are free:
```
cargo run --example paxos -- --path hydroflow/examples/paxos/members.json --role proposer --addr localhost:12300
cargo run --example paxos -- --path hydroflow/examples/paxos/members.json --role proposer --addr localhost:12310
```

Now for each acceptor, launch a process on the node with the matching IP address as follows.
Here we assume the acceptor's IP address is `127.0.0.1` and ports `12321` and `12322` are free:
```
cargo run --example paxos -- --path hydroflow/examples/paxos/members.json --role acceptor --addr localhost:12320
cargo run --example paxos -- --path hydroflow/examples/paxos/members.json --role acceptor --addr localhost:12330
cargo run --example paxos -- --path hydroflow/examples/paxos/members.json --role acceptor --addr localhost:12340
```

Now, in the proposer process you can type a string payload at `stdin`.

Adding the `--graph <graph_type>` flag to the end of the command lines above will print out a node-and-edge diagram of the program. Supported values for `<graph_type>` include [mermaid](https://mermaid-js.github.io/) and [dot](https://graphviz.org/doc/info/lang.html).