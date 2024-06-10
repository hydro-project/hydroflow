use gossip_protocol::{ClientRequest, Key};
use hydroflow::DemuxEnum;

/// Convenience enum to represent a client request with the address of the client. Makes it
/// possible to use `demux_enum` in the surface syntax.
#[derive(Debug, DemuxEnum)]
pub enum ClientRequestWithAddress<A> {
    /// A get request with the key and the address of the client.
    Get { key: Key, addr: A },
    /// A set request with the key, value and the address of the client.
    Set { key: Key, value: String, addr: A },
    /// A delete request with the key and the address of the client.
    Delete { key: Key, addr: A },
}

impl<A> ClientRequestWithAddress<A> {
    /// Create a `ClientRequestWithAddress` from a `ClientRequest` and an address.
    pub fn from_request_and_address(request: ClientRequest, addr: A) -> Self {
        match request {
            ClientRequest::Get { key } => Self::Get { key, addr },
            ClientRequest::Set { key, value } => Self::Set { key, value, addr },
            ClientRequest::Delete { key } => Self::Delete { key, addr },
        }
    }
}
