//! Surface syntax tests of asynchrony and networking.

use std::collections::HashSet;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use bytes::Bytes;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{collect_ready, tcp_lines};
use hydroflow::{hydroflow_syntax, rassert, rassert_eq};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::task::LocalSet;
use tokio_util::codec::{BytesCodec, FramedWrite, LinesCodec};

#[tokio::test]
pub async fn test_echo_udp() -> Result<(), Box<dyn Error>> {
    let local = LocalSet::new();

    let server_socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let server_addr = server_socket.local_addr()?;
    let server_addr: SocketAddr = (Ipv4Addr::LOCALHOST, server_addr.port()).into();

    // Server:
    let serv = local.spawn_local(async {
        let socket = server_socket;
        let (udp_send, udp_recv, _) = hydroflow::util::udp_lines(socket);
        println!("Server live!");

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df: Hydroflow = hydroflow_syntax! {
            recv = source_stream(udp_recv)
                -> map(|r| r.unwrap())
                -> tee();
            // Echo
            recv[0] -> dest_sink(udp_send);
            // Testing
            recv[1] -> map(|(s, _addr)| s) -> for_each(|s| seen_send.send(s).unwrap());
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        let seen: HashSet<_> = collect_ready(seen_recv);
        rassert_eq!(4, seen.len())?;
        rassert!(seen.contains("Hello"))?;
        rassert!(seen.contains("World"))?;
        rassert!(seen.contains("Raise"))?;
        rassert!(seen.contains("Count"))?;

        Ok(()) as Result<(), Box<dyn Error>>
    });

    // Client A:
    let client_a = local.spawn_local(async move {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
            .await
            .unwrap();
        let (send_udp, recv_udp, _) = hydroflow::util::udp_lines(socket);

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df = hydroflow_syntax! {
            recv = source_stream(recv_udp)
                -> map(|r| r.unwrap())
                -> tee();
            recv[0] -> for_each(|x| println!("client A recv: {:?}", x));
            recv[1] -> map(|(s, _addr)| s) -> for_each(|s| seen_send.send(s).unwrap());

            // Sending
            source_iter([ "Hello", "World" ]) -> map(|s| (s.to_owned(), server_addr)) -> dest_sink(send_udp);
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        let seen: Vec<_> = collect_ready(seen_recv);
        rassert_eq!(&["Hello".to_owned(), "World".to_owned()], &*seen)?;

        Ok(()) as Result<(), Box<dyn Error>>
    });

    // Client B:
    let client_b = local.spawn_local(async move {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
            .await
            .unwrap();
        let (send_udp, recv_udp, _) = hydroflow::util::udp_lines(socket);

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df = hydroflow_syntax! {
            recv = source_stream(recv_udp)
                -> map(|r| r.unwrap())
                -> tee();
            recv[0] -> for_each(|x| println!("client B recv: {:?}", x));
            recv[1] -> map(|(s, _addr)| s) -> for_each(|s| seen_send.send(s).unwrap());

            // Sending
            source_iter([ "Raise", "Count" ]) -> map(|s| (s.to_owned(), server_addr)) -> dest_sink(send_udp);
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        let seen: Vec<_> = collect_ready(seen_recv);
        rassert_eq!(&["Raise".to_owned(), "Count".to_owned()], &*seen)?;

        Ok(()) as Result<(), Box<dyn Error>>
    });

    local.await;
    serv.await??;
    client_a.await??;
    client_b.await??;

    Ok(())
}

#[tokio::test]
pub async fn test_echo_tcp() -> Result<(), Box<dyn Error>> {
    let local = LocalSet::new();

    // Port 0 -> picks any available port.
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let addr = listener.local_addr()?;

    // Server:
    let serv = local.spawn_local(async move {
        let (server_stream, _) = listener.accept().await?;
        let (server_send, server_recv) = tcp_lines(server_stream);

        println!("Server accepted connection!");

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df: Hydroflow = hydroflow_syntax! {
            rev = source_stream(server_recv)
                -> map(|x| x.unwrap())
                -> tee();
            rev[0] -> dest_sink(server_send);
            rev[1] -> for_each(|s| seen_send.send(s).unwrap());
        };

        tokio::time::timeout(Duration::from_secs(1), df.run_async())
            .await
            .expect_err("Expected time out");

        let seen: Vec<_> = collect_ready(seen_recv);
        rassert_eq!(&["Hello".to_owned(), "World".to_owned()], &*seen)?;

        Ok(()) as Result<(), Box<dyn Error>>
    });

    // Client:
    let client = local.spawn_local(async move {
        let client_stream = TcpStream::connect(addr).await?;
        let (client_send, client_recv) = tcp_lines(client_stream);

        println!("Client connected!");

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df = hydroflow_syntax! {
            recv = source_stream(client_recv)
                -> map(|x| x.unwrap())
                -> tee();

            recv[0] -> for_each(|s| println!("echo {}", s));
            recv[1] -> for_each(|s| seen_send.send(s).unwrap());

            source_iter([
                "Hello",
                "World",
            ]) -> dest_sink(client_send);
        };

        println!("Client running!");

        tokio::time::timeout(Duration::from_secs(1), df.run_async())
            .await
            .expect_err("Expected time out");

        let seen: Vec<_> = collect_ready(seen_recv);
        rassert_eq!(&["Hello".to_owned(), "World".to_owned()], &*seen)?;

        Ok(()) as Result<(), Box<dyn Error>>
    });

    local.await;
    serv.await??;
    client.await??;

    Ok(())
}

#[tokio::test]
pub async fn test_echo() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (lines_send, lines_recv) = hydroflow::util::unbounded_channel::<String>();

    // LinesCodec separates each line from `lines_recv` with `\n`.
    let stdout_lines = FramedWrite::new(tokio::io::stdout(), LinesCodec::new());

    let mut df: Hydroflow = hydroflow_syntax! {
        source_stream(lines_recv) -> dest_sink(stdout_lines);
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    lines_send.send("Hello".to_owned()).unwrap();
    lines_send.send("World".to_owned()).unwrap();
    df.run_available();

    lines_send.send("Hello".to_owned()).unwrap();
    lines_send.send("World".to_owned()).unwrap();
    df.run_available();

    // Allow background thread to catch up.
    tokio::time::sleep(Duration::from_secs(1)).await;
}

#[tokio::test]
pub async fn test_futures_stream_sink() -> Result<(), Box<dyn Error>> {
    const MAX: usize = 20;

    let (mut send, recv) = hydroflow::futures::channel::mpsc::channel::<usize>(5);
    send.try_send(0).unwrap();

    let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

    let mut df = hydroflow_syntax! {
        recv = source_stream(recv) -> tee();
        recv[0] -> map(|x| x + 1)
            -> filter(|&x| x < MAX)
            -> dest_sink(send);
        recv[1] -> for_each(|x| seen_send.send(x).unwrap());
    };

    tokio::select! {
        _ = df.run_async() => (),
        _ = tokio::time::sleep(Duration::from_secs(1)) => (),
    };

    let seen: Vec<_> = collect_ready(seen_recv);
    rassert_eq!(&std::array::from_fn::<_, MAX, _>(|i| i), &*seen)?;

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn asynctest_dest_sink_bounded_channel() {
    // In this example we use a _bounded_ channel for our `Sink`. This is for demonstration only,
    // instead you should use [`hydroflow::util::unbounded_channel`]. A bounded channel results in
    // `Hydroflow` buffering items internally instead of within the channel.
    let (send, recv) = tokio::sync::mpsc::channel::<usize>(5);
    let send = tokio_util::sync::PollSender::new(send);
    let mut recv = tokio_stream::wrappers::ReceiverStream::new(recv);

    let mut flow = hydroflow_syntax! {
        source_iter(0..10) -> dest_sink(send);
    };
    tokio::time::timeout(std::time::Duration::from_secs(1), flow.run_async())
        .await
        .expect_err("Expected time out");

    // Only 5 elemts received due to buffer size
    let out: Vec<_> = collect_ready(&mut recv);
    assert_eq!(&[0, 1, 2, 3, 4], &*out);
}

#[tokio::test(flavor = "current_thread")]
async fn asynctest_dest_sink_duplex() {
    use bytes::Bytes;
    use tokio::io::AsyncReadExt;
    use tokio_util::codec;

    // Like a channel, but for a stream of bytes instead of discrete objects.
    let (asyncwrite, mut asyncread) = tokio::io::duplex(256);
    // Now instead handle discrete byte lists by length-encoding them.
    let mut sink = codec::LengthDelimitedCodec::builder()
        // Use 1 byte len field (max 255) so we don't have to worry about endianness.
        .length_field_length(1)
        .new_write(asyncwrite);

    let mut flow = hydroflow_syntax! {
        source_iter([
            Bytes::from_static(b"hello"),
            Bytes::from_static(b"world"),
        ]) -> dest_sink(&mut sink);
    };
    tokio::time::timeout(std::time::Duration::from_secs(1), flow.run_async())
        .await
        .expect_err("Expected time out");

    let mut buf = Vec::<u8>::new();
    asyncread.read_buf(&mut buf).await.unwrap();
    // `\x05` is length prefix of "5".
    assert_eq!(b"\x05hello\x05world", &*buf);
}

#[tokio::test(flavor = "current_thread")]
async fn asynctest_dest_asyncwrite_duplex() {
    use tokio::io::AsyncReadExt;

    // Like a channel, but for a stream of bytes instead of discrete objects.
    // This could be an output file, network port, stdout, etc.
    let (asyncwrite, mut asyncread) = tokio::io::duplex(256);
    let sink = FramedWrite::new(asyncwrite, BytesCodec::new());

    let mut flow = hydroflow_syntax! {
        source_iter([
            Bytes::from_static("hello".as_bytes()),
            Bytes::from_static("world".as_bytes()),
        ]) -> dest_sink(sink);
    };
    tokio::time::timeout(std::time::Duration::from_secs(1), flow.run_async())
        .await
        .expect_err("Expected time out");

    let mut buf = Vec::<u8>::new();
    asyncread.read_buf(&mut buf).await.unwrap();
    // `\x05` is length prefix of "5".
    assert_eq!(b"helloworld", &*buf);
}

#[tokio::test(flavor = "current_thread")]
async fn asynctest_source_stream() {
    LocalSet::new()
        .run_until(async {
            let (a_send, a_recv) = hydroflow::util::unbounded_channel::<usize>();
            let (b_send, b_recv) = hydroflow::util::unbounded_channel::<usize>();

            tokio::task::spawn_local(async move {
                let mut flow = hydroflow_syntax! {
                    source_stream(a_recv) -> for_each(|x| { b_send.send(x).unwrap(); });
                };
                flow.run_async().await.unwrap();
            });
            tokio::task::spawn_local(async move {
                let mut flow = hydroflow_syntax! {
                    source_stream(b_recv) -> for_each(|x| println!("{}", x));
                };
                flow.run_async().await.unwrap();
            });

            a_send.send(1).unwrap();
            a_send.send(2).unwrap();
            a_send.send(3).unwrap();

            tokio::task::yield_now().await;
        })
        .await;
}
