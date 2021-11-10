use std::any::Any;
use super::{Handoff, HandoffMeta};

#[derive(Default)]
pub struct NullHandoff;
impl Handoff for NullHandoff {
    type Inner = ();
    fn take_inner(&self) -> Self::Inner {}
}

impl HandoffMeta for NullHandoff {
    fn any_ref(&self) -> &dyn Any {
        self
    }

    fn is_bottom(&self) -> bool {
        true
    }
}
