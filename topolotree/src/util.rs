use std::fmt::Debug;

use hydroflow::bytes::BytesMut;
use serde::Serialize;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::UnboundedSender;

use crate::Payload;

pub fn simulate_input<T: Debug + Serialize>(
    input_send: &mut UnboundedSender<Result<(u32, BytesMut), std::io::Error>>,
    (id, payload): (u32, Payload<T>),
) -> Result<(), SendError<Result<(u32, BytesMut), std::io::Error>>> {
    input_send.send(Ok((
        id,
        BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()),
    )))
}
