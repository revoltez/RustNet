//# Faillure Detector
//!assumes synchrony which means timing Bounds of network and processes are fixed with the `timeout` variable
use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};
extern crate pub_sub;
use colored::Colorize;
use flume::Receiver;

use crate::types::*;
//# Faillure Detector
/// Perfect Faillure Detector that Assumes Synchrony which means the Timing Bounds of the
/// network and the processes are fixed with the timeout variable

pub struct FaillureDetector {
    nodes_status: Arc<Mutex<HashMap<String, bool>>>,
    temp_status: Arc<Mutex<HashMap<String, bool>>>,
    timeout: Duration,
    delay: Duration,
    addr: String,
    component_channels: Option<ComponentChannels>,
    peers: Vec<String>,
}
impl FaillureDetector {
    /// Creates a new faillure detector
    pub fn new(
        peers_threashold: usize,
        timeout: Duration,
        delay: Duration,
        addr: String,
        peers: Vec<String>,
    ) -> FaillureDetector {
        FaillureDetector {
            nodes_status: Arc::new(Mutex::new(HashMap::with_capacity(peers_threashold))),
            temp_status: Arc::new(Mutex::new(HashMap::with_capacity(peers_threashold))),
            timeout, //represensts the time needed for nodes to receive the sent heartbeat and reply with another heartbeat
            delay,   // represensts the interval between sending heartbeats
            addr,
            component_channels: None,
            peers,
        }
    }
}

impl NetComponent for FaillureDetector {
    /// starts the Faillure detector by sending and receiving heartbeats and checking nodes status after each timeout + delay seconds
    fn start(&mut self) {
        let fd_rc = self.component_channels.as_ref().unwrap().rc.clone();
        let addr = self.addr.clone();
        let receiver_temp_status = self.temp_status.clone();
        thread::spawn(move || {
            handle_heartbeat(receiver_temp_status, fd_rc, addr.clone());
        });
        for peer in self.peers.iter_mut() {
            self.nodes_status.lock().unwrap().insert(peer.clone(), true);
            self.temp_status.lock().unwrap().insert(peer.clone(), false);
        }
        /*let mut timeout_handle: JoinHandle<HashMap<String, bool>> =
        thread::spawn(move || HashMap::with_capacity(threashold));*/

        let nodes_status = self.nodes_status.clone();
        let temp_status = self.temp_status.clone();
        let timeout = self.timeout.clone();
        let delay = self.delay.clone();
        let publisher = self.component_channels.as_ref().unwrap().publisher.clone();
        let addr = self.addr.clone();
        thread::spawn(move || {
            loop {
                let msg = Message {
                    message_type: MessageType::RequestHeartBeat,
                    info: String::from("Heartbeat"),
                    source: addr.clone(),
                    sender: addr.clone(),
                    sn: Message::get_sn(),
                };
                thread::sleep(delay);
                let nodes_status_clone = nodes_status.clone();
                let temp_status_clone = temp_status.clone();
                // loop through all the remaining correct nodes and check if they are still correct by sending a heartbeat
                thread::spawn(move || {
                    for peer in nodes_status_clone.lock().unwrap().iter() {
                        let socketaddr = SocketAddr::from_str(peer.0).unwrap();
                        if *peer.1 == true {
                            let stream = TcpStream::connect_timeout(
                                &socketaddr,
                                Duration::from_millis(3000),
                            );
                            match stream {
                                Ok(strm) => {
                                    serde_json::to_writer(strm, &msg).expect(
                                        "FROM FD: Failed to serialize The Message into the Sream",
                                    );
                                }
                                Err(e) => {
                                    println!(
                                        "{}:Node: {} is down",
                                        "FROM FD".red(),
                                        peer.0.clone()
                                    );
                                    println!(
                                        "--------------------{}----------------------",
                                        "ERROR".red()
                                    );
                                    println!("{}{}", "FROM FD:".red(), e);
                                    println!("------------------------------------------",);
                                    temp_status_clone
                                        .lock()
                                        .unwrap()
                                        .insert(peer.0.clone(), false);
                                }
                            }
                        }
                    }
                });
                // wait for peers to reply with a heartbeat and update
                // check only for new updates
                thread::sleep(timeout - Duration::from_millis(1000));
                for node in nodes_status.lock().unwrap().iter_mut() {
                    if *node.1 == false
                        && temp_status
                            .lock()
                            .expect("FROM FD:failled accessing temp status")[node.0]
                            == true
                    {
                        println!(
                            "{}: Node:{} was down and is now Alive",
                            "FROM FD".green(),
                            node.0
                        );
                        *node.1 = true;
                    } else if *node.1 == true
                        && temp_status
                            .lock()
                            .expect("FROM FD:failled accessing temp status")[node.0]
                            == false
                    {
                        // publish results to intrested components
                        *node.1 = false;
                        let msg = Message {
                            info: "Node is Down".to_string(),
                            message_type: MessageType::FailledNode,
                            sn: Message::get_sn(),
                            source: node.0.clone(),
                            sender: addr.clone(),
                        };
                        publisher
                            .send(msg)
                            .expect("Error publishing Failled Nodes Results");
                        println!("{}:Node {} is Down", "FROM FD".red(), node.0);
                    }
                }
                // reset all nodes as failled nodes
                for node in temp_status
                    .lock()
                    .expect("FROM FD:failled accessing temp status")
                    .iter_mut()
                {
                    *node.1 = false;
                }
            }
        });
    }
    fn add_component_channels(&mut self, cmp: ComponentChannels) {
        self.component_channels = Some(cmp);
    }
}
/// updates the status of peer after receiving heartBeat to Correct
pub fn handle_heartbeat(
    temp_status: Arc<Mutex<HashMap<String, bool>>>,
    fd_rc: Receiver<Message>,
    addr: String,
) {
    loop {
        let result = fd_rc.recv().expect("From FD : Erroneous value sent");
        match result.message_type {
            MessageType::HeartBeat => {
                temp_status
                    .lock()
                    .unwrap()
                    .insert(result.source.clone(), true);
            }
            MessageType::RequestHeartBeat => {
                let heartbeat = Message {
                    message_type: MessageType::HeartBeat,
                    info: String::from("HeartBeat Message"),
                    source: addr.clone(),
                    sender: addr.clone(),
                    sn: Message::get_sn(),
                };
                let stream =
                    TcpStream::connect(result.source).expect("From Node: error connecting to Peer");
                serde_json::to_writer(stream, &heartbeat)
                    .expect("From Node:failed to push value into stream");
            }
            _ => {}
        }
    }
}
