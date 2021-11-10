use std::rc::Rc;

use once_cell::unsync::OnceCell;

/**
 * Creates a once channel where the [SendOnce] end can set a value which the [Once] end will receive and store.
 */
pub fn once<T>() -> (SendOnce<T>, Once<T>) {
    let channel = Rc::new(OnceCell::new());
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
    channel: Rc<OnceCell<T>>,
}
impl<T> SendOnce<T> {
    pub fn send(self, item: T) {
        let result = self.channel.set(item);
        assert!(result.is_ok());
    }
}

/**
 * The receiving half of a once channel.
 */
pub struct Once<T> {
    channel: Rc<OnceCell<T>>,
}
impl<T> Once<T> {
    pub fn get(&mut self) -> &mut T {
        Rc::get_mut(&mut self.channel)
            .expect("Called Once::get() before value is set.")
            .get_mut()
            .unwrap()
    }
}
