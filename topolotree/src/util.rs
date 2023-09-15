use std::fmt::Debug;

use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::tokio_stream::wrappers::UnboundedReceiverStream;
use hydroflow::util::collect_ready_async;
use hydroflow::util::multiset::HashMultiSet;
use serde::Serialize;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::UnboundedSender;

use crate::{OperationPayload, Payload};

pub fn simulate_input<T: Debug + Serialize>(
    input_send: &mut UnboundedSender<Result<(u32, BytesMut), std::io::Error>>,
    (id, payload): (u32, Payload<T>),
) -> Result<(), SendError<Result<(u32, BytesMut), std::io::Error>>> {
    input_send.send(Ok((
        id,
        BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()),
    )))
}

pub fn simulate_operation(
    input_send: &mut UnboundedSender<Result<BytesMut, std::io::Error>>,
    payload: OperationPayload,
) -> Result<(), SendError<Result<BytesMut, std::io::Error>>> {
    input_send.send(Ok(BytesMut::from(
        serde_json::to_string(&payload).unwrap().as_str(),
    )))
}

pub async fn read_all(
    mut output_recv: &mut UnboundedReceiverStream<(u32, Bytes)>,
) -> HashMultiSet<(u32, Payload<i64>)> {
    let collected = collect_ready_async::<Vec<_>, _>(&mut output_recv).await;
    collected
        .iter()
        .map(|(id, bytes)| {
            (
                *id,
                serde_json::from_slice::<Payload<i64>>(&bytes[..]).unwrap(),
            )
        })
        .collect::<HashMultiSet<_>>()
}
