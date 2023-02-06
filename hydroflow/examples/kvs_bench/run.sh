#!/bin/bash


topology=127.0.0.100:5000
topology+=,127.0.0.101:5000
topology+=,127.0.0.102:5000
topology+=,127.0.0.103:5000

for i in ${topology//,/ }
do
    (trap 'kill 0' SIGINT; cargo run --release --example kvs_bench -- server --addr "$i" --peers "$topology") &
done

wait
