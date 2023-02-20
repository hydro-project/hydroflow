use std::collections::HashMap;

use hydroflow::{
    hydroflow_syntax,
    util::{connection::ConnectionPipe, unix_lines},
};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    let connection_pipes =
        serde_json::from_str::<HashMap<String, ConnectionPipe>>(trimmed).unwrap();

    if let Some(ConnectionPipe::UnixSocket(s)) = connection_pipes.get("foo") {
        println!("ready");
        let mut start_buf = String::new();
        std::io::stdin().read_line(&mut start_buf).unwrap();
        if start_buf != "start\n" {
            panic!("expected start");
        }

        let client_stream = UnixStream::connect(s).await.unwrap();
        let (client_send, _) = unix_lines(client_stream);

        let mut df = hydroflow_syntax! {
            foo = merge() -> dest_sink(client_send);

            repeat_iter([
                "Hello",
                "World",
            ]) -> foo;
        };

        df.run_async().await;
    }
}
