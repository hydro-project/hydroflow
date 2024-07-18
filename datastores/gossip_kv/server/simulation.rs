use std::any::{Any};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::future::ready;
use std::pin::Pin;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedSender};
use tracing::trace;
use hydroflow::futures::{Sink, sink, SinkExt, StreamExt};
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::tokio_stream::{Stream};
use hydroflow::tokio_stream::wrappers::UnboundedReceiverStream;
use hydroflow::util::{collect_ready_async};

pub type Hostname = String;
type InterfaceName = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    host: Hostname,
    interface: InterfaceName,
}

impl Address {
    pub fn new(host: Hostname, interface: InterfaceName) -> Self {
        Address {
            host,
            interface,
        }
    }
}

pub struct InputPort {
    sender: Box<dyn MessageSender>,
}

pub trait MessageSender {
    fn send(&self, message: (Box<dyn Any>, Address));
}

impl<T: 'static> MessageSender for UnboundedSender<(T, Address)> {
    fn send(&self, message: (Box<dyn Any>, Address)) {
        match message.0.downcast::<T>() {
            Ok(msg) => {
                self.send((*msg, message.1)).unwrap();
            }
            Err(e) => {
                panic!("Failed to downcast message to expected type: {:?}", e);
            }
        }
    }
}

pub struct OutputPort {
    receiver: Pin<Box<dyn Stream<Item=(Box<dyn Any>, Address)>>>,
}


pub struct Host
{
    name: Hostname,
    process: Hydroflow<'static>,
    inputs: HashMap<InterfaceName, InputPort>,
    output: HashMap<InterfaceName, OutputPort>,
}

impl Host {
    pub fn run_tick(&mut self) -> bool {
        self.process.run_tick()
    }
}

pub struct HostBuilder {
    name: Hostname,
    process: Option<Hydroflow<'static>>,
    inputs: HashMap<InterfaceName, InputPort>,
    outputs: HashMap<InterfaceName, OutputPort>,
}

pub struct ProcessBuilderContext<'context> {
    inputs: &'context mut HashMap<InterfaceName, InputPort>,
    outputs: &'context mut HashMap<InterfaceName, OutputPort>,
}

fn sink_from_fn<T>(mut f: impl FnMut(T)) -> impl Sink<T, Error=Infallible> {
    sink::drain().with(move |item| {
        (f)(item);
        ready(Result::<(), Infallible>::Ok(()))
    })
}

impl<'context> ProcessBuilderContext<'context> {
    pub fn new_inbox<T: 'static>(&mut self, interface: InterfaceName) -> UnboundedReceiverStream<(T, Address)> {
        let (sender, receiver) = hydroflow::util::unbounded_channel::<(T, Address)>();
        self.inputs.insert(interface, InputPort {
            sender: Box::new(sender),
        });
        receiver
    }
    pub fn new_outbox<T: 'static>(&mut self, interface: InterfaceName) -> impl Sink<(T, Address), Error=Infallible> {
        let (sender, receiver) = hydroflow::util::unbounded_channel::<(T, Address)>();

        let receiver = receiver.map(|(msg, addr)| {
            (Box::new(msg) as Box<dyn Any>, addr)
        });

        self.outputs.insert(interface, OutputPort {
            receiver: Box::pin(receiver),
        });

        sink_from_fn(move |message: (T, Address)| { sender.send((message.0, message.1)).unwrap() })
    }
}

impl HostBuilder {
    pub fn new(name: Hostname) -> Self {
        HostBuilder {
            name,
            process: None,
            inputs: Default::default(),
            outputs: Default::default(),
        }
    }

    pub fn with_process<F>(mut self, process_builder: F) -> Self
    where
        F: FnOnce(&mut ProcessBuilderContext) -> Hydroflow<'static>,
    {
        let mut context = ProcessBuilderContext {
            inputs: &mut self.inputs,
            outputs: &mut self.outputs,
        };
        let process = process_builder(&mut context);
        self.process = Some(process);
        self
    }

    pub fn build(self) -> Host {
        if self.process.is_none() {
            panic!("Process is required to build a host");
        }

        Host {
            name: self.name,
            process: self.process.unwrap(),
            inputs: self.inputs,
            output: self.outputs,
        }
    }
}

pub struct Fleet
{
    hosts: HashMap<String, Host>,
}

impl Fleet {
    pub fn new() -> Self {
        Fleet {
            hosts: HashMap::new(),
        }
    }

    pub fn add_host<F>(&mut self, name: String, process_builder: F) -> &Host
    where
        F: FnOnce(&mut ProcessBuilderContext) -> Hydroflow<'static>,
    {
        let host = HostBuilder::new(name.clone())
            .with_process(process_builder)
            .build();
        assert!(self.hosts.insert(host.name.clone(), host).is_none(), "Host with name {} already exists", name);
        self.get_host(&name).unwrap()
    }

    pub fn get_host(&self, name: &str) -> Option<&Host> {
        self.hosts.get(name)
    }

    pub fn get_host_mut(&mut self, name: &str) -> Option<&mut Host>
    {
        self.hosts.get_mut(name)
    }

    pub async fn run_single_tick_all_hosts(&mut self) -> bool {
        let mut work_done: bool = false;

        for (name, host) in self.hosts.iter_mut() {
            trace!("Running tick for host: {}", name);
            work_done |= host.run_tick();
        }

        self.process_network().await;

        work_done
    }

    pub async fn process_network(&mut self) {
        let mut all_messages: Vec<(Address, (Box<dyn Any>, Address))> = Vec::new();

        for (name, host) in self.hosts.iter_mut() {
            for (interface, output) in host.output.iter_mut() {
                let src_address = Address::new(name.clone(), interface.clone());
                let all_messages_on_interface: Vec<_> = collect_ready_async(&mut output.receiver).await;
                for message_on_interface in all_messages_on_interface {
                    all_messages.push((src_address.clone(), message_on_interface));
                }
            }
        }

        for (src_address, (msg, addr)) in all_messages {
            if let Some(destination_host) = self.hosts.get(&addr.host) {
                if let Some(input) = destination_host.inputs.get(&addr.interface) {
                    input.sender.send((msg, src_address.clone()));
                } else {
                    trace!("No interface named {:?} found on host {:?}. Dropping message {:?}.", addr.interface, addr.host, msg);
                }
            } else {
                trace!("No host named {:?} found. Dropping message {:?}.", addr.host, msg);
            }
        }
    }

    pub async fn run_until_quiescent(&mut self) {
        while self.run_single_tick_all_hosts().await {}
    }
}

mod tests {
    use hydroflow::futures::StreamExt;
    use hydroflow::hydroflow_syntax;
    use crate::simulation::{Address, Fleet, Hostname};

    #[hydroflow::test]
    async fn echo_test() {
        let mut fleet = Fleet::new();

        let server: Hostname = "server".to_string();
        let client: Hostname = "client".to_string();
        let chat_interface: String = "chat".to_string();

        let server_address = Address::new(server.clone(), chat_interface.clone());

        // Create the echo server
        fleet.add_host(server.clone(), |ctx| {
            let network_input = ctx.new_inbox::<String>(chat_interface.clone());
            let network_output = ctx.new_outbox::<String>(chat_interface.clone());
            hydroflow_syntax! {
                out = dest_sink(network_output);

                source_stream(network_input)
                    -> inspect(|(msg, addr)| println!("Received {:?} from {:?}", msg, addr))
                    -> map(|(msg, addr)| (msg, addr))
                    -> out;
            }
        });

        let (client_trigger_tx, client_trigger_rx) = hydroflow::util::unbounded_channel::<String>();
        let (client_response_tx, mut client_response_rx) = hydroflow::util::unbounded_channel::<String>();

        fleet.add_host(client.clone(), |ctx| {
            let network_out = ctx.new_outbox::<String>(chat_interface.clone());
            let network_in = ctx.new_inbox::<String>(chat_interface.clone());

            hydroflow_syntax! {
                out = dest_sink(network_out);

                source_stream(client_trigger_rx)
                    -> map(|msg| (msg, server_address.clone()))
                    -> out;

                source_stream(network_in)
                    -> inspect(|(msg, addr)| println!("Received {:?} from {:?}", msg, addr))
                    -> for_each(|(msg, _addr)| client_response_tx.send(msg).unwrap());

            }
        });

        // Send a message.
        client_trigger_tx.send("Hello, world!".to_string()).unwrap();

        fleet.run_until_quiescent().await;

        // Check that the message was received.
        let response = client_response_rx.next().await.unwrap();
        assert_eq!(response, "Hello, world!");
    }
}