//# Uniform Reliable Delivery
//! gurantees reliability and also considers the behaviour of failed nodes,
//! reliable Delivery is faster but it doesnt gurantee if all nodes delivered the message including Failled nodes
use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use colored::Colorize;
use flume::Receiver;

use crate::{
    types::{ComponentChannels, NetComponent},
    Message, MessageType,
};

pub struct PendingMessage {
    pub msg: Message,
    pub receipts: HashMap<String, HashMap<i64, Message>>,
    pub validation_list: HashMap<String, bool>,
    pub validated: bool,
}
impl PendingMessage {
    pub fn new(peers: Vec<String>, msg: Message) -> PendingMessage {
        let mut receipt_map: HashMap<String, HashMap<i64, Message>> = HashMap::new();
        let mut validation_map: HashMap<String, bool> = HashMap::new();
        for peer in peers.iter() {
            receipt_map.insert(peer.clone(), HashMap::new());
            validation_map.insert(peer.clone(), false);
        }
        return PendingMessage {
            msg,
            receipts: receipt_map,
            validation_list: validation_map,
            validated: false,
        };
    }
}

pub struct UniformReliableDelivery {
    addr: String,
    peers: Arc<Mutex<HashMap<String, bool>>>,
    pub component_channels: Option<ComponentChannels>,
    pub delivered_messages: Arc<Mutex<HashMap<String, HashMap<i64, Message>>>>,
    pub pending: Arc<Mutex<HashMap<String, HashMap<i64, PendingMessage>>>>,
}
impl UniformReliableDelivery {
    pub fn new(addr: String, target: Vec<String>) -> UniformReliableDelivery {
        let map: Arc<Mutex<HashMap<String, bool>>> =
            Arc::new(Mutex::new(HashMap::with_capacity(target.len())));
        let map2: Arc<Mutex<HashMap<String, HashMap<i64, Message>>>> =
            Arc::new(Mutex::new(HashMap::with_capacity(target.len())));
        let map3: Arc<Mutex<HashMap<String, HashMap<i64, PendingMessage>>>> =
            Arc::new(Mutex::new(HashMap::with_capacity(target.len())));
        for peer in &target {
            map.lock().unwrap().insert(peer.clone(), true);
            map2.lock().unwrap().insert(peer.clone(), HashMap::new());
            map3.lock().unwrap().insert(peer.clone(), HashMap::new());
        }
        UniformReliableDelivery {
            addr,
            peers: map,
            component_channels: None,
            delivered_messages: map2,
            pending: map3,
        }
    }
}
impl NetComponent for UniformReliableDelivery {
    /// after detecting node faillure redistribute every message of that node to guarantee reliability
    // could be optimized to make only naighbouring peers have the respnsability to redistribute, like a DHT or selecting oly the last delivered_messages to be redistributed and not all
    fn start(&mut self) {
        let delivered_messages = self.delivered_messages.clone();
        let receiver = self.component_channels.as_mut().unwrap().rc.clone();
        let peers = self.peers.clone();
        let pending = self.pending.clone();
        let addr = self.addr.clone();
        thread::spawn(move || handle_requests(delivered_messages, pending, receiver, peers, addr));

        for subscirption in &self.component_channels.as_mut().unwrap().subscriptions {
            let sub = subscirption.clone();
            let peers = self.peers.clone();
            let delivered_messages = self.delivered_messages.clone();
            thread::spawn(move || loop {
                let msg = sub
                    .recv()
                    .expect("FROM UniformReliableDelivery: Failled unwrapping msg from pub_sub");
                match msg.message_type {
                    MessageType::FailledNode => {
                        println!("{}: received Failled Node Message and proceeding to redistribute its previous delivered_messages", "FROM URB".blue());
                        peers
                            .lock()
                            .expect("failled locking the peers map")
                            .insert(msg.source.clone(), false);
                        for map in delivered_messages
                            .lock()
                            .expect("failled locking node delivered_messages")
                            .get_mut(&msg.source.clone())
                            .iter_mut()
                        {
                            for message in map.iter_mut() {
                                for peer in peers.lock().unwrap().iter() {
                                    if *peer.1 == true {
                                        let stream = TcpStream::connect(peer.0).expect(
                                            "From UniformReliableDelivery: error connecting to Peer",
                                        );
                                        serde_json::to_writer(stream, message.1)
                                        .expect("From UniformReliableDelivery:failed to push value into stream");
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            });
        }
    }
    fn add_component_channels(&mut self, cmp: ComponentChannels) {
        self.component_channels = Some(cmp);
    }
}
pub fn handle_requests(
    delivered_messages: Arc<Mutex<HashMap<String, HashMap<i64, Message>>>>,
    pending_messages: Arc<Mutex<HashMap<String, HashMap<i64, PendingMessage>>>>,
    rc: Receiver<Message>,
    target: Arc<Mutex<HashMap<String, bool>>>,
    addr: String,
) {
    let mut peers = Vec::new();
    for peer in target.lock().unwrap().iter_mut() {
        peers.push(peer.0.clone());
    }
    loop {
        let msg = rc
            .recv()
            .expect("FROM UniformReliable Broadcast: Failled unwraping Message");
        match msg.message_type {
            MessageType::AckDelivery => {
                println!("{}: Received a Message to be delivered", "FROM URB".blue());
                let pending_message = PendingMessage::new(peers.clone(), msg.clone());
                let mut pending = pending_messages.lock().unwrap();
                let map_entry = pending.get_mut(&msg.source).unwrap();
                if map_entry.contains_key(&msg.sn) {
                    if map_entry.get(&msg.sn).unwrap().validated {
                        println!("{}:Message is already Delivered", "FROM URB".green());
                    } else {
                        if *map_entry
                            .get(&msg.sn)
                            .unwrap()
                            .validation_list
                            .get(&msg.sender)
                            .unwrap()
                            == true
                        {
                            println!(
                                "{}:Node {} already sent an ACKDelivery",
                                "FROM URB".blue(),
                                msg.sender
                            );
                        } else {
                            map_entry
                                .get_mut(&msg.sn)
                                .unwrap()
                                .validation_list
                                .insert(msg.sender.clone(), true);
                            let mut counter = 0;
                            for peer in map_entry.get(&msg.sn).unwrap().validation_list.iter() {
                                if *peer.1 == true {
                                    counter = counter + 1;
                                }
                            }
                            if counter == peers.len() {
                                println!("{}:all Peers received the Message", "FROM URB".green());
                                map_entry.get_mut(&msg.sn).unwrap().validated = true;
                                let delivered_message = Message {
                                    info: msg.info.clone(),
                                    message_type: MessageType::UniformReliableDelivery,
                                    sender: msg.sender.clone(),
                                    source: msg.source.clone(),
                                    sn: msg.sn,
                                };
                                // this will register the message with sender being as the last node who sent an ack
                                delivered_messages
                                    .lock()
                                    .unwrap()
                                    .get_mut(&msg.source)
                                    .unwrap()
                                    .insert(msg.sn, delivered_message);
                                println!("********************************************************************");
                                println!(
                                    "{}: Message with sn:{} from peer:{} delivered successfully",
                                    "FROM URB".green(),
                                    msg.sn,
                                    msg.source
                                );
                                println!("********************************************************************");
                            } else {
                                println!("{}:waiting for other peers Ack", "FROM URB".blue());
                            }
                        }
                    }
                } else {
                    map_entry.insert(msg.sn, pending_message);
                    map_entry
                        .get_mut(&msg.sn)
                        .unwrap()
                        .validation_list
                        .insert(msg.sender.clone(), true);
                }
            }
            MessageType::UniformReliableDelivery => {
                let pending_message = PendingMessage::new(peers.clone(), msg.clone());
                if delivered_messages
                    .lock()
                    .unwrap()
                    .get(&msg.source.clone())
                    .unwrap()
                    .contains_key(&msg.sn)
                {
                    println!(
                        "{} message with sn:{} from {} is already delivered",
                        "FROM URB".green(),
                        msg.sn,
                        msg.source
                    );
                } else {
                    if pending_messages
                        .lock()
                        .unwrap()
                        .get_mut(&msg.source)
                        .unwrap()
                        .contains_key(&msg.sn)
                        != true
                    {
                        pending_messages
                            .lock()
                            .unwrap()
                            .get_mut(&msg.source)
                            .unwrap()
                            .insert(msg.sn, pending_message);
                        pending_messages
                            .lock()
                            .unwrap()
                            .get_mut(&msg.source)
                            .unwrap()
                            .get_mut(&msg.sn)
                            .unwrap()
                            .validation_list
                            .insert(msg.source.clone(), true);
                    }
                }
                let ack_delivery = Message {
                    info: msg.info.clone(),
                    message_type: MessageType::AckDelivery,
                    sender: addr.clone(),
                    source: msg.source.clone(),
                    sn: msg.sn,
                };
                for peer in peers.iter() {
                    let socketaddr = SocketAddr::from_str(peer).unwrap();
                    let result =
                        TcpStream::connect_timeout(&socketaddr, Duration::from_millis(2000));
                    match result {
                        Ok(stream) => {
                            serde_json::to_writer(stream, &ack_delivery).expect(
                                "From UniformReliableDelivery:failed to push value into stream",
                            );
                        }
                        Err(_err) => {
                            println!("{},Failled sending AckDelivery to node {}, Node Could be Down, Check with FD", "FROM URB".red(),peer);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
