use std::marker::PhantomData;

use super::Pusherator;

variadics::variadic! {
    /// A variadic list of Pusherators.
    pub variadic<T> PusheratorList where T: Pusherator {}
}

pub struct Demux<Func, Nexts, Item> {
    func: Func,
    nexts: Nexts,
    _phantom: PhantomData<fn(Item)>,
}
impl<Func, Nexts, Item> Pusherator for Demux<Func, Nexts, Item>
where
    Func: FnMut(Item, &mut Nexts),
    Nexts: PusheratorList,
{
    type Item = Item;
    fn give(&mut self, item: Self::Item) {
        (self.func)(item, &mut self.nexts);
    }
}
impl<Func, Nexts, Item> Demux<Func, Nexts, Item>
where
    Func: FnMut(Item, &mut Nexts),
    Nexts: PusheratorList,
{
    pub fn new(func: Func, nexts: Nexts) -> Self {
        Self {
            func,
            nexts,
            _phantom: PhantomData,
        }
    }
}
