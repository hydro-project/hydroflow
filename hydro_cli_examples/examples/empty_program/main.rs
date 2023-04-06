use hydroflow::{
    hydroflow_syntax,
    util::cli::{ConnectedBidi, ConnectedSource},
};

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    loop {}
}
