use crate::{Decode, Encode, Opts};

use std::time::Duration;

use hydroflow::compiled::{pull::SymmetricHashJoin, IteratorToPusherator, PusheratorBuild};
use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::ctx::{RecvCtx, SendCtx};
use hydroflow::scheduled::{handoff::VecHandoff, net::Message};
use hydroflow::tokio::net::TcpListener;
use hydroflow::{
    scheduled::{graph::Hydroflow, graph_ext::GraphExt},
    tl, tt,
};
use rand::Rng;

pub(crate) async fn run_client(opts: Opts) {
    let mut df = Hydroflow::new();
}
