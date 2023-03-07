use hydroflow::{hydroflow_syntax, util::deserialize_from_bytes};

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::connection::hydro_cli_init().await;
    let bar_recv = ports.remove("bar").unwrap().0.unwrap();

    let mut df = hydroflow_syntax! {
        bar = source_stream(bar_recv)
            -> map(|x| deserialize_from_bytes(x.unwrap()));

        bar[0] -> for_each(|s: String| println!("echo {}", s));
    };

    df.run_async().await;
}
