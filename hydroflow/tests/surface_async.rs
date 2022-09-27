use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::time::Duration;

use hydroflow::hydroflow_syntax;

#[tokio::test]
pub async fn test_futures_stream_sink() -> Result<(), Box<dyn Error>> {
    const MAX: usize = 20;

    let (mut send, recv) = hydroflow::futures::channel::mpsc::channel::<usize>(5);
    send.try_send(0).unwrap();

    let seen = <Rc<RefCell<Vec<usize>>>>::default();
    let seen_inner = Rc::clone(&seen);

    let mut df = hydroflow_syntax! {
        recv_stream(recv)
            -> map(|x| { seen_inner.borrow_mut().push(x); x })
            -> map(|x| x + 1)
            -> filter(|&x| x < MAX)
            -> sink_async(send);
    };

    tokio::select! {
        _ = df.run_async() => (),
        _ = tokio::time::sleep(Duration::from_secs(1)) => (),
    };

    assert_eq!(&std::array::from_fn::<_, MAX, _>(|i| i), &**seen.borrow());

    Ok(())
}
