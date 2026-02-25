//! Eche-Lite Wire Protocol
//!
//! Single source of truth for the Eche-Lite binary protocol (ADR-035).
//! This module is `no_std`-compatible and has zero additional dependencies,
//! so it can be consumed by both embedded (`eche-lite`) and hosted (`eche-mesh`) code.

pub mod capabilities;
pub mod constants;
pub mod crdt_type;
pub mod error;
pub mod header;
pub mod message_type;
pub mod ota;
pub mod ttl;

pub use capabilities::NodeCapabilities;
pub use constants::*;
pub use crdt_type::CrdtType;
pub use error::MessageError;
pub use header::{decode_header, encode_header, Header};
pub use message_type::MessageType;
pub use ttl::{append_ttl, default_ttl_for_crdt, strip_ttl};
