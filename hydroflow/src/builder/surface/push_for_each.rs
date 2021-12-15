use super::PushSurfaceReversed;

use std::marker::PhantomData;

use crate::builder::build::push_for_each::ForEachPushBuild;
use crate::builder::connect::NullPushConnect;
use crate::tt;

pub struct ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(In),
{
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Func, In> ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(In),
{
    pub fn new(func: Func) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }
}

impl<Func, In> PushSurfaceReversed for ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(In),
{
    type OutputHandoffs = tt!();

    type ItemIn = In;

    type Connect = NullPushConnect;
    type Build = ForEachPushBuild<Func, In>;

    fn into_parts(self) -> (Self::Connect, Self::Build) {
        let connect = NullPushConnect::new();
        let build = ForEachPushBuild::new(self.func);
        (connect, build)
    }
}
