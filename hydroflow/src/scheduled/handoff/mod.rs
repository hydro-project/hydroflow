mod handoff_list;
mod null;
mod tee;
mod vector;

pub use handoff_list::HandoffList;
pub use null::NullHandoff;
pub use tee::TeeingHandoff;
pub use vector::VecHandoff;

use std::any::Any;

pub trait TryCanReceive<T> {
    fn try_give(&self, item: T) -> Result<T, T>;
}
pub trait CanReceive<T> {
    fn give(&self, item: T) -> T;
}

/**
 * A handle onto the metadata part of a [Handoff], with no element type.
 */
pub trait HandoffMeta: Any {
    // TODO(justin): more fine-grained info here.
    fn is_bottom(&self) -> bool;
}

pub trait Handoff: Default + HandoffMeta {
    type Inner;

    fn take_inner(&self) -> Self::Inner;

    fn give<T>(&self, item: T) -> T
    where
        Self: CanReceive<T>,
    {
        <Self as CanReceive<T>>::give(self, item)
    }

    fn try_give<T>(&self, item: T) -> Result<T, T>
    where
        Self: TryCanReceive<T>,
    {
        <Self as TryCanReceive<T>>::try_give(self, item)
    }
}
