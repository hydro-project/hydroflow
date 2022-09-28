//! Surface syntax tests of asynchrony and networking.

use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr};
use std::rc::Rc;
use std::time::Duration;

use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::{hydroflow_syntax, rassert, rassert_eq};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::task::LocalSet;
use tokio_stream::wrappers::LinesStream;

#[tokio::test]
pub async fn test_echo_udp() -> Result<(), Box<dyn Error>> {
    const SERVER_PORT: u16 = 8099;
    let server_socket = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), SERVER_PORT);

    let local = LocalSet::new();

    // Server:
    let serv = local.spawn_local(async {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, SERVER_PORT))
            .await
            .unwrap();
        let (send, recv) = hydroflow::util::udp_lines(socket);
        println!("Server live!");

        let seen = <Rc<RefCell<HashSet<String>>>>::default();
        let seen_inner = Rc::clone(&seen);

        let mut df: Hydroflow = hydroflow_syntax! {
            recv_stream(recv)
                -> map(|r| r.unwrap())
                -> map(|x| { seen_inner.borrow_mut().insert(x.0.clone()); x })
                -> sink_async(send);
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        rassert_eq!(4, seen.borrow().len())?;
        rassert!(seen.borrow().contains("Hello"))?;
        rassert!(seen.borrow().contains("World"))?;
        rassert!(seen.borrow().contains("Raise"))?;
        rassert!(seen.borrow().contains("Count"))?;

        Ok(()) as Result<(), Box<dyn Error>>
    });

    // Client A:
    let client_a = local.spawn_local(async move {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
            .await
            .unwrap();
        let (send, recv) = hydroflow::util::udp_lines(socket);

        let seen = <Rc<RefCell<Vec<String>>>>::default();
        let seen_inner = Rc::clone(&seen);

        let mut df = hydroflow_syntax! {
            recv_stream(recv)
                -> map(|x| x.unwrap())
                -> map(|x| { seen_inner.borrow_mut().push(x.0.clone()); x })
                -> for_each(|x| println!("client A recv: {:?}", x));
            recv_iter([ "Hello", "World" ]) -> map(|s| (s.to_owned(), server_socket)) -> sink_async(send);
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        rassert_eq!(&["Hello".to_owned(), "World".to_owned()], &**seen.borrow())?;

        Ok(()) as Result<(), Box<dyn Error>>
    });

    // Client B:
    let client_b = local.spawn_local(async move {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
            .await
            .unwrap();
        let (send, recv) = hydroflow::util::udp_lines(socket);

        let seen = <Rc<RefCell<Vec<String>>>>::default();
        let seen_inner = Rc::clone(&seen);

        let mut df = hydroflow_syntax! {
            recv_stream(recv)
                -> map(|x| x.unwrap())
                -> map(|x| { seen_inner.borrow_mut().push(x.0.clone()); x })
                -> for_each(|x| println!("client B recv: {:?}", x));
            recv_iter([ "Raise", "Count" ]) -> map(|s| (s.to_owned(), server_socket)) -> sink_async(send);
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        rassert_eq!(&["Raise".to_owned(), "Count".to_owned()], &**seen.borrow())?;

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

    // Server:
    let serv = local.spawn_local(async {
        let listener = TcpListener::bind("localhost:8090").await?;

        let (server_stream, _) = listener.accept().await?;
        let (server_recv, server_send) = server_stream.into_split();
        let lines_recv = LinesStream::new(BufReader::new(server_recv).lines());

        println!("Server accepted connection!");

        let mut df: Hydroflow = hydroflow_syntax! {
            recv_stream(lines_recv)
                -> map(|x| x.unwrap())
                -> map(|s| { println!("serv {}", s); s })
                -> map(|s| format!("{}\n", s))
                -> write_async(server_send);
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        Ok(()) as Result<(), Box<dyn Error>>
    });

    // Client:
    let client = local.spawn_local(async {
        let client_stream = TcpStream::connect("localhost:8090").await?;

        println!("Client connected!");

        let (client_recv, client_send) = client_stream.into_split();
        let lines_recv = LinesStream::new(BufReader::new(client_recv).lines());

        let mut df = hydroflow_syntax! {
            recv_stream(lines_recv)
                -> map(|x| x.unwrap())
                -> for_each(|s| println!("echo {}", s));
            recv_iter([ "Hello\n", "World\n" ]) -> write_async(client_send);
        };

        println!("Client running!");

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

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

    let seen = <Rc<RefCell<Vec<usize>>>>::default();
    let seen_inner = Rc::clone(&seen);

    let mut df = hydroflow_syntax! {
        recv_stream(recv)
            -> map(|x| { seen_inner.borrow_mut().push(x); x })
            -> map(|x| x + 1)
            -> filter(|&x| x < MAX)
            -> sink_async(send);
    };

    tokio::select! {
        _ = df.run_async() => (),
        _ = tokio::time::sleep(Duration::from_secs(1)) => (),
    };

    assert_eq!(&std::array::from_fn::<_, MAX, _>(|i| i), &**seen.borrow());

    Ok(())
}
