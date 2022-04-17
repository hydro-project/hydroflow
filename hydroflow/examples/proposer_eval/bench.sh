
cargo run --example proposer_eval --  --role acceptor --port 1400 --addr localhost --id 14 &
cargo run --example proposer_eval --  --role acceptor --port 1401 --addr localhost --id 15 &
cargo run --example proposer_eval --  --role acceptor --port 1402 --addr localhost --id 16 &

sleep 2

cargo run --example proposer_eval --  --role proposer --port 12222 --addr localhost --id 10
