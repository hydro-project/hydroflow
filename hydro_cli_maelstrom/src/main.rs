use std::error::Error;
use std::path::PathBuf;

use tokio::io::stderr;

mod cli_refs;
mod config;
mod hydro_interact;
mod maelstrom;
mod ports;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // this should be serialized input to the program
    let config = config::Config {
        binary: PathBuf::from(
            r"/mnt/c/Users/rhala/Code/hydroflow/target/debug/examples/maelstrom_broadcast",
        ),
        // binary: PathBuf::from(r"C://Users/rhala/Code/hydroflow/target/debug/examples/maelstrom_broadcast"),
        ports: vec![
            config::Port::Source(config::MaelstromPort::new(
                "read_in".to_string(),
                "read".to_string(),
            )),
            config::Port::Sink(config::MaelstromPort::new(
                "readok_out".to_string(),
                "read_ok".to_string(),
            )),
            config::Port::Source(config::MaelstromPort::new(
                "topology_in".to_string(),
                "topology".to_string(),
            )),
            config::Port::Sink(config::MaelstromPort::new(
                "topologyok_out".to_string(),
                "topology_ok".to_string(),
            )),
            config::Port::Source(config::MaelstromPort::new(
                "broadcast_in".to_string(),
                "broadcast".to_string(),
            )),
            config::Port::Sink(config::MaelstromPort::new(
                "broadcastok_out".to_string(),
                "broadcast_ok".to_string(),
            )),
            config::Port::Custom(config::CustomPort::new(
                "gossip_in".to_string(),
                "gossip_out".to_string(),
            )),
        ],
    };

    // Spawn the child process
    let child = utils::spawn_child(&config.binary)?;
    let (mut child, mut child_stdin, mut child_stdout, child_stderr) = child;

    // Perform initialization handshake with maelstrom
    let mael_cfg = maelstrom::maelstrom_init().await?;

    // Send setup string which initializes source ports (source from the perspective of the wrapped crate)
    let setup_string = ports::make_setup_string(&config.ports, mael_cfg.node_count)?;
    utils::write_to_child(&mut child_stdin, &setup_string).await?;

    // Recieves the ready message from hydro cli with bound source ports
    let ready_message = utils::read_from_child(&mut child_stdout).await?;

    // Connect to all source ports
    let source_connections =
        hydro_interact::connect_to_sources(&ready_message, &config.ports).await?;

    // Bind to all sink ports
    let sink_connections = ports::bind_to_sinks(&config.ports, mael_cfg.node_count).await?;

    // Send start string with sink bindings
    let start_string =
        ports::make_start_string(&sink_connections, mael_cfg.node_id, &mael_cfg.node_names)?;

    // Spawn thread to demux stdin to the relevant ports
    hydro_interact::spawn_input_handler(source_connections, &mael_cfg.node_names);
    hydro_interact::spawn_output_handlers(
        sink_connections,
        mael_cfg.node_id,
        &mael_cfg.node_names,
    )?;

    // Send start string to finish hydro cli initialization
    utils::write_to_child(&mut child_stdin, &start_string).await?;

    // Forward child's standard out and error stderr for maelstrom logging
    tokio::task::spawn(utils::debug_link(
        child_stdout,
        stderr(),
        "child-stdout".into(),
    ));
    tokio::task::spawn(utils::debug_link(
        child_stderr,
        stderr(),
        "child-stderr".into(),
    ));

    child.wait().await?;

    Ok(())
}

// Example inputs:
// INIT PAYLOAD SHOULD ALWAYS BE FIRST:
// {"src": "c1", "dest": "n1","body": {"msg_id": 0,"type": "init", "node_id": "n1", "node_ids": ["n1", "n2", "n3"]}}
//
// echo examples:
// {"src": "c1", "dest": "n1","body": {"msg_id": 1,"type": "echo", "echo": "hello world!"}}
//
// unique id examples:
// {"src": "c1", "dest": "n1","body": {"msg_id": 1, "type": "generate"}}
//
// broadcast examples:
// {"src": "n3", "dest": "n1","body": {"msg_id": 1,"type": "topology", "topology": {"n1": ["n2"], "n2": ["n1", "n2"], "n3": ["n2"]}}}
// {"src": "n2", "dest": "n1","body": {"msg_id": 1,"type": "_~*hydromael*~_gossip_out_~*hydromael*~_gossip_in", "text": "10"}}
// {"src": "n2", "dest": "n1","body": {"msg_id": 1,"type": "broadcast", "message": 0}}
// {"src": "c1", "dest": "n1","body": {"msg_id": 1,"type": "read"}}
