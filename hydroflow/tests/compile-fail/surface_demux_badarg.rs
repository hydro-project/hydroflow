use hydroflow::{hydroflow_syntax, var_args};

fn main() {
    let mut df = hydroflow_syntax! {
        my_demux = source_iter(0..10) -> demux(std::mem::drop);
        my_demux[a] -> for_each(std::mem::drop);
        my_demux[b] -> for_each(std::mem::drop);
        my_demux[c] -> for_each(std::mem::drop);
    };
    df.run_available();
}
