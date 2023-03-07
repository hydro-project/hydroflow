use crdts::CvRDT;
use crdts::GSet;
use hydroflow::hydroflow_syntax;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut df = hydroflow_syntax! {
        x = stamp::<GSet>();

        source_iter(0..2) -> map(|x| {
            let mut set = GSet::new();
            set.insert(x as usize);
            set
        }) -> [0]x;

        source_iter(0..10) -> [1]x;

        x -> for_each(|x| { println!("{x:?}"); });
    };

    df.run_async().await;
}
