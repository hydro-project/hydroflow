use super::{Pusherator, PusheratorBuild};

pub struct Split<NextA, NextB> {
    next_a: NextA,
    next_b: NextB,
}
impl<NextA, NextB> Pusherator for Split<NextA, NextB>
where
    NextA: Pusherator,
    NextB: Pusherator,
{
    type Item = (NextA::Item, NextB::Item);
    fn give(&mut self, item: Self::Item) {
        let (item_a, item_b) = item;
        self.next_a.give(item_a);
        self.next_b.give(item_b);
    }
}
impl<NextA, NextB> Split<NextA, NextB>
where
    NextA: Pusherator,
    NextB: Pusherator,
{
    pub fn new(next_a: NextA, next_b: NextB) -> Self {
        Self { next_a, next_b }
    }
}

pub struct SplitBuild<Prev, NextA>
where
    Prev: PusheratorBuild,
    NextA: Pusherator,
{
    prev: Prev,
    next_a: NextA,
}
impl<Prev, NextA> SplitBuild<Prev, NextA>
where
    Prev: PusheratorBuild,
    NextA: Pusherator,
{
    pub fn new(prev: Prev, next_a: NextA) -> Self {
        Self { prev, next_a }
    }
}
impl<Prev, NextA, ItemB> PusheratorBuild for SplitBuild<Prev, NextA>
where
    Prev: PusheratorBuild<ItemOut = (NextA::Item, ItemB)>,
    NextA: Pusherator,
{
    type ItemOut = ItemB;

    type Output<Next: Pusherator<Item = Self::ItemOut>> = Prev::Output<Split<NextA, Next>>;
    fn push_to<Next>(self, input: Next) -> Self::Output<Next>
    where
        Next: Pusherator<Item = Self::ItemOut>,
    {
        self.prev.push_to(Split::new(self.next_a, input))
    }
}
