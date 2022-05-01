use colored::Colorize;
pub use rust_net;
use rust_net::{Message, MessageType, Node};
use std::{
    process::exit,
    thread::{self},
    time::Duration,
};
fn main() {
    let peers = vec![
        String::from("172.17.0.2:8888"),
        String::from("172.17.0.3:8888"),
        String::from("172.17.0.4:8888"),
        String::from("172.17.0.5:8888"),
    ];

    let mut node = Node::new("First".to_string(), 8888, 5);
    node.add_peers(peers)
        .has_faillure_detector(Duration::from_millis(3000), Duration::from_millis(2000))
        .on_receive_message(
            MessageType::UniformReliableDelivery,
            Box::new(|msg| {
                println!("*********************************************");
                println!("{}: Triggering my callback", "FROM USER".green());
                println!("*********************************************");
            }),
        )
        .has_uniform_reliable_delivery();
    let nodehandle = node.start();

    let msg = Message {
        info: "Testing Uniform reliable delivery".to_string(),
        message_type: MessageType::UniformReliableDelivery,
        sn: Message::get_sn(),
        sender: node.addr.clone(),
        source: node.addr.clone(),
    };
    thread::sleep(Duration::from_millis(4000));
    node.broadcast(&msg);

    ctrlc::set_handler(|| {
        exit(0);
    })
    .expect("Failled exiing process");

    nodehandle.join().expect("Failled Joining node handle"); // will never join
}
