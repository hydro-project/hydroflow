pub use hydroflow_macro::DemuxEnum;
use pusherator::demux::PusheratorList;
use pusherator::for_each::ForEach;
use pusherator::{Pusherator, PusheratorBuild};
use variadics::{var_args, var_expr, var_type, Variadic};

// TODO(mingwei): doc this trait should be derived only.
pub trait DemuxEnum<Nexts>: DemuxEnumItems
where
    Nexts: PusheratorListForItems<Self::Items>,
{
    fn demux_enum(self, outputs: &mut Nexts);
}

pub trait DemuxEnumItems {
    type Items: Variadic;
}

pub trait PusheratorListForItems<Items>: PusheratorList
where
    Items: Variadic,
{
}
impl<HeadPush, RestPush, Head, Rest> PusheratorListForItems<(Head, Rest)> for (HeadPush, RestPush)
where
    HeadPush: Pusherator<Item = Head>,
    RestPush: PusheratorListForItems<Rest>,
    Rest: Variadic,
{
}
impl PusheratorListForItems<()> for () {}
