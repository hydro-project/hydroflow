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
    let once = Once::Unresolved { channel };
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
pub enum Once<T> {
    Resolved(T),
    Unresolved { channel: Rc<RefCell<Option<T>>> },
    Errored,
}
impl<T> Once<T> {
    pub fn get(&mut self) -> &mut T {
        match self {
            Self::Resolved(item) => return item,
            Self::Errored => panic!("Initialization previously failed."),
            _ => {}
        }

        *self = match std::mem::replace(self, Self::Errored) {
            Self::Unresolved { channel } => {
                let item = Rc::try_unwrap(channel)
                    .ok()
                    .expect("Initialization failed: channel is still live.")
                    .into_inner()
                    .expect("Initialization failed: item never sent.");
                Self::Resolved(item)
            }
            v => v,
        };
        self.get()
    }
}
