use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{Arc, Mutex},
    thread::{self},
};

use colored::Colorize;
use flume::Receiver;

use crate::{
    types::{
        ComponentChannels,
        MessageType::{self},
        NetComponent,
    },
    Message,
};

/// gurantees reliability independent of whether the sender is correct or not
/// it ensures all or none of the correct nodes gets the message
/// even if the sender crashes without giving it to everyone, every correct node that already received the message will play the role of the sender
/// Requires a Faillure detector
/// Use case: need to guarantee that a message is reached to all correct nodes even if if the sender crashes after sending to one correct node.
pub struct ReliableDelivery {
    peers: Arc<Mutex<HashMap<String, bool>>>,
    pub component_channels: Option<ComponentChannels>,
    pub delivered_messages: Arc<Mutex<HashMap<String, HashMap<i64, Message>>>>,
}

impl ReliableDelivery {
    pub fn new(target: Vec<String>) -> ReliableDelivery {
        let map: Arc<Mutex<HashMap<String, bool>>> =
            Arc::new(Mutex::new(HashMap::with_capacity(target.len())));
        let map2: Arc<Mutex<HashMap<String, HashMap<i64, Message>>>> =
            Arc::new(Mutex::new(HashMap::with_capacity(target.len())));
        for peer in &target {
            map.lock().unwrap().insert(peer.clone(), true);
            map2.lock().unwrap().insert(peer.clone(), HashMap::new());
        }
        ReliableDelivery {
            peers: map,
            component_channels: None,
            delivered_messages: map2,
        }
    }
}

impl NetComponent for ReliableDelivery {
    /// after detecting node faillure redistribute every message of that node to guarantee reliability
    // could be optimized to make only naighbouring peers have the respnsability to redistribute, like a DHT or selecting oly the last delivered_messages to be redistributed and not all
    fn start(&mut self) {
        let delivered_messages = self.delivered_messages.clone();
        let receiver = self.component_channels.as_mut().unwrap().rc.clone();
        thread::spawn(move || handle_requests(delivered_messages, receiver));

        for subscirption in &self.component_channels.as_mut().unwrap().subscriptions {
            let sub = subscirption.clone();
            let peers = self.peers.clone();
            let delivered_messages = self.delivered_messages.clone();
            thread::spawn(move || loop {
                let msg = sub
                    .recv()
                    .expect("FROM ReliableDelivery: Failled unwrapping msg from pub_sub");
                match msg.message_type {
                    MessageType::FailledNode => {
                        println!("{}: received Failled Node Message and proceeding to redistribute its previous delivered_messages", "FROM RB".blue());
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
                                            "From ReliableDelivery: error connecting to Peer",
                                        );
                                        serde_json::to_writer(stream, message.1)
                                        .expect("From ReliableDelivery:failed to push value into stream");
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

impl ReliableDelivery {
    pub fn send(&self, msg: &Message) {
        for peer in self.peers.lock().unwrap().iter_mut() {
            let stream = TcpStream::connect(peer.0)
                .expect("From BestEffortDelivery: error connecting to Peer");
            serde_json::to_writer(stream, &msg)
                .expect("From BestEffortDelivery:failed to push value into stream");
        }
    }
}

pub fn handle_requests(
    delivered_messages: Arc<Mutex<HashMap<String, HashMap<i64, Message>>>>,
    rc: Receiver<Message>,
) {
    loop {
        let msg = rc
            .recv()
            .expect("FROM Reliable Broadcast: Failled unwraping Message");
        match msg.message_type {
            MessageType::ReliableDelivery => {
                println!("{}: Received a Message to be delivered", "FROM RB".blue());
                if delivered_messages
                    .lock()
                    .unwrap()
                    .get_mut(&msg.source.clone())
                    .unwrap()
                    .contains_key(&msg.sn)
                    != true
                {
                    delivered_messages
                        .lock()
                        .unwrap()
                        .get_mut(&msg.source.clone())
                        .unwrap()
                        .insert(msg.sn, msg.clone());

                    println!(
                        "{}: Message with info:{} and SN:{} From node:{} delivered succesfully",
                        "FROM RB".green(),
                        msg.info,
                        msg.sn,
                        msg.source
                    );
                } else {
                    println!("{}:Message Already exists", "FROM RB".green());
                }
            }
            _ => {}
        }
    }
}
