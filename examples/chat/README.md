Simple chat example, with a single central server broadcasting to clients.

To run the example, open 3 terminals.

In one terminal run the server like so:
```
cargo run -- --name "_" --role "server" --port 12347 --addr 127.0.0.1
```

In another terminal run the first client:
```
cargo run -- --name "alice" --role client --port 12347 --addr 127.0.0.1
```

In the third terminal run the second client:
```
cargo run -- --name "bob" --role client --port 12347 --addr 127.0.0.1
```

If you type in the client terminals the messages should appear everywhere.
