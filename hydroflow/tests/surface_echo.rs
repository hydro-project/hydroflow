use std::time::Duration;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;

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
