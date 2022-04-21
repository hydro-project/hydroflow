#!/bin/bash

set -e

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

NUM_ACCEPTORS=50

ACCEPTOR_MIN_PORT=1400
ACCEPTOR_MAX_PORT=$(($ACCEPTOR_MIN_PORT+$NUM_ACCEPTORS-1))
for PORT in $(seq $ACCEPTOR_MIN_PORT $ACCEPTOR_MAX_PORT);
do
    ./target/release/examples/proposer_eval --role acceptor --port $PORT --addr localhost --id 14 &
    bgpids+=($!)
done

sleep 2

NUM_ACCEPTORS2=50

if [ "$1" = "control" ]; then
    ./target/release/examples/proposer_eval --role proposer --port 20000 --addr localhost --id 10 --acceptors $NUM_ACCEPTORS2
elif [ "$1" = "proxy" ]; then
    ./target/release/examples/proposer_eval --role proxy-leader  --port 1200 --addr localhost --id 14 --acceptors $NUM_ACCEPTORS2 &
    bgpids+=($!)
    ./target/release/examples/proposer_eval --role proxy-leader  --port 1201  --addr localhost --id 15 --acceptors $NUM_ACCEPTORS2 &
    bgpids+=($!) 
    ./target/release/examples/proposer_eval --role proxy-leader --port 1202 --addr localhost --id 16 --acceptors $NUM_ACCEPTORS2 &
    bgpids+=($!)

    sleep 2

    ./target/release/examples/proposer_eval --role proposer --port 20000 --addr localhost --id 10 --use-proxy --acceptors $NUM_ACCEPTORS2
    #pid=$!
    #dtrace -x ustackframes=100 -n "profile-97 /pid == $pid/ { @[ustack()] = count(); } tick-60s { exit(0); }"  -o out.user_stacks
fi
