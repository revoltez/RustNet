use std::net::TcpStream;

use crate::Message;

/// Gurantees reliability only if sender is correct
pub struct BestEffortDelivery {
    peers: Vec<String>,
}

/// gurantees reliability and also considers the behaviour of failed nodes,
/// reliable Delivery gurantees reliability of correct nodes receiving the message but doesnt gurantee if all nodes delivered the message, thats why all nodes
/// must inform each other whether they delivred the message M or not

impl BestEffortDelivery {
    pub fn new(target: Vec<String>) -> BestEffortDelivery {
        BestEffortDelivery { peers: target }
    }
    pub fn send(&self, msg: &Message) {
        for peer in &self.peers {
            let stream = TcpStream::connect(peer)
                .expect("From BestEffortDelivery: error connecting to Peer");
            serde_json::to_writer(stream, &msg)
                .expect("From BestEffortDelivery:failed to push value into stream");
        }
    }
}
