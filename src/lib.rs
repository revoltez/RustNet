//# Rustnet
//! A Fast and Reactive Distributed Systems ToolBox
//!
//! RustNet is a set of tools to make building distributed systems easier
mod Delivery;
mod failure_detectors;
mod node;
pub use failure_detectors::failure_detector;
pub use node::Node;
pub use types::{ComponentChannels, Message, MessageType, NetComponent, NetComponents};
pub use Delivery::{BestEffortDelivery, ReliableDelivery, UniformReliableDelivery};
mod types;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
