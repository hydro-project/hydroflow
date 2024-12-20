use hydroflow::util::demux_enum::DemuxEnum;
use hydroflow::dfir_syntax;

fn main() {
    #[derive(DemuxEnum)]
    enum Shape {
    }

    let mut df = dfir_syntax! {
        my_demux = source_iter([]) -> demux_enum::<Shape>();
        my_demux[Square] -> for_each(std::mem::drop);
    };
    df.run_available();
}
