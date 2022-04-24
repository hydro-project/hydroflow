use clap::{ArgEnum, Parser};
// use coordinator::run_coordinator;
use acceptor_blank::run_acceptor;
use hydroflow::tokio;
use proposer::run_proposer;
use proxy_leader::run_proxy_leader;

mod acceptor_blank;
mod proposer;
mod protocol;
mod proxy_leader;
mod raw;

#[derive(Clone, ArgEnum, Debug)]
enum Role {
    Proposer,
    Acceptor,
    ProxyLeader,
}

#[derive(Parser, Debug)]
struct CLIOpts {
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    port: u16,
    #[clap(long)]
    addr: String,
    #[clap(long)]
    id: u16,
    #[clap(long)]
    use_proxy: bool,
    #[clap(long)]
    raw: bool,
    #[clap(long, default_value_t = 3)]
    acceptors: u32,
    #[clap(long, default_value_t=3)]
    proxies: u32,
    #[clap(long, default_value_t=3)]
    num_messages: u32,
    // #[clap(long)]
    // output_dir: String,
}

pub struct Opts {
    // #[clap(long)]
    // path: String,
    // contain CLIOpts instance
    port: u16,
    addr: String,
    id: u16,
    use_proxy: bool,
    acceptor_addrs: Vec<String>,
    proxy_addrs: Vec<String>,
}

#[tokio::main]
async fn main() {
    let cli_opts = CLIOpts::parse();
    // create new Opts
    let mut opts = Opts {
        // path: cli_opts.path,
        port: cli_opts.port,
        addr: cli_opts.addr,
        id: cli_opts.id,
        use_proxy: cli_opts.use_proxy,
        acceptor_addrs: vec![],
        proxy_addrs: vec![],
        // output_dir: cli_opts.output_dir,
    };

    for port in 1400..1400 + cli_opts.acceptors {
        opts.acceptor_addrs
            .push(String::from(format!("localhost:{}", port)));
    }

    for port in 1200..1200+cli_opts.proxies {
        opts.proxy_addrs.push(String::from(format!("localhost:{}", port)));
    }

    // opts.use_proxy = false;
    // println!("{:?}", opts.use_proxy);

    if !cli_opts.raw {
        match cli_opts.role {
            Role::Proposer => run_proposer(opts).await,
            Role::Acceptor => run_acceptor(opts.port).await,
            Role::ProxyLeader => run_proxy_leader(opts).await,
        }
    } else {
        match cli_opts.role {
            Role::Proposer => raw::proposer::run(opts).await,
            Role::Acceptor => raw::acceptor_blank::run(opts).await,
            Role::ProxyLeader => raw::proxy_leader::run(opts).await,
        }
    }
}
