// Copyright (c) 2025-2026 (r)evolve - Revolve Team LLC
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # hive-lite
//!
//! Lightweight CRDT primitives for resource-constrained HIVE nodes.
//!
//! This crate provides bounded, `no_std`-compatible data structures suitable
//! for devices with limited memory (256KB RAM budget), such as:
//!
//! - WearTAK on Samsung watches
//! - ESP32 sensor nodes
//! - LoRa mesh devices
//!
//! ## Features
//!
//! - **`std`** (default): Enables standard library support
//! - Disable default features for `no_std`: `--no-default-features`
//!
//! ## Primitives
//!
//! | Type | Purpose | Memory |
//! |------|---------|--------|
//! | [`NodeId`] | 32-bit node identifier | 4 bytes |
//! | [`CannedMessage`] | Predefined message codes | 1 byte |
//! | [`CannedMessageEvent`] | Message with metadata | ~24 bytes |
//! | [`LwwRegister`] | Last-writer-wins register | sizeof(T) + 12 bytes |
//! | [`GCounter`] | Grow-only distributed counter | 4 bytes per node |
//!
//! ## Example
//!
//! ```rust
//! use hive_lite::{NodeId, CannedMessage, CannedMessageEvent};
//!
//! let my_node = NodeId::new(0x12345678);
//! let event = CannedMessageEvent::new(
//!     CannedMessage::Ack,
//!     my_node,
//!     Some(NodeId::new(0xDEADBEEF)),  // target
//!     1706234567000,  // timestamp ms
//! );
//!
//! // Encode for transmission
//! let bytes = event.encode();
//! assert_eq!(bytes[0], 0xAF);  // CannedMessage marker
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod node_id;
pub mod canned;
pub mod lww;
pub mod counter;
pub mod wire;

// Re-export main types at crate root
pub use node_id::NodeId;
pub use canned::{CannedMessage, CannedMessageEvent, CannedMessageStore};
pub use lww::LwwRegister;
pub use counter::GCounter;
pub use wire::{CANNED_MESSAGE_MARKER, WireError};
