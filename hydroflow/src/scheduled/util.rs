use std::cell::RefCell;
use std::rc::Rc;

/**
 * Creates a once channel where the [SendOnce] end can set a value which the [Once] end will receive and store.
 */
pub fn once<T>() -> (SendOnce<T>, Once<T>) {
    let channel = Rc::new(RefCell::new(None));
    let sender = SendOnce {
        channel: channel.clone(),
    };
    let once = Once { channel };
    (sender, once)
}

/**
 * The sending half of a once channel.
 */
pub struct SendOnce<T> {
    channel: Rc<RefCell<Option<T>>>,
}
impl<T> SendOnce<T> {
    pub fn send(self, item: T) {
        let old_item = self.channel.borrow_mut().replace(item);
        assert!(old_item.is_none());
    }
}

/**
 * The receiving half of a once channel.
 */
pub struct Once<T> {
    channel: Rc<RefCell<Option<T>>>,
}
impl<T> Once<T> {
    pub fn get(&self) -> std::cell::RefMut<'_, T> {
        std::cell::RefMut::map(self.channel.borrow_mut(), |x| x.as_mut().unwrap())
    }
}
