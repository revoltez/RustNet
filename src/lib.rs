//# Rustnet
//! A Fast Reactive Distributed Systems ToolBox
//!
//! RustNet is a set of tools to make building distributed systems easier
mod Delivery;
pub use node::Node;
pub use types::{Message, MessageType, NetComponents, Peer};
pub use Delivery::{BestEffortDelivery, ReliableDelivery, UniformReliableDelivery};
use FaillureDetectors::FD;
pub use FD::FaillureDetector;
mod FaillureDetectors;
mod node;
mod types;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
