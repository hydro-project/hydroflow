use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::future::ready;
use std::marker::PhantomData;
use std::pin::Pin;
use std::process::Output;
use std::task::{Context, Poll};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tracing::trace;
use hydroflow::futures::{Sink, sink, SinkExt, StreamExt};
use hydroflow::futures::task::noop_waker;
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::tokio_stream::{Stream};
use hydroflow::tokio_stream::wrappers::UnboundedReceiverStream;
use hydroflow::util::{collect_ready, collect_ready_async};

pub type Hostname = String;
type InterfaceName = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address {
    host: Hostname,
    interface: InterfaceName,
}

impl Address {
    fn new(host: Hostname, interface: InterfaceName) -> Self {
        Address {
            host,
            interface,
        }
    }
}

struct InputPort {
    type_id: TypeId,
    sender: Box<dyn MessageSender>,
}

trait MessageSender {
    fn send(&self, message: (Box<dyn Any>, Address));
}

impl<T: 'static> MessageSender for UnboundedSender<(T, Address)> {
    fn send(&self, message: (Box<dyn Any>, Address)) {
        if let Ok(msg) = message.0.downcast::<T>() {
            self.send((*msg, message.1)).unwrap();
        } else {
            panic!("Type mismatch");
        }
    }
}

struct OutputPort {
    type_id: TypeId,
    receiver: Pin<Box<dyn Stream<Item=(Box<dyn Any>, Address)>>>,
}


struct Host
{
    name: Hostname,
    process: Hydroflow<'static>,
    inputs: HashMap<InterfaceName, InputPort>,
    output: HashMap<InterfaceName, OutputPort>,
}

impl Host {
    fn run_tick(&mut self) -> bool {
        self.process.run_tick()
    }
}

struct HostBuilder {
    name: Hostname,
    process: Option<Hydroflow<'static>>,
    inputs: HashMap<InterfaceName, InputPort>,
    outputs: HashMap<InterfaceName, OutputPort>,
}

struct ProcessBuilderContext<'context> {
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
    fn new_inbox<T: 'static>(&mut self, interface: InterfaceName) -> UnboundedReceiverStream<(T, Address)> {
        let (sender, receiver) = hydroflow::util::unbounded_channel::<(T, Address)>();
        let type_id = TypeId::of::<T>();
        self.inputs.insert(interface, InputPort {
            type_id,
            sender: Box::new(sender),
        });
        receiver
    }
    fn new_outbox<T: 'static>(&mut self, interface: InterfaceName) -> impl Sink<(T, Address), Error=Infallible> {
        let (sender, receiver) = hydroflow::util::unbounded_channel::<(T, Address)>();
        let type_id = TypeId::of::<T>();

        let receiver = receiver.map(|(msg, addr)| {
            (Box::new(msg) as Box<dyn Any>, addr)
        });

        self.outputs.insert(interface, OutputPort {
            type_id,
            receiver: Box::pin(receiver),
        });

        sink_from_fn(move |message: (T, Address)| { sender.send((message.0, message.1)).unwrap() })
    }
}

impl HostBuilder {
    fn new(name: Hostname) -> Self {
        HostBuilder {
            name,
            process: None,
            inputs: Default::default(),
            outputs: Default::default(),
        }
    }

    fn with_process<F>(mut self, process_builder: F) -> Self
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

    fn build(self) -> Host {
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

struct Fleet
{
    hosts: HashMap<String, Host>,
}

impl Fleet {
    fn new() -> Self {
        Fleet {
            hosts: HashMap::new(),
        }
    }

    fn add_host<F>(&mut self, name: String, process_builder: F) -> &Host
    where
        F: FnOnce(&mut ProcessBuilderContext) -> Hydroflow<'static>,
    {
        let host = HostBuilder::new(name.clone())
            .with_process(process_builder)
            .build();
        self.hosts.insert(name.clone(), host);
        self.get_host(&name).unwrap()
    }

    fn get_host(&self, name: &str) -> Option<&Host> {
        self.hosts.get(name)
    }

    fn get_host_mut(&mut self, name: &str) -> Option<&mut Host>
    {
        self.hosts.get_mut(name)
    }

    fn run_tick_all(&mut self) {
        for (name, host) in self.hosts.iter_mut() {
            trace!("Running tick for host: {}", name);
            host.run_tick();
        }
    }

    async fn run_until_quiescent(&mut self) {
        loop {
            let mut quiescent = true;

            let mut all_messages: Vec<(Address, (Box<dyn Any>, Address))> = Vec::new();

            for (name, host) in self.hosts.iter_mut() {
                trace!("Running tick for host: {}", name);
                if host.run_tick() {
                    quiescent = false;

                    for (interface, mut output) in host.output.iter_mut() {
                        let src_address = Address::new(name.clone(), interface.clone());
                        let all_messages_on_interface: Vec<_> = collect_ready_async(&mut output.receiver).await;
                        for message_on_interface in all_messages_on_interface {
                            all_messages.push((src_address.clone(), message_on_interface));
                        }
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

            if quiescent {
                trace!("System quiescent, stopping.");
                break;
            } else {
                trace!("System not quiescent, running another set of ticks.");
            }
        }
    }
}

mod tests {
    use rand::{Rng, thread_rng};
    use rand::distributions::Alphanumeric;
    use hydroflow::futures::StreamExt;
    use hydroflow::hydroflow_syntax;
    use crate::server::server;
    use crate::simulation::{Address, Fleet, Hostname};


    fn generate_random_string(length: usize) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .filter(|c| c.is_ascii_alphabetic())
            .take(length)
            .map(char::from)
            .collect()
    }

    #[hydroflow::test]
    async fn echo_test() {
        let mut fleet = Fleet::new();

        let server: Hostname = "server".to_string();
        let client: Hostname = "client".to_string();
        let chat_interface: String = "chat".to_string();

        let server_address = Address::new(server.clone(), chat_interface.clone());
        let client_address = Address::new(client.clone(), chat_interface.clone());


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
                    -> for_each(|(msg, addr)| client_response_tx.send(msg).unwrap());

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