use hydroflow::pusherator::Pusherator;
use hydroflow::{hydroflow_syntax, tl};

fn main() {
    let mut df = hydroflow_syntax! {
        my_demux = source_iter(0..10) -> demux(|item, tl!(evens, odds)| {
            if 0 == item % 2 {
                evens.give(item);
            }
            else {
                odds.give(item)
            }
        });
        my_demux[evens] -> for_each(std::mem::drop);
        my_demux -> for_each(std::mem::drop);
    };
    df.run_available();
}
