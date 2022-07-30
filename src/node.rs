use crate::failure_detector::FailureDetector;
use crate::reliable_delivery::ReliableDelivery;
use crate::types::{ComponentChannels, ComponentTypes, Message, MessageType, NetComponent};
use crate::uniform_reliable_delivery::UniformReliableDelivery;
use colored::Colorize;
use flume::Sender;
use local_ip_address::local_ip;
use pub_sub::PubSub;
use serde_json;
extern crate pub_sub;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::net::TcpListener;
use std::net::{self, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

/// Acts as a mediator between Components
/// All communications is done via channels
/// Node will automatically choose the right component to do the right Task in Runtime
/// Extern Messages are delivered to componnents via flume channels and internal Messages are delivered via pub_sub
pub struct Node {
    name: String,
    pub addr: String,
    peers: Vec<String>,
    peers_threashold: usize,
    listner: TcpListener,
    ready: bool,
    pub message_broker: Arc<Mutex<HashMap<MessageType, Vec<Sender<Message>>>>>,
    pub publishers: HashMap<ComponentTypes, PubSub<Message>>, // used only for cloning to newer components
    pub components: HashMap<ComponentTypes, Option<Box<dyn NetComponent>>>,
    pub user_callbacks: Arc<Mutex<HashMap<MessageType, Vec<Box<dyn Fn(&Message) + Send>>>>>,
}

impl Node {
    pub fn new(name: String, port: usize, peers_threashold: usize) -> Node {
        let addr = format!("{}:{}", local_ip().unwrap().to_string(), port,);
        println!("{}: addr is at :{}", "FROM NODE".blue(), addr);

        return Node {
            name,
            addr: addr.clone(),
            peers: Vec::with_capacity(peers_threashold),
            peers_threashold,
            listner: net::TcpListener::bind(addr.to_string()).unwrap(),
            ready: false,
            message_broker: Arc::new(Mutex::new(HashMap::new())),
            publishers: HashMap::with_capacity(peers_threashold),
            components: HashMap::new(),
            user_callbacks: Arc::new(Mutex::new(HashMap::new())),
        };
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        for component in self.components.iter_mut() {
            let cfg = component.1;
            match cfg.borrow_mut() {
                Some(cmp) => {
                    cmp.start();
                }
                None => {}
            }
        }
        let listner = self.listner.try_clone().expect("Failled cloning Listner");
        let addr = self.addr.clone();
        let user_callbacks = self.user_callbacks.clone();
        let broker = self.message_broker.clone();
        let handle = thread::spawn(move || {
            handle_requests(listner, addr, user_callbacks, broker);
        });
        self.ready = true;
        return handle;
    }

    pub fn add_component(
        &mut self,
        mut component: Box<dyn NetComponent>,
        cmp_type: ComponentTypes,
        messages: Vec<MessageType>,
        target_componets: Vec<ComponentTypes>,
    ) {
        let (_tx, _rc) = flume::unbounded();
        let pub_sub_channel: PubSub<Message> = pub_sub::PubSub::new();
        self.publishers
            .insert(cmp_type.clone(), pub_sub_channel.clone());
        let mut component_channels = ComponentChannels {
            subscriptions: Vec::new(),
            rc: _rc.clone(),
            publisher: pub_sub_channel.clone(),
        };

        let mut message_broker = self.message_broker.lock().unwrap();
        for msg_type in messages.iter() {
            message_broker.entry(msg_type.clone()).or_insert(Vec::new());
            message_broker.get_mut(msg_type).unwrap().push(_tx.clone());
        }
        // target components for which you want to subscribe messages to
        for target_cmp in target_componets.iter() {
            component_channels
                .subscriptions
                .push(self.publishers.get(target_cmp).unwrap().subscribe());
        }
        component.add_component_channels(component_channels);
        self.components.insert(cmp_type, Some(component));
    }

    pub fn has_failure_detector(&mut self, timeout: Duration, delay: Duration) -> &mut Node {
        let mut peers = Vec::new();
        for peer in self.peers.iter() {
            if *peer != self.addr {
                peers.push(peer.clone());
            }
        }
        let fd = FailureDetector::new(
            self.peers_threashold,
            timeout,
            delay,
            self.addr.clone(),
            peers,
        );
        let messages = vec![MessageType::HeartBeat, MessageType::RequestHeartBeat];
        self.add_component(
            Box::new(fd),
            ComponentTypes::FaillureDetector,
            messages,
            Vec::new(),
        );
        return self;
    }

    pub fn has_reliable_delivery(&mut self) -> &mut Node {
        let rb = ReliableDelivery::new(self.peers.clone());
        if self
            .components
            .contains_key(&ComponentTypes::FaillureDetector)
        {
            let messages = vec![MessageType::ReliableDelivery];
            let target_components = vec![ComponentTypes::FaillureDetector];
            self.add_component(
                Box::new(rb),
                ComponentTypes::ReliableDelivery,
                messages,
                target_components,
            );
            return self;
        } else {
            panic!("Reliable delivery requires Faillure detector consider adding has_faillure_detector method");
        }
    }

    pub fn has_uniform_reliable_delivery(&mut self) -> &mut Node {
        let urb = UniformReliableDelivery::new(self.addr.clone(), self.peers.clone());
        if self
            .components
            .contains_key(&ComponentTypes::FaillureDetector)
        {
            let messages = vec![
                MessageType::UniformReliableDelivery,
                MessageType::AckDelivery,
            ];
            let target_components = vec![ComponentTypes::FaillureDetector];
            self.add_component(
                Box::new(urb),
                ComponentTypes::UniformReliableDelivery,
                messages,
                target_components,
            );
            return self;
        } else {
            panic!("Reliable delivery requires Faillure detector consider adding has_faillure_detector method");
        }
    }

    pub fn send(&self, value: &Message, addr: String) {
        println!("sending request");
        let stream = TcpStream::connect(addr).expect("From Node: error connecting to Peer");
        serde_json::to_writer(stream, value).expect("From Node:failed to push value into stream");
        println!("From Client: Message sent");
    }
    pub fn add_peer(&mut self, peer: String) -> &mut Node {
        self.peers.push(peer);
        return self;
    }

    pub fn add_peers(&mut self, peers: Vec<String>) -> &mut Node {
        self.peers = peers;
        return self;
    }

    pub fn broadcast(&mut self, value: &Message) {
        for peer in self.peers.iter() {
            let result = TcpStream::connect(peer.clone());
            match result {
                Ok(stream) => {
                    serde_json::to_writer(stream, value)
                        .expect("FROM Server:failed to push value into stream");
                }
                Err(e) => {
                    println!("Failled Brodcasting Message");
                    println!("{}", e);
                }
            }
        }
    }

    pub fn on_receive_message(
        &mut self,
        msg: MessageType,
        callback: Box<dyn Fn(&Message) + Send>,
    ) -> &mut Node {
        self.user_callbacks
            .lock()
            .unwrap()
            .entry(msg.clone())
            .or_insert(Vec::new());
        self.user_callbacks
            .lock()
            .unwrap()
            .get_mut(&msg)
            .unwrap()
            .push(callback);
        return self;
    }
}
//deserilize the message and treat each case diffrently
pub fn handle_connection(stream: net::TcpStream) -> Message {
    let result: Message =
        serde_json::from_reader(stream).expect("FROM Node: Failed deserializing the Message");
    return result;
}

pub fn handle_requests(
    listner: TcpListener,
    source: String,
    _callbacks: Arc<Mutex<HashMap<MessageType, Vec<Box<dyn Fn(&Message) + Send>>>>>,
    message_broker: Arc<Mutex<HashMap<MessageType, Vec<Sender<Message>>>>>,
) {
    println!(
        "{}: server started succesfully and ready to receive peer requests",
        "FROM NODE".blue()
    );

    for stream in listner.incoming() {
        let stream = stream.expect("From Server:failed connecting to client request");
        //let client_addr = stream.local_addr().unwrap().to_string(); // used for authentification
        //match filter_request(client_addr)
        //pass the handle_connection function to the thread pool
        // need to optimize this
        let user_callbacks = _callbacks.clone();
        let broker = message_broker.clone();
        thread::spawn(move || {
            let msg = handle_connection(stream);
            println!(
                "{}:Received {:?} from node:{}",
                "From Node".blue(),
                msg.message_type,
                msg.sender
            );
            if user_callbacks
                .lock()
                .unwrap()
                .contains_key(&msg.message_type)
            {
                for callback in user_callbacks
                    .lock()
                    .unwrap()
                    .get(&msg.message_type)
                    .unwrap()
                    .iter()
                {
                    callback(&msg);
                }
            }
            for component in broker
                .lock()
                .unwrap()
                .get(&msg.message_type)
                .unwrap()
                .iter()
            {
                component
                    .send(msg.clone())
                    .expect("failled forwarding message to apporpriate component");
            }
        });
    }
}
