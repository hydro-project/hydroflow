use hydroflow::hydroflow_syntax;
use hydroflow::util::cli::{ConnectedDirect, ConnectedSink, ConnectedSource};
use hydroflow::util::serialize_to_bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoMsg {
    pub msg_id: Value,
    pub echo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoOkMsg {
    pub echo: String,
    pub in_reply_to: Value,
}

impl EchoMsg {
    /// Generate EchoOkMsg response to this EchoMsg
    fn response(
        EchoMsg {
            echo,
            msg_id: source_msg_id,
        }: Self,
    ) -> EchoOkMsg {
        EchoOkMsg {
            echo,
            in_reply_to: source_msg_id,
        }
    }
}

#[hydroflow::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;

    // TODO: use ConnectedDemux?
    let echo_in = ports
        .port("echo_in")
        .connect::<ConnectedDirect>()
        .await
        .into_source();
    let echo_out = ports
        .port("echo_out")
        .connect::<ConnectedDirect>()
        .await
        .into_sink();

    let df = hydroflow_syntax! {
        input = source_stream(echo_in)
            -> map(Result::unwrap)
            -> map(|x| x.to_vec())
            -> map(String::from_utf8)
            -> map(Result::unwrap);

        output = map(|x| serde_json::to_string(&x))
            -> map(Result::unwrap)
            -> map(serialize_to_bytes)
            -> dest_sink(echo_out);


        input
        -> map(|x| serde_json::from_str::<EchoMsg>(&x).unwrap())
        //-> map(|x| EchoMsg {msg_id: x.msg_id, echo: x.echo + "hi"})
        -> map(EchoMsg::response)
        -> output;
    };

    hydroflow::util::cli::launch_flow(df).await;
}
