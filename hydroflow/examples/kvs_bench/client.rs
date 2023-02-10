use crate::{KVSRequest, KVSResponse};
use futures::SinkExt;
use hydroflow::util::{
    deserialize_from_bytes, deserialize_from_bytes2, serialize_to_bytes, tcp_bytes,
};
use rand::{prelude::Distribution, rngs::StdRng, RngCore, SeedableRng};
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;

pub async fn run_client(targets: Vec<SocketAddr>) {
    println!(
        "client:{}. {:?}",
        palaver::thread::gettid(),
        tokio::runtime::Handle::current()
    );

    let puts = Arc::new(AtomicUsize::new(0));

    println!("{targets:?}");
    for target in targets {
        let puts = puts.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread() // Single threaded seems to hang for whatever reason? This will all get replaced soon anyway.
                .enable_all()
                .build()
                .unwrap();

            let _guard = rt.enter();

            rt.block_on(async {
                println!(
                    "client:{}. {:?}",
                    palaver::thread::gettid(),
                    tokio::runtime::Handle::current()
                );

                println!("tid client: {}", palaver::thread::gettid());

                let ctx = tmq::Context::new();

                println!("target: {target:?}");

                let mut dealer_socket = tmq::dealer(&ctx)
                    .connect(&format!("tcp://{}", target))
                    .unwrap();

                // let stream = TcpStream::connect(target).await.unwrap();
                // stream.set_nodelay(true).unwrap();

                println!("connected");

                // let (mut outbound, mut inbound) = tcp_bytes(stream);

                let mut rng = StdRng::from_entropy();

                let dist = rand_distr::Zipf::new(8000, 8.0).unwrap();

                let mut outstanding = 0;

                loop {
                    // println!("client:{}. iter", palaver::thread::gettid());
                    while outstanding < 1 {
                        let key = dist.sample(&mut rng) as u64;
                        let value = rng.next_u64();

                        dealer_socket
                            .send(vec![
                                serialize_to_bytes(KVSRequest::Put { key, value }).to_vec()
                            ])
                            .await
                            .unwrap();

                        // outbound
                        //     .feed(serialize_to_bytes(KVSRequest::Put { key, value }))
                        //     .await
                        //     .unwrap();

                        outstanding += 1;
                    }

                    // dealer_socket.flush().await.unwrap();

                    // outbound.flush().await.unwrap();

                    // println!("client:{}. wait", palaver::thread::gettid());

                    // if let Some(Ok(_response)) = inbound.next().await {
                    //     let response: KVSResponse = deserialize_from_bytes(_response);
                    //     match response {
                    //         KVSResponse::GetResponse { key, reg } => println!("{reg:?}"),
                    //         KVSResponse::PutResponse { key } => (),
                    //     }
                    //     outstanding -= 1;
                    //     puts.fetch_add(1, Ordering::SeqCst);
                    // }

                    if let Some(Ok(_response)) = dealer_socket.next().await {
                        let response: KVSResponse = deserialize_from_bytes2(&_response.0[0]);
                        match response {
                            KVSResponse::GetResponse { key, reg } => println!("{reg:?}"),
                            KVSResponse::PutResponse { key } => (),
                        }
                        outstanding -= 1;
                        puts.fetch_add(1, Ordering::SeqCst);
                    }

                    // tokio::time::sleep(Duration::from_millis(1)).await;
                }
            })
        });
    }

    let mut time_since_last_report = std::time::Instant::now();
    loop {
        if time_since_last_report.elapsed() >= Duration::from_secs(1) {
            time_since_last_report = Instant::now();
            println!("puts/s: {}", puts.load(Ordering::SeqCst));
            puts.store(0, Ordering::SeqCst);
        }

        tokio::time::sleep(Duration::from_millis(32)).await;
    }
}

// let mut futs = Vec::new();
// let mut counter = 0;
// loop {
//     while futs.len() < max_outstanding {
//         if let Some(entry) = iter.next() {
//             futs.push(func(cloner(), entry).boxed());
//         } else {
//             break;
//         }
//     }

//     if futs.len() == 0 {
//         break;
//     }

//     let (item, _, remaining_futures) = futures_util::future::select_all(futs).await;

//     futs = remaining_futures;

//     counter += 1;

//     on_item_completed(counter, item);
// }
