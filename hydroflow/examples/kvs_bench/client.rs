use crate::{KVSRequest, KVSResponse};
use futures::SinkExt;
use hydroflow::util::{deserialize_from_bytes, serialize_to_bytes, tcp_bytes};
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, RngCore, SeedableRng};
use std::{
    collections::HashMap,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;

pub async fn run_client(server_addr: SocketAddr) {
    println!("tid client: {}", palaver::thread::gettid());

    let stream = TcpStream::connect(server_addr).await.unwrap();
    stream.set_nodelay(true).unwrap();

    println!("connected");

    let (mut outbound, mut inbound) = tcp_bytes(stream);

    let mut rng = StdRng::from_entropy();

    let mut keys = Vec::new();
    let mut map = HashMap::new();

    for _ in 0..100000 {
        let random_key = rng.next_u64();
        let random_val = rng.next_u64();

        keys.push(random_key.clone());
        map.insert(random_key.clone(), random_val.clone());

        outbound
            .send(serialize_to_bytes(KVSRequest::Put {
                key: random_key,
                value: random_val,
            }))
            .await
            .unwrap();
    }

    println!("sent puts");

    let mut outstanding = 0;

    let mut time_since_last_report = std::time::Instant::now();
    let mut gets = 0;

    loop {
        while outstanding < 5000 {
            let dist = Uniform::new(0, keys.len());
            let key = keys[dist.sample(&mut rng)].clone();

            outbound
                .send(serialize_to_bytes(KVSRequest::Get { key: key }))
                .await
                .unwrap();

            outstanding += 1;
        }

        if let Some(Ok(response)) = inbound.next().await {
            let response: KVSResponse = deserialize_from_bytes(response);
            match response {
                KVSResponse::Response { key, value } => assert_eq!(value, map[&key]),
            }
            outstanding -= 1;
            gets += 1;
        }

        if time_since_last_report.elapsed() >= Duration::from_secs(1) {
            time_since_last_report = Instant::now();
            println!("get/s: {gets}");
            gets = 0;
        }
    }
}
