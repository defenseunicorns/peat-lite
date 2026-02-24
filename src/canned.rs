// Copyright (c) 2025-2026 (r)evolve - Revolve Team LLC
// SPDX-License-Identifier: Apache-2.0

//! Canned (predefined) message types for resource-constrained devices.
//!
//! These message codes are designed for button-based interaction on devices
//! without keyboard input (e.g., WearTAK on Samsung watches).

use crate::node_id::NodeId;
use crate::wire::CANNED_MESSAGE_MARKER;
use heapless::FnvIndexMap;

/// Predefined message codes for resource-constrained devices.
///
/// Designed for button-based interaction (no keyboard input).
/// Each code fits in a single byte, making wire format compact.
///
/// # Code Ranges
///
/// - `0x00-0x0F`: Acknowledgments
/// - `0x10-0x1F`: Status updates
/// - `0x20-0x2F`: Alerts and emergencies
/// - `0x30-0x3F`: Requests
/// - `0xF0-0xFF`: Reserved/custom
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CannedMessage {
    // ===== Acknowledgments (0x00-0x0F) =====
    /// "Message received" - general acknowledgment
    Ack = 0x00,
    /// "Will comply" - affirmative acknowledgment
    AckWilco = 0x01,
    /// "Cannot comply" - negative acknowledgment
    AckNegative = 0x02,
    /// "Say again" - request repeat
    AckSayAgain = 0x03,

    // ===== Status (0x10-0x1F) =====
    /// "I'm here / still alive" - periodic check-in
    CheckIn = 0x10,
    /// "En route" - moving to objective
    Moving = 0x11,
    /// "Stationary / waiting" - holding position
    Holding = 0x12,
    /// "Arrived at position" - on station
    OnStation = 0x13,
    /// "Returning" - heading back
    Returning = 0x14,
    /// "Mission complete" - task finished
    Complete = 0x15,

    // ===== Alerts (0x20-0x2F) =====
    /// "Need immediate help" - emergency distress
    Emergency = 0x20,
    /// "Attention needed" - non-emergency alert
    Alert = 0x21,
    /// "Situation resolved" - cancel previous alert
    AllClear = 0x22,
    /// "Contact" - enemy/threat spotted
    Contact = 0x23,
    /// "Under fire" - taking fire
    UnderFire = 0x24,

    // ===== Requests (0x30-0x3F) =====
    /// "Request pickup" - need extraction
    NeedExtract = 0x30,
    /// "Request assistance" - need support
    NeedSupport = 0x31,
    /// "Medical emergency" - need medic
    NeedMedic = 0x32,
    /// "Need resupply" - ammunition/supplies
    NeedResupply = 0x33,

    // ===== Reserved (0xF0-0xFF) =====
    /// Custom/application-specific message
    Custom = 0xFF,
}

impl CannedMessage {
    /// Convert from raw byte value.
    ///
    /// Returns `None` for undefined codes.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Ack),
            0x01 => Some(Self::AckWilco),
            0x02 => Some(Self::AckNegative),
            0x03 => Some(Self::AckSayAgain),
            0x10 => Some(Self::CheckIn),
            0x11 => Some(Self::Moving),
            0x12 => Some(Self::Holding),
            0x13 => Some(Self::OnStation),
            0x14 => Some(Self::Returning),
            0x15 => Some(Self::Complete),
            0x20 => Some(Self::Emergency),
            0x21 => Some(Self::Alert),
            0x22 => Some(Self::AllClear),
            0x23 => Some(Self::Contact),
            0x24 => Some(Self::UnderFire),
            0x30 => Some(Self::NeedExtract),
            0x31 => Some(Self::NeedSupport),
            0x32 => Some(Self::NeedMedic),
            0x33 => Some(Self::NeedResupply),
            0xFF => Some(Self::Custom),
            _ => None,
        }
    }

    /// Convert to raw byte value.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is an emergency/alert type message.
    #[inline]
    pub const fn is_alert(self) -> bool {
        matches!(
            self,
            Self::Emergency | Self::Alert | Self::Contact | Self::UnderFire
        )
    }

    /// Check if this is an acknowledgment type message.
    #[inline]
    pub const fn is_ack(self) -> bool {
        matches!(
            self,
            Self::Ack | Self::AckWilco | Self::AckNegative | Self::AckSayAgain
        )
    }

    /// Get a human-readable short name for display.
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Ack => "ACK",
            Self::AckWilco => "WILCO",
            Self::AckNegative => "NEGATIVE",
            Self::AckSayAgain => "SAY AGAIN",
            Self::CheckIn => "CHECK IN",
            Self::Moving => "MOVING",
            Self::Holding => "HOLDING",
            Self::OnStation => "ON STATION",
            Self::Returning => "RTB",
            Self::Complete => "COMPLETE",
            Self::Emergency => "EMERGENCY",
            Self::Alert => "ALERT",
            Self::AllClear => "ALL CLEAR",
            Self::Contact => "CONTACT",
            Self::UnderFire => "UNDER FIRE",
            Self::NeedExtract => "NEED EXTRACT",
            Self::NeedSupport => "NEED SUPPORT",
            Self::NeedMedic => "MEDIC",
            Self::NeedResupply => "RESUPPLY",
            Self::Custom => "CUSTOM",
        }
    }
}

impl Default for CannedMessage {
    fn default() -> Self {
        Self::Ack
    }
}

/// A canned message event with metadata.
///
/// Contains the message code plus source, optional target, timestamp,
/// and sequence number for deduplication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CannedMessageEvent {
    /// The message type
    pub message: CannedMessage,
    /// Source node that sent this message
    pub source_node: NodeId,
    /// Target node (if directed message, e.g., ACK to specific node)
    pub target_node: Option<NodeId>,
    /// Timestamp in milliseconds (epoch or boot time)
    pub timestamp: u64,
    /// Sequence number for deduplication
    pub sequence: u32,
}

impl CannedMessageEvent {
    /// Create a new canned message event.
    pub fn new(
        message: CannedMessage,
        source_node: NodeId,
        target_node: Option<NodeId>,
        timestamp: u64,
    ) -> Self {
        Self {
            message,
            source_node,
            target_node,
            timestamp,
            sequence: 0,
        }
    }

    /// Create with explicit sequence number.
    pub fn with_sequence(
        message: CannedMessage,
        source_node: NodeId,
        target_node: Option<NodeId>,
        timestamp: u64,
        sequence: u32,
    ) -> Self {
        Self {
            message,
            source_node,
            target_node,
            timestamp,
            sequence,
        }
    }

    /// Encode to wire format.
    ///
    /// Format:
    /// ```text
    /// ┌──────┬──────────┬──────────┬──────────┬───────────┬──────┐
    /// │ 0xAF │ msg_code │ src_node │ tgt_node │ timestamp │ seq  │
    /// │ 1B   │ 1B       │ 4B       │ 4B (opt) │ 8B        │ 4B   │
    /// └──────┴──────────┴──────────┴──────────┴───────────┴──────┘
    /// ```
    ///
    /// If target_node is None, those 4 bytes are 0x00000000.
    pub fn encode(&self) -> heapless::Vec<u8, 22> {
        let mut buf = heapless::Vec::new();

        // Marker
        let _ = buf.push(CANNED_MESSAGE_MARKER);

        // Message code
        let _ = buf.push(self.message.as_u8());

        // Source node (4 bytes LE)
        for b in self.source_node.to_le_bytes() {
            let _ = buf.push(b);
        }

        // Target node (4 bytes LE, 0 if None)
        let target = self.target_node.unwrap_or(NodeId::NULL);
        for b in target.to_le_bytes() {
            let _ = buf.push(b);
        }

        // Timestamp (8 bytes LE)
        for b in self.timestamp.to_le_bytes() {
            let _ = buf.push(b);
        }

        // Sequence (4 bytes LE)
        for b in self.sequence.to_le_bytes() {
            let _ = buf.push(b);
        }

        buf
    }

    /// Decode from wire format.
    ///
    /// Returns `None` if data is malformed.
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 22 {
            return None;
        }

        if data[0] != CANNED_MESSAGE_MARKER {
            return None;
        }

        let message = CannedMessage::from_u8(data[1])?;

        let source_node = NodeId::from_le_bytes([data[2], data[3], data[4], data[5]]);

        let target_bytes = [data[6], data[7], data[8], data[9]];
        let target_node = if target_bytes == [0, 0, 0, 0] {
            None
        } else {
            Some(NodeId::from_le_bytes(target_bytes))
        };

        let timestamp = u64::from_le_bytes([
            data[10], data[11], data[12], data[13], data[14], data[15], data[16], data[17],
        ]);

        let sequence = u32::from_le_bytes([data[18], data[19], data[20], data[21]]);

        Some(Self {
            message,
            source_node,
            target_node,
            timestamp,
            sequence,
        })
    }

    /// Check if this event is newer than another from the same source.
    pub fn is_newer_than(&self, other: &Self) -> bool {
        self.timestamp > other.timestamp
            || (self.timestamp == other.timestamp && self.sequence > other.sequence)
    }
}

/// Bounded storage for canned message events.
///
/// Uses LWW (Last-Writer-Wins) semantics per (source_node, message_type) pair.
/// Only stores the latest event of each type from each peer.
///
/// Memory usage: approximately `MAX_ENTRIES * 24 bytes`.
/// Default capacity of 256 entries ≈ 6KB.
pub struct CannedMessageStore<const MAX_ENTRIES: usize = 256> {
    /// Map from (source_node, message) to event
    events: FnvIndexMap<(NodeId, CannedMessage), CannedMessageEvent, MAX_ENTRIES>,
}

impl<const MAX_ENTRIES: usize> Default for CannedMessageStore<MAX_ENTRIES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_ENTRIES: usize> CannedMessageStore<MAX_ENTRIES> {
    /// Create a new empty store.
    pub const fn new() -> Self {
        Self {
            events: FnvIndexMap::new(),
        }
    }

    /// Insert or update an event.
    ///
    /// Only updates if the new event is newer than the existing one.
    /// Returns `true` if the event was inserted/updated.
    pub fn insert(&mut self, event: CannedMessageEvent) -> bool {
        let key = (event.source_node, event.message);

        match self.events.get(&key) {
            Some(existing) if !event.is_newer_than(existing) => false,
            _ => {
                // Insert, possibly evicting oldest if full
                if self.events.len() >= MAX_ENTRIES {
                    // Find and remove oldest entry
                    if let Some(oldest_key) = self.find_oldest_key() {
                        self.events.remove(&oldest_key);
                    }
                }
                self.events.insert(key, event).is_ok()
            }
        }
    }

    /// Get the latest event of a specific type from a specific node.
    pub fn get(&self, source: NodeId, message: CannedMessage) -> Option<&CannedMessageEvent> {
        self.events.get(&(source, message))
    }

    /// Get all events from a specific node.
    pub fn events_from(&self, source: NodeId) -> impl Iterator<Item = &CannedMessageEvent> {
        self.events
            .iter()
            .filter(move |((src, _), _)| *src == source)
            .map(|(_, event)| event)
    }

    /// Get all events of a specific type.
    pub fn events_of_type(
        &self,
        message: CannedMessage,
    ) -> impl Iterator<Item = &CannedMessageEvent> {
        self.events
            .iter()
            .filter(move |((_, msg), _)| *msg == message)
            .map(|(_, event)| event)
    }

    /// Get all emergency/alert events.
    pub fn alerts(&self) -> impl Iterator<Item = &CannedMessageEvent> {
        self.events
            .iter()
            .filter(|((_, msg), _)| msg.is_alert())
            .map(|(_, event)| event)
    }

    /// Number of stored events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Find the key of the oldest event (for eviction).
    fn find_oldest_key(&self) -> Option<(NodeId, CannedMessage)> {
        self.events
            .iter()
            .min_by_key(|(_, event)| (event.timestamp, event.sequence))
            .map(|(key, _)| *key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canned_message_roundtrip() {
        for code in [
            CannedMessage::Ack,
            CannedMessage::Emergency,
            CannedMessage::CheckIn,
            CannedMessage::Custom,
        ] {
            let recovered = CannedMessage::from_u8(code.as_u8()).unwrap();
            assert_eq!(code, recovered);
        }
    }

    #[test]
    fn test_event_encode_decode() {
        let event = CannedMessageEvent::with_sequence(
            CannedMessage::Ack,
            NodeId::new(0x12345678),
            Some(NodeId::new(0xDEADBEEF)),
            1706234567000,
            42,
        );

        let encoded = event.encode();
        assert_eq!(encoded.len(), 22);
        assert_eq!(encoded[0], CANNED_MESSAGE_MARKER);

        let decoded = CannedMessageEvent::decode(&encoded).unwrap();
        assert_eq!(decoded.message, event.message);
        assert_eq!(decoded.source_node, event.source_node);
        assert_eq!(decoded.target_node, event.target_node);
        assert_eq!(decoded.timestamp, event.timestamp);
        assert_eq!(decoded.sequence, event.sequence);
    }

    #[test]
    fn test_event_no_target() {
        let event = CannedMessageEvent::new(
            CannedMessage::Emergency,
            NodeId::new(0x12345678),
            None,
            1706234567000,
        );

        let encoded = event.encode();
        let decoded = CannedMessageEvent::decode(&encoded).unwrap();
        assert_eq!(decoded.target_node, None);
    }

    #[test]
    fn test_store_lww() {
        let mut store = CannedMessageStore::<16>::new();

        let node = NodeId::new(0x123);

        // Insert first event
        let event1 = CannedMessageEvent::with_sequence(CannedMessage::Ack, node, None, 1000, 1);
        assert!(store.insert(event1));

        // Insert older event - should be rejected
        let event_old = CannedMessageEvent::with_sequence(CannedMessage::Ack, node, None, 500, 1);
        assert!(!store.insert(event_old));

        // Insert newer event - should replace
        let event2 = CannedMessageEvent::with_sequence(CannedMessage::Ack, node, None, 2000, 1);
        assert!(store.insert(event2));

        let stored = store.get(node, CannedMessage::Ack).unwrap();
        assert_eq!(stored.timestamp, 2000);
    }

    #[test]
    fn test_store_different_types() {
        let mut store = CannedMessageStore::<16>::new();
        let node = NodeId::new(0x123);

        store.insert(CannedMessageEvent::new(CannedMessage::Ack, node, None, 1000));
        store.insert(CannedMessageEvent::new(
            CannedMessage::Emergency,
            node,
            None,
            1000,
        ));
        store.insert(CannedMessageEvent::new(
            CannedMessage::CheckIn,
            node,
            None,
            1000,
        ));

        assert_eq!(store.len(), 3);
        assert!(store.get(node, CannedMessage::Ack).is_some());
        assert!(store.get(node, CannedMessage::Emergency).is_some());
        assert!(store.get(node, CannedMessage::CheckIn).is_some());
    }
}
