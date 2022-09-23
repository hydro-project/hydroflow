use std::error::Error;
use std::time::Duration;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::LocalSet;
use tokio_stream::wrappers::LinesStream;

#[tokio::test]
pub async fn test_echo_tcp() -> Result<(), Box<dyn Error>> {
    let local = LocalSet::new();

    // Server:
    local.spawn_local(async {
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
                -> send_async(server_send);
        };

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        Ok(()) as Result<(), Box<dyn Error>>
    });

    // Client:
    local.spawn_local(async {
        let client_stream = TcpStream::connect("localhost:8090").await?;

        println!("Client connected!");

        let (client_recv, client_send) = client_stream.into_split();
        let lines_recv = LinesStream::new(BufReader::new(client_recv).lines());

        let mut df = hydroflow_syntax! {
            recv_stream(lines_recv)
                -> map(|x| x.unwrap())
                -> for_each(|s| println!("echo {}", s));
            recv_iter([ "Hello\n", "World\n" ]) -> send_async(client_send);
        };

        println!("Client running!");

        tokio::select! {
            _ = df.run_async() => (),
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        };

        Ok(()) as Result<(), Box<dyn Error>>
    });

    local.await;

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
        recv_stream(lines_recv) -> map(|line| line + "\n") -> send_async(stdout_lines);
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
