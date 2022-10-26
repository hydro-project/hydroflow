// TODO(mingwei): Need rust-analyzer support
#![allow(clippy::uninlined_format_args)]

//! Surface syntax tests of asynchrony and networking.

use std::collections::HashSet;
use std::error::Error;
use std::net::Ipv4Addr;
use std::time::Duration;

use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::{hydroflow_syntax, rassert, rassert_eq};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::task::LocalSet;
use tokio_stream::wrappers::LinesStream;

#[tokio::test]
pub async fn test_echo_udp() -> Result<(), Box<dyn Error>> {
    let local = LocalSet::new();

    let server_socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let server_addr = server_socket.local_addr()?;

    // Server:
    let serv = local.spawn_local(async {
        let socket = server_socket;
        let (udp_send, udp_recv) = hydroflow::util::udp_lines(socket);
        println!("Server live!");

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df: Hydroflow = hydroflow_syntax! {
            recv = recv_stream(udp_recv)
                -> map(|r| r.unwrap())
                -> tee();
            // Echo
            recv[0] -> sink_async(udp_send);
            // Testing
            recv[1] -> map(|(s, _addr)| s) -> for_each(|s| seen_send.send(s).unwrap());
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        let seen: HashSet<_> = hydroflow::util::collect_ready(seen_recv).await;
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
        let (send_udp, recv_udp) = hydroflow::util::udp_lines(socket);

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df = hydroflow_syntax! {
            recv = recv_stream(recv_udp)
                -> map(|r| r.unwrap())
                -> tee();
            recv[0] -> for_each(|x| println!("client A recv: {:?}", x));
            recv[1] -> map(|(s, _addr)| s) -> for_each(|s| seen_send.send(s).unwrap());

            // Sending
            recv_iter([ "Hello", "World" ]) -> map(|s| (s.to_owned(), server_addr)) -> sink_async(send_udp);
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        let seen: Vec<_> = hydroflow::util::collect_ready(seen_recv).await;
        rassert_eq!(&["Hello".to_owned(), "World".to_owned()], &*seen)?;

        Ok(()) as Result<(), Box<dyn Error>>
    });

    // Client B:
    let client_b = local.spawn_local(async move {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
            .await
            .unwrap();
        let (send_udp, recv_udp) = hydroflow::util::udp_lines(socket);

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df = hydroflow_syntax! {
            recv = recv_stream(recv_udp)
                -> map(|r| r.unwrap())
                -> tee();
            recv[0] -> for_each(|x| println!("client B recv: {:?}", x));
            recv[1] -> map(|(s, _addr)| s) -> for_each(|s| seen_send.send(s).unwrap());

            // Sending
            recv_iter([ "Raise", "Count" ]) -> map(|s| (s.to_owned(), server_addr)) -> sink_async(send_udp);
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        let seen: Vec<_> = hydroflow::util::collect_ready(seen_recv).await;
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
        let (server_recv, server_send) = server_stream.into_split();
        let lines_recv = LinesStream::new(BufReader::new(server_recv).lines());

        println!("Server accepted connection!");

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df: Hydroflow = hydroflow_syntax! {
            rev = recv_stream(lines_recv)
                -> map(|x| x.unwrap())
                -> tee();
            rev[0] -> map(|s| format!("{}\n", s)) -> write_async(server_send);
            rev[1] -> for_each(|s| seen_send.send(s).unwrap());
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        let seen: Vec<_> = hydroflow::util::collect_ready(seen_recv).await;
        rassert_eq!(&["Hello".to_owned(), "World".to_owned()], &*seen)?;

        Ok(()) as Result<(), Box<dyn Error>>
    });

    // Client:
    let client = local.spawn_local(async move {
        let client_stream = TcpStream::connect(addr).await?;

        println!("Client connected!");

        let (client_recv, client_send) = client_stream.into_split();
        let lines_recv = LinesStream::new(BufReader::new(client_recv).lines());

        let (seen_send, seen_recv) = hydroflow::util::unbounded_channel();

        let mut df = hydroflow_syntax! {
            recv = recv_stream(lines_recv)
                -> map(|x| x.unwrap())
                -> tee();

            recv[0] -> for_each(|s| println!("echo {}", s));
            recv[1] -> for_each(|s| seen_send.send(s).unwrap());

            recv_iter([ "Hello\n", "World\n" ]) -> write_async(client_send);
        };

        println!("Client running!");

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        let seen: Vec<_> = hydroflow::util::collect_ready(seen_recv).await;
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

    //use tokio::io::{AsyncBufReadExt, BufReader};
    // use tokio_stream::wrappers::LinesStream;
    // let stdin_lines = LinesStream::new(BufReader::new(tokio::io::stdin()).lines());
    let stdout_lines = tokio::io::stdout();

    let mut df: Hydroflow = hydroflow_syntax! {
        recv_stream(lines_recv) -> map(|line| line + "\n") -> write_async(stdout_lines);
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
        recv = recv_stream(recv) -> tee();
        recv[0] -> map(|x| x + 1)
            -> filter(|&x| x < MAX)
            -> sink_async(send);
        recv[1] -> for_each(|x| seen_send.send(x).unwrap());
    };

    tokio::select! {
        _ = df.run_async() => (),
        _ = tokio::time::sleep(Duration::from_secs(1)) => (),
    };

    let seen: Vec<_> = hydroflow::util::collect_ready(seen_recv).await;
    rassert_eq!(&std::array::from_fn::<_, MAX, _>(|i| i), &*seen)?;

    Ok(())
}
