//# Rustnet
//! A Fast and Reactive Distributed Systems ToolBox
//!
//! RustNet is a set of tools to make building distributed systems easier
mod delivery;
mod failure_detectors;
mod node;
pub use delivery::{best_effort_delivery, reliable_delivery, uniform_reliable_delivery};
pub use failure_detectors::failure_detector;
pub use node::Node;
pub use types::{ComponentChannels, ComponentTypes, Message, MessageType, NetComponent};
mod types;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
