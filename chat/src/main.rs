use clap::{ArgEnum, Parser};
use client::run_client;
use hydroflow::tokio;
use serde::{de::DeserializeOwned, Serialize};
use server::run_server;

mod client;
mod server;

// This is a distributed version of the covid tracing app. It somewhat
// arbitrarily splits apart two "responsibilities" of the app to exercise
// network communication.

#[derive(Clone, ArgEnum, Debug)]
enum Role {
    Client,
    Server,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(long)]
    name: String,
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    port: usize,
    #[clap(long)]
    addr: String,
}

// TODO(justin): this trait kind of sucks but it's a placeholder.
trait Encode {
    fn encode(&self, v: &mut Vec<u8>);
}

impl<T> Encode for T
where
    T: Serialize,
{
    fn encode(&self, v: &mut Vec<u8>) {
        v.extend(serde_json::to_vec(self).unwrap());
    }
}

trait Decode {
    fn decode(v: bytes::Bytes) -> Self;
}

// TODO(justin): figure out how to do this without DeserializeOwned
impl<T> Decode for T
where
    T: 'static + DeserializeOwned,
{
    fn decode(v: bytes::Bytes) -> Self {
        let st = std::str::from_utf8(&v).unwrap().to_owned();
        serde_json::from_str(&st).unwrap()
    }
}

const SERVER_ADDR: u32 = 0;
const CLIENT1_ADDR: u32 = 1;
const CLIENT2_ADDR: u32 = 2;

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    match opts.role {
        Role::Client => {
            run_client(opts).await;
        }
        Role::Server => {
            run_server(opts).await;
        }
    }
}
