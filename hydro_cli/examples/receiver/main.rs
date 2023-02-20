use std::collections::HashMap;

use hydroflow::{
    hydroflow_syntax,
    util::{connection::ConnectionPipe, unix_lines},
};
use tokio::net::UnixListener;

#[tokio::main]
async fn main() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    let connection_pipes =
        serde_json::from_str::<HashMap<String, ConnectionPipe>>(trimmed).unwrap();

    if let Some(ConnectionPipe::UnixSocket(s)) = connection_pipes.get("bar") {
        let server = UnixListener::bind(s).unwrap();
        println!("ready");

        let client_stream = server.accept().await.unwrap().0;
        let (_, client_recv) = unix_lines(client_stream);

        let mut start_buf = String::new();
        std::io::stdin().read_line(&mut start_buf).unwrap();
        if start_buf != "start\n" {
            panic!("expected start");
        }

        let mut df = hydroflow_syntax! {
            bar = source_stream(client_recv)
                -> map(|x| x.unwrap())
                -> tee();

            bar[0] -> for_each(|s| println!("echo {}", s));
        };

        df.run_async().await;
    }
}
