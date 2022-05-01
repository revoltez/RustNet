use std::{clone, string, sync::Mutex};

use flume::{Receiver, Sender};
use once_cell::sync::OnceCell;
use pub_sub::{PubSub, Subscription};
use serde::{Deserialize, Serialize};
use serde_json::Result;

// static serial number that increments each time a message gets created
static SN: OnceCell<Mutex<i64>> = OnceCell::new();

/// a wrapper around all types of messages
// need to update this
#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub message_type: MessageType,
    pub info: String,
    pub source: String,
    pub sender: String, //sender could resend a message from source
    pub sn: i64,
}

impl Message {
    pub fn get_sn() -> i64 {
        let mut num = SN
            .get_or_init(|| Mutex::new(0))
            .lock()
            .expect("failled locking into Global Serial number");
        *num = *num + 1;
        return *num;
    }
}

/// all message types must be included in this eunm even newly added component

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageType {
    HeartBeat,
    RequestHeartBeat,
    FailledNode,
    ReliableDelivery,
    BestEffortDelivery,
    UniformReliableDelivery,
    AckDelivery,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum NetComponents {
    FaillureDetector = 1,
    ReliableDelivery = 2,
    BestEffortDelivery = 3,
    UniformReliableDelivery = 4,
}
/// a set of channels that every component must have to communicate with external and internal messages
pub struct ComponentChannels {
    pub rc: Receiver<Message>,
    pub publisher: PubSub<Message>,
    pub subscriptions: Vec<Subscription<Message>>, //subscriptions to other components
}
impl ComponentChannels {
    pub fn subscribe(&mut self, channel: PubSub<Message>) {
        self.subscriptions.push(channel.subscribe());
    }
}

/// NetComponent Trait must be implemeneted to add a new component
pub trait NetComponent {
    fn add_component_channels(&mut self, cmp: ComponentChannels);
    fn start(&mut self);
}
