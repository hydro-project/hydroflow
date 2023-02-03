#!/bin/bash
(trap 'kill 0' SIGINT; cargo run --example kvs_bench -- server --addr 127.0.0.100:5000 --peer 127.0.0.101:5000) &
(trap 'kill 0' SIGINT; cargo run --example kvs_bench -- server --addr 127.0.0.101:5000 --peer 127.0.0.100:5000) &
wait
