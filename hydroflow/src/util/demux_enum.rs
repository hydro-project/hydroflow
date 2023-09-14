pub use hydroflow_macro::DemuxEnum;
use pusherator::demux::PusheratorList;
use pusherator::for_each::ForEach;
use pusherator::Pusherator;
use variadics::{var_args, var_expr, var_type};

pub trait DemuxEnum<Nexts>
where
    Nexts: PusheratorList,
{
    fn demux_enum(self, outputs: &mut Nexts);
}
