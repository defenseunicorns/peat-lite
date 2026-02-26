//! Peat-Lite Gossip Protocol
//!
//! Lightweight gossip protocol for mesh communication between Peat nodes.
//! Compatible with Peat-Full nodes via capability negotiation.

pub mod message;
pub mod peer;
pub mod state;
pub mod capabilities;

pub use message::{Message, MessageType, MAX_PACKET_SIZE};
pub use peer::{Peer, PeerTable};
pub use state::{GossipState, MessageTarget};
pub use capabilities::NodeCapabilities;

// Re-export canonical protocol constants from the shared crate.
pub use peat_lite::{DEFAULT_PORT, MAGIC, MULTICAST_ADDR, PROTOCOL_VERSION};
