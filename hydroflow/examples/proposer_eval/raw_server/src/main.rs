use clap::{ArgEnum, Parser};

mod acceptor_blank;
mod proposer;
mod protocol;
mod proxy_leader;

//use acceptor_blank;
//use proposer;
//use proxy_leader;

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
    #[clap(long, default_value_t = 3)]
    acceptors: u32,
    #[clap(long, default_value_t = 3)]
    proxies: u32,
    #[clap(long, default_value_t = 3)]
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

fn main() {
    let cli_opts = CLIOpts::parse();
    let mut opts = Opts {
        port: cli_opts.port,
        addr: cli_opts.addr,
        id: cli_opts.id,
        use_proxy: cli_opts.use_proxy,
        acceptor_addrs: vec![],
        proxy_addrs: vec![],
    };

    for port in 1400..1400 + cli_opts.acceptors {
        opts.acceptor_addrs
            .push(String::from(format!("localhost:{}", port)));
    }

    for port in 1200..1200 + cli_opts.proxies {
        opts.proxy_addrs
            .push(String::from(format!("localhost:{}", port)));
    }

    // opts.use_proxy = false;
    // println!("{:?}", opts.use_proxy);

    match cli_opts.role {
        Role::Proposer => proposer::run(opts),
        Role::Acceptor => acceptor_blank::run(String::from(format!("localhost:{}", opts.port))),
        Role::ProxyLeader => proxy_leader::run(opts),
    }
}
