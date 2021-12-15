use std::sync::mpsc::channel;

use hydroflow::{
    lang::collections::Iter,
    scheduled::{
        ctx::RecvCtx, graph::Hydroflow, graph_demux::GraphDemux, graph_ext::GraphExt,
        handoff::VecHandoff, net::Message,
    },
};
use tokio::net::{TcpListener, TcpStream};

#[test]
fn test_networked() {
    let (port_sender, port_receiver) = channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let mut df = Hydroflow::new();

            let stream = TcpListener::bind("localhost:0").await.unwrap();
            let addr = stream.local_addr().unwrap();

            port_sender.send(addr.port()).unwrap();

            let (stream, _) = stream.accept().await.unwrap();
            let network_send = df.add_write_tcp_stream(stream);

            let (input, out) = df.add_input();

            input.give(Iter(
                vec![Message::Data {
                    address: 0,
                    batch: vec![1, 2, 3, 4].into(),
                }]
                .into_iter(),
            ));

            df.add_edge(out, network_send);

            df.run_async().await.unwrap();
        });
    });

    let (send, recv) = channel();

    std::thread::spawn(move || {
        let port = port_receiver.recv().unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let mut df = Hydroflow::new();

            let stream = TcpStream::connect(format!("localhost:{}", port))
                .await
                .unwrap();
            let network_recv = df.add_read_tcp_stream(stream);

            let input = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<_>>| {
                for v in recv.take_inner() {
                    send.send(v).unwrap();
                }
            });

            let (demux, input_port) = df.add_demux::<_, _, _, VecHandoff<_>>(|_| ());

            df.add_edge(network_recv, input_port);
            df.add_demux_edge(&demux, (), input);

            df.run_async().await.unwrap();
        });
    });

    let val = recv.recv().unwrap();
    assert_eq!(
        val,
        Message::Data {
            address: 0,
            batch: vec![1, 2, 3, 4].into(),
        }
    );
}
