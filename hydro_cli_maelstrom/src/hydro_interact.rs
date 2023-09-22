use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;

use bytes::Bytes;
use futures::SinkExt;
use serde_json::Value;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::cli_refs::ServerPort;
use crate::config::{CustomPort, MaelstromPort, Port};
use crate::maelstrom::{CustomBody, HydroBody, Message, UnknownBody};
use crate::ports;

pub enum SourceConnection {
    Direct(FramedWrite<TcpStream, LengthDelimitedCodec>),
    Tagged(HashMap<usize, FramedWrite<TcpStream, LengthDelimitedCodec>>),
}

impl SourceConnection {
    fn is_tagged(&self) -> bool {
        matches!(self, Self::Tagged(_))
    }

    fn get_mut(
        &mut self,
        node_id: &usize,
    ) -> Option<&mut FramedWrite<TcpStream, LengthDelimitedCodec>> {
        match self {
            Self::Direct(writer) => Some(writer),
            Self::Tagged(map) => map.get_mut(node_id),
        }
    }

    async fn try_from(port: ServerPort) -> Result<Self, Box<dyn Error>> {
        match port {
            ServerPort::TcpPort(socket) => connect_to_socket(&socket).await.map(Self::Direct),
            ServerPort::Merge(merge) => connect_to_merge(&merge).await.map(Self::Tagged),
            _ => Err("Attempted to connect to invalid port type".into()),
        }
    }
}

async fn connect_to_socket(
    socket: &SocketAddr,
) -> Result<FramedWrite<TcpStream, LengthDelimitedCodec>, Box<dyn Error>> {
    let stream = TcpStream::connect(socket).await?;
    let codec = LengthDelimitedCodec::new();
    Ok(FramedWrite::new(stream, codec))
}

async fn connect_to_merge(
    merge: &Vec<ServerPort>,
) -> Result<HashMap<usize, FramedWrite<TcpStream, LengthDelimitedCodec>>, Box<dyn Error>> {
    let mut map = HashMap::new();
    for tagged in merge {
        let (tcpport, node_id) = match tagged {
            ServerPort::Tagged(tcpport, node_id) => (tcpport, node_id),
            _ => return Err("ServerPort in merge was not tagged".into()),
        };
        let socket = match **tcpport {
            ServerPort::TcpPort(socket) => socket,
            _ => return Err("ServerPort in tagged was not tcp port".into()),
        };

        let writer = connect_to_socket(&socket).await?;

        map.insert(*node_id as usize, writer);
    }

    Ok(map)
}

// Connects to all source ports, returning a mapping from maelstrom type to the corresponding connection
pub async fn connect_to_sources<'a, I: IntoIterator<Item = &'a Port>>(
    ready_message: &str,
    ports: I,
) -> Result<HashMap<String, SourceConnection>, Box<dyn Error>> {
    let mut source_name_to_type = ports::source_name_to_type(ports);

    // Parse the ready message into the map from port name to ServerPort for all source ports
    let ready_message = ready_message.trim_start_matches("ready: ");
    let source_ports = serde_json::from_str::<HashMap<String, ServerPort>>(ready_message)?;
    #[cfg(debug_assertions)]
    println!("Parsed ready: {}", serde_json::to_string(&source_ports)?);

    // Connect to all source ports
    let mut connections = HashMap::new();
    for (port_name, port) in source_ports {
        let connection = SourceConnection::try_from(port).await?;
        let maelstrom_type = source_name_to_type
            .remove(&port_name)
            .ok_or("Port name mapping missing from ready message")?;
        connections.insert(maelstrom_type, connection);
    }

    Ok(connections)
}

/// Demuxes standard in to the corresponding connections
pub fn spawn_input_handler(
    connections: HashMap<String, SourceConnection>,
    node_names: &[String],
) -> JoinHandle<()> {
    let name_to_id = node_names
        .iter()
        .enumerate()
        .map(|(id, name)| (name.to_string(), id))
        .collect::<HashMap<_, _>>();

    tokio::task::spawn(input_handler(connections, name_to_id))
}

/// Demux stdin to the relevant source ports in the hydroflow program
async fn input_handler(
    mut connections: HashMap<String, SourceConnection>,
    name_to_id: HashMap<String, usize>,
) {
    let mut lines = BufReader::new(stdin()).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        // Parse the initial message structure
        let message = serde_json::from_str::<Message<Value>>(&line).unwrap();
        let maelstrom_type = serde_json::from_value::<UnknownBody>(message.body.clone())
            .unwrap()
            .maelstrom_type;

        // Find the connection corresponding to the maelstrom_type and source
        let connection = connections.get_mut(&maelstrom_type).unwrap();
        let is_custom = connection.is_tagged();
        let node_id = name_to_id.get(&message.src).unwrap_or(&0);
        let target_port = connection.get_mut(node_id).unwrap();

        let body_string = if is_custom {
            // If custom port, simply forward the inner text field
            serde_json::from_value::<CustomBody>(message.body)
                .unwrap()
                .text
        } else {
            // If maelstrom port, remove the type field and update the msg_id so responses can be directed correctly
            let mut body_value = message.body;
            let body = body_value.as_object_mut().unwrap();
            body.remove("type").unwrap();

            // Update the msg_id to be the pair [src, msg_id]
            body.entry("msg_id").and_modify(|msg_id| {
                *msg_id = Value::Array(vec![message.src.into(), msg_id.clone()])
            });

            serde_json::to_string(body).unwrap()
        };

        #[cfg(debug_assertions)]
        println!(
            "Sending line {} to {}.{}",
            body_string, &maelstrom_type, node_id
        );
        target_port.send(Bytes::from(body_string)).await.unwrap();
    }
}

pub enum SinkConnection {
    Direct(DirectSink),
    Demux(DemuxSink),
}

pub struct DirectSink {
    port_name: String,
    binding: ServerPort,
    listener: TcpListener,
    maelstrom_type: String,
}

pub struct DemuxSink {
    port_name: String,
    binding: ServerPort,
    listeners: Vec<TcpListener>,
    maelstrom_type: String,
}

impl SinkConnection {
    fn spawn_handlers(
        self,
        node_id: usize,
        node_names: &[String],
        output_id: usize,
        output_count: usize,
    ) {
        let node_name = node_names[node_id].clone();
        match self {
            Self::Direct(direct) => direct.spawn_handler(node_name, output_id, output_count),
            Self::Demux(demux) => demux.spawn_handlers(node_name, node_names),
        };
    }

    pub fn defn(&self) -> (&str, &ServerPort) {
        match self {
            Self::Direct(direct) => direct.defn(),
            Self::Demux(demux) => demux.defn(),
        }
    }
}

impl DirectSink {
    pub async fn bind(port: &MaelstromPort) -> Result<Self, Box<dyn Error>> {
        let port_name = port.port_name().to_string();
        let maelstrom_type = port.maelstrom_type().to_string();

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let binding = ServerPort::TcpPort(addr);

        Ok(Self {
            port_name,
            binding,
            listener,
            maelstrom_type,
        })
    }

    fn spawn_handler(self, node_name: String, output_id: usize, output_count: usize) {
        tokio::task::spawn(output_handler(
            self.listener,
            self.maelstrom_type,
            node_name,
            output_id,
            output_count,
        ));
    }

    fn defn(&self) -> (&str, &ServerPort) {
        (&self.port_name, &self.binding)
    }
}

impl DemuxSink {
    pub async fn bind(port: &CustomPort, node_count: usize) -> Result<Self, Box<dyn Error>> {
        let port_name = port.demux_port_name().to_string();
        let maelstrom_type = port.maelstrom_type().to_string();

        let mut connections = HashMap::new();
        let mut listeners = Vec::new();
        for dest_id in 0..node_count {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let addr = listener.local_addr()?;
            let tcpport = ServerPort::TcpPort(addr);
            connections.insert(dest_id as u32, tcpport);
            listeners.push(listener);
        }
        let binding = ServerPort::Demux(connections);

        Ok(Self {
            port_name,
            binding,
            listeners,
            maelstrom_type,
        })
    }

    fn spawn_handlers(self, node_name: String, node_names: &[String]) {
        self.listeners
            .into_iter()
            .enumerate()
            .for_each(|(dest_id, listener)| {
                let maelstrom_type = self.maelstrom_type.clone();
                let dest = node_names[dest_id].clone();
                let node_name = node_name.clone();
                tokio::task::spawn(custom_output_handler(
                    listener,
                    maelstrom_type,
                    node_name,
                    dest,
                ));
            });
    }

    fn defn(&self) -> (&str, &ServerPort) {
        (&self.port_name, &self.binding)
    }
}

pub fn spawn_output_handlers(
    sink_connections: Vec<SinkConnection>,
    node_id: usize,
    node_names: &[String],
) -> Result<(), Box<dyn Error>> {
    let output_count = sink_connections.len();
    for (output_id, connection) in sink_connections.into_iter().enumerate() {
        connection.spawn_handlers(node_id, node_names, output_id, output_count);
    }

    Ok(())
}

/// Accept a connection on each sink port which wraps outputs in maelstrom payload of specified type
/// Generated "msg_id"s will be `= output_id (mod output_count)` to ensure no overlap
async fn output_handler(
    listener: TcpListener,
    maelstrom_type: String,
    node_name: String,
    output_id: usize,
    output_count: usize,
) {
    let in_stream = listener.accept().await.unwrap().0;

    let mut lines = FramedRead::new(in_stream, LengthDelimitedCodec::new());
    #[cfg(debug_assertions)]
    println!("accepted connection for {}", maelstrom_type);

    // Initialize counter which tracks the next available msg_id
    let mut msg_id_counter = output_id;

    while let Some(Ok(line)) = lines.next().await {
        // Transforms output into maelstrom payload
        // For example:
        // {"echo":"hello world!","in_reply_to":["n1", 1]}
        // ->
        // {"src":"n1","dest":"c1","body":{"echo":"hello world!","msg_id":0,"in_reply_to":1,"type":"echo_ok"}}

        // Parse line to string
        let raw_line: String = bincode::deserialize(&line).unwrap();

        // Read maelstrom specific hydro content
        let hydro_body = serde_json::from_str::<HydroBody>(&raw_line).unwrap();
        let (dest, in_reply_to) = hydro_body.in_reply_to;

        // Parse body as a raw json object
        let mut raw_body = serde_json::from_str::<HashMap<String, Value>>(&raw_line).unwrap();

        // Insert in the maelstrom specific fields
        raw_body.insert("in_reply_to".into(), in_reply_to);
        raw_body.insert("msg_id".into(), msg_id_counter.into());
        raw_body.insert("type".into(), maelstrom_type.clone().into());
        msg_id_counter += output_count;

        // Wrap in maelstrom payload
        let message = Message {
            src: node_name.clone(),
            dest,
            body: raw_body,
        };

        // Send the message to maelstrom
        println!("{}", serde_json::to_string(&message).unwrap());
    }
}

/// Handles forwarding messages from custom ports to maelstrom
async fn custom_output_handler(
    listener: TcpListener,
    maelstrom_type: String,
    node_name: String,
    dest: String,
) {
    let in_stream = listener.accept().await.unwrap().0;

    let mut lines = FramedRead::new(in_stream, LengthDelimitedCodec::new());
    #[cfg(debug_assertions)]
    println!("accepted custom connection for {}.{}", maelstrom_type, dest);

    while let Some(Ok(line)) = lines.next().await {
        // Parse line to string
        let line = bincode::deserialize(&line).unwrap();

        // Wrap in maelstrom payload for custom packets
        let message = Message {
            src: node_name.clone(),
            dest: dest.clone(),
            body: CustomBody {
                maelstrom_type: maelstrom_type.clone(),
                text: line,
            },
        };

        // Send the message to maelstrom
        println!("{}", serde_json::to_string(&message).unwrap());
    }
}
