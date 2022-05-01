//# Rustnet
//! A Fast and Reactive Distributed Systems ToolBox
//!
//! RustNet is a set of tools to make building distributed systems easier
mod Delivery;
mod FaillureDetectors;
mod node;
pub use node::Node;
pub use types::{ComponentChannels, Message, MessageType, NetComponent, NetComponents};
pub use Delivery::{BestEffortDelivery, ReliableDelivery, UniformReliableDelivery};
pub use FaillureDetectors::FaillureDetector;
mod types;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
