use clap::{Parser, ValueEnum};
use hydroflow::tokio;
// use hydroflow::util::{bind_udp_bytes, ipv4_resolve};//Not using bind_udp_bytes yet.
use hydroflow::util::ipv4_resolve;
use hydroflow::hydroflow_syntax;
use std::net::SocketAddr;

#[derive(Clone, ValueEnum, Debug)]
enum Role {
    Client,
    Server,
}
#[derive(Clone, ValueEnum, Debug)]
enum GraphType {
    Mermaid,
    Dot,
    Json,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(value_enum, long)]
    role: Role,
    #[clap(long, value_parser = ipv4_resolve)]
    addr: Option<SocketAddr>,
    #[clap(long, value_parser = ipv4_resolve)]
    server_addr: Option<SocketAddr>,
    #[clap(value_enum, long)]
    graph: Option<GraphType>,
}

#[tokio::main]
async fn main() {
    let (local_send, local_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (parent_send, parent_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (left_send, left_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (right_send, right_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    // let (query_send, query_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>(); //Not implemented yet
    

    let (to_right_tx, _to_right_rx) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (to_left_tx, _to_left_rx) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (to_parent_tx, _to_parent_rx) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (to_query_tx, _to_query_rx) = hydroflow::util::unbounded_channel::<(usize, usize)>();


    println!("Server live!");


    let my_merge_function = |(mut current_time, mut current_value), (x, y)| {
        if x > current_time {
            current_time = x;
            current_value = y;
        }

        (current_time, current_value)
    };

    let time_incrementer = |(mut current_time, mut _current_value), new_value| {
        current_time += 1;

        (current_time, new_value)
    };

    let update_value = |mut current_value, new_value| {
        current_value += new_value;

        current_value
    };


    parent_send.send((1, 2)).unwrap();
    left_send.send((1,5)).unwrap();
    right_send.send((0,0)).unwrap();
    local_send.send((0,0,)).unwrap();
    

    let mut df = hydroflow_syntax! {


        from_parent = source_stream(parent_recv)
            -> inspect(|x| println!("from_parent: {x:?}"))
            -> fold::<'static>((0,0), my_merge_function)
            -> map(|(_current_time, current_value)| current_value)
            -> tee();

        from_left = source_stream(left_recv)
            -> inspect(|x| println!("from_left: {x:?}"))
            -> fold::<'static>((0,0), my_merge_function)
            -> map(|(_current_time, current_value)| current_value)
            -> tee();
        
        from_right = source_stream(right_recv)
        -> inspect(|x| println!("from_left: {x:?}"))
        -> fold::<'static>((0,0), my_merge_function)
        -> map(|(_current_time, current_value)| current_value)
        -> tee();

        from_local = source_stream(local_recv) //TODO implement
        -> inspect(|x| println!("from_left: {x:?}"))
        -> fold::<'static>((0,0), my_merge_function)
        -> map(|(_current_time, current_value)| current_value)
        -> tee();
        
        to_right = merge();

        from_parent -> to_right;
        from_left -> to_right;
        from_local -> to_right;

        to_right
            -> fold::<'tick>(0, update_value)
            -> fold::<'static>((0, 0), time_incrementer)
            -> inspect(|x| println!("to_right: {x:?}"))
            -> for_each(|x| to_right_tx.send(x).unwrap()); //send result to output channel

        to_left = merge();

        from_parent -> to_left;
        from_right -> to_left;
        from_local -> to_left;

        to_left
            -> fold::<'tick>(0, update_value)
            -> fold::<'static>((0, 0), time_incrementer)
            -> inspect(|x| println!("to_left: {x:?}"))
            -> for_each(|x| to_left_tx.send(x).unwrap()); //send result to output channel

        to_parent = merge();

        from_right -> to_parent;
        from_left -> to_parent;
        from_local -> to_parent;

        to_parent
            -> fold::<'tick>(0, update_value)
            -> fold::<'static>((0, 0), time_incrementer)
            -> inspect(|x| println!("to_parent: {x:?}"))
            -> for_each(|x| to_parent_tx.send(x).unwrap()); //send result to output channel
    
        to_query = merge();

        from_parent -> to_query;
        from_left -> to_query;
        from_right -> to_query;
        from_local -> to_query;

        to_query
            -> fold::<'tick>(0, update_value)
            -> fold::<'static>((0, 0), time_incrementer)
            -> inspect(|x| println!("to_query: {x:?}"))
            -> for_each(|x| to_query_tx.send(x).unwrap()); //send result to output channel

        // // NW channels
        // outbound_chan = merge() -> dest_sink_serde(outbound);
        // inbound_chan = source_stream_serde(inbound)
        //     -> demux(|(m, a), var_args!(puts, gets, errs)| match m {
        //             KVSMessage::Put {..} => puts.give((m, a)),
        //             KVSMessage::Get {..} => gets.give((m, a)),
        //             _ => errs.give((m, a)),
        //     });
        // puts = inbound_chan[puts] -> tee();
        // gets = inbound_chan[gets] -> tee();
        // inbound_chan[errs] -> for_each(|(m, a)| println!("Received unexpected message type {:?} from {:?}", m, a));

        // puts[0] -> for_each(|(m, a)| println!("Got a Put {:?} from {:?}", m, a));
        // gets[0] -> for_each(|(m, a)| println!("Got a Get {:?} from {:?}", m, a));

        // parsed_puts = puts[1] -> filter_map(|(m, a)| {
        //     match m {
        //         KVSMessage::Put{key, value} => Some((key, value, a)),
        //         _ => None }
        //     }) -> tee();
        // parsed_gets = gets[1] -> filter_map(|(m, a)| {
        //     match m {
        //         KVSMessage::Get{key} => Some((key, a)),
        //         _ => None }
        //     });

        // // ack puts
        // parsed_puts[0] -> map(| (key, value, client) |
        //                         (KVSMessage::Response{key, value}, client))
        //     -> [0]outbound_chan;

        // // join PUTs and GETs by key
        // lookup = join()->tee();
        // parsed_puts[1] -> map(|(key, value, _)| (key, value)) -> [0]lookup;
        // parsed_gets -> [1]lookup;
        // lookup[0] -> for_each(|t| println!("Found a match: {:?}", t));

        // // send lookup responses back to the client address from the GET
        // lookup[1] -> map(|(key, (value, client))| (KVSMessage::Response{key, value}, client)) -> [1]outbound_chan;
    };

    let serde_graph = df
        .serde_graph()
        .expect("No graph found, maybe failed to parse.");
            println!("{}", serde_graph.to_mermaid());

    // df.run_async().await.unwrap();
    df.run_tick();
    df.run_tick();
    df.run_tick();
}
//     let opts = Opts::parse();
//     let addr = opts.addr.unwrap();

//     match opts.role {
//         Role::Client => {
//             let (outbound, inbound, _) = bind_udp_bytes(addr).await;
//             println!("Client is bound to {:?}", addr);
//             println!("Attempting to connect to server at {:?}", opts.server_addr);
//             run_client(
//                 outbound,
//                 inbound,
//                 opts.server_addr.unwrap(),
//                 opts.graph.clone(),
//             )
//             .await;
//         }
//         Role::Server => {
//             let (outbound, inbound, _) = bind_udp_bytes(addr).await;
//             println!("Listening on {:?}", opts.addr.unwrap());
//             run_server(outbound, inbound, opts.graph.clone()).await;
//         }
//     }
// }
