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

NUM_ACCEPTORS=10
NUM_ACCEPTORS_USE=$NUM_ACCEPTORS
NUM_PROXY=5

ACCEPTOR_MIN_PORT=1400
ACCEPTOR_MAX_PORT=$(($ACCEPTOR_MIN_PORT+$NUM_ACCEPTORS-1))

PROXY_MIN_PORT=1200
PROXY_MAX_PORT=$(($PROXY_MIN_PORT+$NUM_PROXY-1))

# get unix time
UNIX_TIME=$(date +%s)
# OUTPUT_DIR = "run_data/$1_$2_$UNIX_TIME"

for PORT in $(seq $ACCEPTOR_MIN_PORT $ACCEPTOR_MAX_PORT);
do
    ./target/release/examples/proposer_eval --role acceptor --port $PORT --addr localhost --id 14 &
    bgpids+=($!) #--output_dir $OUTPUT_DIR
done

sleep 2

if [ "$1" = "control" ]; then
    ./target/release/examples/proposer_eval --role proposer --port 20000 --addr localhost --id 10 --acceptors $NUM_ACCEPTORS_USE --proxies $NUM_PROXY
elif [ "$1" = "proxy" ]; then

    for PORT in $(seq $PROXY_MIN_PORT $PROXY_MAX_PORT);
    do
        ./target/release/examples/proposer_eval --role proxy-leader  --port $PORT --addr localhost --id 14 --acceptors $NUM_ACCEPTORS_USE --proxies $NUM_PROXY &
        bgpids+=($!)
    done

    sleep 2

    ./target/release/examples/proposer_eval --role proposer --port 20000 --addr localhost --id 10 --use-proxy --acceptors $NUM_ACCEPTORS_USE --proxies $NUM_PROXY
    #pid=$!
    #dtrace -x ustackframes=100 -n "profile-97 /pid == $pid/ { @[ustack()] = count(); } tick-60s { exit(0); }"  -o out.user_stacks
fi
