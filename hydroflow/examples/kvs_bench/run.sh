#!/bin/bash
(trap 'kill 0' SIGINT; cargo run --example kvs_bench -- server --batch-addr 127.0.0.100:5000 --client-addr 127.0.0.100:5001 --peer 127.0.0.101:5000) &
(trap 'kill 0' SIGINT; cargo run --example kvs_bench -- server --batch-addr 127.0.0.101:5000 --client-addr 127.0.0.101:5001 --peer 127.0.0.100:5000) &
wait
