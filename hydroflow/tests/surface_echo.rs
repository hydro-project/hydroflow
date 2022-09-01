use std::time::Duration;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[test]
pub fn test_echo() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (lines_send, lines_recv) = tokio::sync::mpsc::unbounded_channel::<String>();

    //use tokio::io::{AsyncBufReadExt, BufReader};
    // use tokio_stream::wrappers::LinesStream;
    // let stdin_lines = LinesStream::new(BufReader::new(tokio::io::stdin()).lines());
    let stdout_lines = tokio::io::stdout();

    let mut df: Hydroflow = hydroflow_syntax! {
        recv_stream(lines_recv) -> map(|line| line + "\n") -> send_async(stdout_lines);
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    lines_send.send("Hello".to_owned()).unwrap();
    lines_send.send("World".to_owned()).unwrap();
    df.run_available();

    lines_send.send("Hello".to_owned()).unwrap();
    lines_send.send("World".to_owned()).unwrap();
    df.run_available();

    // Allow background thread to catch up.
    std::thread::sleep(Duration::from_secs(1));
}

#[derive(Clone)]
pub struct UnboundedSenderEq<T>(pub UnboundedSender<T>);
impl<T> PartialEq for UnboundedSenderEq<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.same_channel(&other.0)
    }
}
impl<T> Eq for UnboundedSenderEq<T> {}

// pub struct UnboundedReceiverEq<T>(UnboundedReceiver<T>);
// impl<T> PartialEq for UnboundedReceiverEq<T> {
//     fn eq(&self, other: &Self) -> bool {
//         std::ptr::eq(self, other)
//     }
// }
// impl<T> Eq for UnboundedReceiverEq<T> {}

#[test]
pub fn test_shuffle_all_to_all() {
    // // initialize 10 nodes
    // let mut channel_vec: Vec<(UnboundedSenderEq<String>, UnboundedReceiverEq<String>)> = (1..10)
    //     .map(|_| tokio::sync::mpsc::unbounded_channel::<String>())
    //     .map(|(send, recv)| (UnboundedSenderEq(send), UnboundedReceiverEq(recv)))
    //     .collect();

    let mut send_vec = Vec::new();
    let mut recv_vec = Vec::new();
    for _ in 0..10 {
        let (send, recv) = tokio::sync::mpsc::unbounded_channel::<String>();
        send_vec.push(UnboundedSenderEq(send));
        recv_vec.push(recv);
    }

    let mut dfs = recv_vec
        .into_iter()
        .enumerate()
        .map(|(i, recv)| {
            let sends = send_vec
                .iter()
                .enumerate()
                .filter(|&(j, _)| i != j)
                .map(|(_, s)| s)
                .cloned()
                .collect::<Vec<_>>();
            let df = hydroflow_syntax! {
                members = recv_iter(sends);
                data = recv_stream(recv) -> map(|line| format!("Recv at {}: {}", i, line)) -> map(|line| { println!("{}", line); line });
                my_join = join();
                data -> map(|d| ((), d)) -> [0]my_join;
                members -> map(|m| ((), m)) -> [1]my_join;
                my_join -> for_each(|((), (d, UnboundedSenderEq(m)))| { m.send(d).unwrap(); });
            };
            println!(
                "{}",
                df.serde_graph()
                    .expect("No graph found, maybe failed to parse.")
                    .to_mermaid()
            );
            df
        })
        .collect::<Vec<_>>();

    send_vec[5].0.send("my message".to_owned()).unwrap();

    loop {
        for df in dfs.iter_mut() {
            df.run_available();
        }
    }

    // df.run_available();

    // lines_send.send("Hello".to_owned()).unwrap();
    // lines_send.send("World".to_owned()).unwrap();
    // df.run_available();

    // lines_send.send("Hello".to_owned()).unwrap();
    // lines_send.send("World".to_owned()).unwrap();
    // df.run_available();

    // // Allow background thread to catch up.
    // std::thread::sleep(Duration::from_secs(1));
}

// async fn async_test() {
//     LocalSet::new()
//         .run_until(async {
//             let (a_send, a_recv) = tokio::sync::mpsc::unbounded_channel::<usize>();
//             let (b_send, b_recv) = tokio::sync::mpsc::unbounded_channel::<usize>();

//             tokio::task::spawn_local(async move {
//                 let mut flow = hydroflow_syntax! {
//                     recv_stream(a_recv) -> for_each(|x| { b_send.send(x).unwrap(); });
//                 };
//                 flow.run_async().await.unwrap();
//             });
//             tokio::task::spawn_local(async move {
//                 let mut flow = hydroflow_syntax! {
//                     recv_stream(b_recv) -> for_each(|x| println!("{}", x));
//                 };
//                 flow.run_async().await.unwrap();
//             });

//             a_send.send(1).unwrap();
//             a_send.send(2).unwrap();
//             a_send.send(3).unwrap();

//             tokio::task::yield_now().await;
//         })
//         .await;
// }
