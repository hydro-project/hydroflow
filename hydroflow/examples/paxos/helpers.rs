use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;

use tokio_stream::wrappers::IntervalStream;

use crate::Config;

pub fn parse_out<T: std::str::FromStr>(line: String) -> Option<T> {
    match line.trim().parse::<T>() {
        Ok(the_xid) => Some(the_xid),
        Err(_) => None,
    }
}

pub fn get_phase_1_addr(addr: SocketAddr) -> SocketAddr {
    SocketAddr::new(addr.ip(), addr.port() + 1)
}
pub fn get_phase_2_addr(addr: SocketAddr) -> SocketAddr {
    SocketAddr::new(addr.ip(), addr.port() + 2)
}

pub fn periodic(timeout: u16) -> IntervalStream {
    let start = tokio::time::Instant::now() + tokio::time::Duration::from_millis(timeout.into());
    IntervalStream::new(tokio::time::interval_at(
        start,
        tokio::time::Duration::from_millis(timeout.into()),
    ))
}

pub fn get_config(path: impl AsRef<Path>) -> Config {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}
