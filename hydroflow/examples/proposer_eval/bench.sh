#!/bin/bash

declare -a bgpids

# when this script ends or is terminated, cleanup will be called
cleanup() {
    for pid in ${bgpids[@]}; do
        kill -9 $pid
    done
}
trap cleanup EXIT SIGINT SIGTERM

echo "This should be running in the root of the repo"
cargo build --release --example proposer_eval

./target/release/examples/proposer_eval  --role acceptor --port 1400 --addr localhost --id 14 &
bgpids+=($!)
./target/release/examples/proposer_eval  --role acceptor --port 1401 --addr localhost --id 15 &
bgpids+=($!)
./target/release/examples/proposer_eval  --role acceptor --port 1402 --addr localhost --id 16 &
bgpids+=($!)

sleep 2

./target/release/examples/proposer_eval  --role proposer --port 12222 --addr localhost --id 10
