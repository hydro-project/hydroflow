use std::net::SocketAddr;

use bytes::Bytes;
use dfir_rs::dfir_syntax;
use dfir_rs::scheduled::graph::Dfir;
use futures::stream::SplitSink;
use tokio_util::codec::LengthDelimitedCodec;
use tokio_util::udp::UdpFramed;

use crate::structs::LineItem;
use crate::test_data::client_class_iter;

pub(crate) async fn orig_flow(
    shopping: impl Iterator<Item = (usize, LineItem)> + 'static,
    out_addr: SocketAddr,
    out: SplitSink<UdpFramed<LengthDelimitedCodec>, (Bytes, SocketAddr)>,
) -> Dfir<'static> {
    let client_class = client_class_iter();

    // This is the straightforward single-transducer sequential case.
    // For each Request in "shopping" we look up its "client_class" (basic or prime)
    // via a join operator, then we group by (client, class), and for each such pair
    // we grow a separate vector of ClLineItems. No seal is needed in the sequential case.
    dfir_syntax! {
        // the original flow
        source_iter(shopping) -> [0]lookup_class;
        source_iter(client_class) -> [1]lookup_class;
        lookup_class = join::<'static>()
          -> map(|(client, (li, class))| ((client, class), li))
          -> fold_keyed::<'static>(Vec::new, Vec::push)
          -> map(|m| (m, out_addr)) -> dest_sink_serde(out);
    }
}
