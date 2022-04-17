declare -a bgpids


# when this script ends or is terminated, cleanup will be called
cleanup() {
    for pid in ${bgpids[@]}; do
        kill -9 $pid
    done
}
trap cleanup EXIT SIGINT SIGTERM

cargo run --example proposer_eval --  --role acceptor --port 1400 --addr localhost --id 14 &
bgpids+=($!)
cargo run --example proposer_eval --  --role acceptor --port 1401 --addr localhost --id 15 &
bgpids+=($!)
cargo run --example proposer_eval --  --role acceptor --port 1402 --addr localhost --id 16 &
bgpids+=($!)

sleep 2

cargo run --example proposer_eval --  --role proposer --port 12222 --addr localhost --id 10