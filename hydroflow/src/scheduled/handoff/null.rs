use super::{Handoff, HandoffMeta};

#[derive(Default)]
pub struct NullHandoff;
impl Handoff for NullHandoff {
    type Inner = ();
    fn take_inner(&mut self) -> Self::Inner {}
}

impl HandoffMeta for NullHandoff {
    fn is_bottom(&self) -> bool {
        true
    }
}
