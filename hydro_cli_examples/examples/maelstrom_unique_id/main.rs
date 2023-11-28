use hydroflow::hydroflow_syntax;
use hydroflow::util::cli::{ConnectedDirect, ConnectedSink, ConnectedSource};
use hydroflow::util::serialize_to_bytes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generate {
    pub msg_id: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOk {
    pub id: Value,
    pub in_reply_to: Value,
}

impl Generate {
    /// Generate GenerateOk response to this Generate message
    pub fn respond(self, i: usize, node_id: usize) -> GenerateOk {
        let id = json!([i, node_id]);

        GenerateOk {
            id,
            in_reply_to: self.msg_id,
        }
    }
}

#[hydroflow::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let node_id = ports.node_id;

    // TODO: use ConnectedDemux?
    let gen_in = ports
        .port("gen_in")
        .connect::<ConnectedDirect>()
        .await
        .into_source();
    let ok_out = ports
        .port("ok_out")
        .connect::<ConnectedDirect>()
        .await
        .into_sink();

    let df = hydroflow_syntax! {
        input = source_stream(gen_in)
            -> map(Result::unwrap)
            -> map(|x| x.to_vec())
            -> map(String::from_utf8)
            -> map(Result::unwrap);

        output = map(|x| serde_json::to_string(&x))
            -> map(Result::unwrap)
            -> map(serialize_to_bytes)
            -> dest_sink(ok_out);


        input
        -> map(|x| serde_json::from_str::<Generate>(&x).unwrap())
        -> enumerate::<'static>() //-> enumerate() will fail!
        -> map(|(i, x)| x.respond(i, node_id))
        -> output;
    };

    hydroflow::util::cli::launch_flow(df).await;
}
