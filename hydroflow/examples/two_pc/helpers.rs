pub fn parse_out<T: std::str::FromStr>(line: String) -> Option<T> {
    match line.trim().parse::<T>() {
        Ok(the_xid) => Some(the_xid),
        Err(_) => None,
    }
}

use rand::Rng;
pub fn decide(odds: u8) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..100) <= odds
}

use std::net::SocketAddr;
pub fn get_output_file(addr: SocketAddr) -> String {
    format!("{:?}.txt", addr.port())
}