// Copyright (c) 2025-2026 (r)evolve - Revolve Team LLC
// SPDX-License-Identifier: Apache-2.0

//! Canned (predefined) message types for resource-constrained devices.
//!
//! These message codes are designed for button-based interaction on devices
//! without keyboard input (e.g., WearTAK on Samsung watches).

use crate::node_id::NodeId;
use crate::wire::{
    CANNED_MESSAGE_MARKER, CANNED_MESSAGE_SIGNED_SIZE, CANNED_MESSAGE_UNSIGNED_SIZE, SIGNATURE_SIZE,
};
use heapless::FnvIndexMap;

/// Maximum ACK entries per CannedMessageAckEvent (memory bound for embedded).
pub const MAX_CANNED_ACKS: usize = 64;

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

    /// Encode to signed wire format with Ed25519 signature.
    ///
    /// Format:
    /// ```text
    /// ┌──────┬──────────┬──────────┬──────────┬───────────┬──────┬───────────┐
    /// │ 0xAF │ msg_code │ src_node │ tgt_node │ timestamp │ seq  │ signature │
    /// │ 1B   │ 1B       │ 4B       │ 4B       │ 8B        │ 4B   │ 64B       │
    /// └──────┴──────────┴──────────┴──────────┴───────────┴──────┴───────────┘
    /// ```
    ///
    /// The signature should cover the first 22 bytes (marker through seq).
    /// Caller is responsible for computing the signature using their identity.
    ///
    /// # Arguments
    /// * `signature` - Ed25519 signature over the unsigned payload (22 bytes)
    pub fn encode_signed(&self, signature: &[u8; SIGNATURE_SIZE]) -> heapless::Vec<u8, 86> {
        let mut buf = heapless::Vec::new();

        // Encode unsigned portion (22 bytes)
        let unsigned = self.encode();
        for b in unsigned.iter() {
            let _ = buf.push(*b);
        }

        // Append signature (64 bytes)
        for b in signature.iter() {
            let _ = buf.push(*b);
        }

        buf
    }

    /// Decode from signed wire format.
    ///
    /// Returns the event and signature if the data is exactly 86 bytes
    /// and has a valid marker.
    ///
    /// **Note:** This does NOT verify the signature. Caller must verify
    /// using the sender's public key from the identity registry.
    ///
    /// # Returns
    /// `Some((event, signature))` if valid signed format, `None` otherwise.
    pub fn decode_signed(data: &[u8]) -> Option<(Self, [u8; SIGNATURE_SIZE])> {
        if data.len() != CANNED_MESSAGE_SIGNED_SIZE {
            return None;
        }

        // Decode the unsigned portion
        let event = Self::decode(&data[..CANNED_MESSAGE_UNSIGNED_SIZE])?;

        // Extract signature
        let mut signature = [0u8; SIGNATURE_SIZE];
        signature.copy_from_slice(&data[CANNED_MESSAGE_UNSIGNED_SIZE..]);

        Some((event, signature))
    }

    /// Get the payload bytes that should be signed.
    ///
    /// Returns the 22-byte unsigned wire format suitable for signing.
    /// Use this with your identity's sign() method:
    ///
    /// ```ignore
    /// let payload = event.signable_payload();
    /// let signature = identity.sign(&payload);
    /// let wire = event.encode_signed(&signature);
    /// ```
    #[inline]
    pub fn signable_payload(&self) -> heapless::Vec<u8, 22> {
        self.encode()
    }

    /// Check if wire data is in signed format (86 bytes) vs unsigned (22 bytes).
    ///
    /// Useful for protocol negotiation and backward compatibility.
    #[inline]
    pub fn is_signed_format(data: &[u8]) -> bool {
        data.len() == CANNED_MESSAGE_SIGNED_SIZE && data.first() == Some(&CANNED_MESSAGE_MARKER)
    }

    /// Check if wire data is in unsigned format (22 bytes).
    #[inline]
    pub fn is_unsigned_format(data: &[u8]) -> bool {
        data.len() == CANNED_MESSAGE_UNSIGNED_SIZE && data.first() == Some(&CANNED_MESSAGE_MARKER)
    }

    /// Decode from either signed or unsigned format.
    ///
    /// Returns `(event, Some(signature))` for signed format,
    /// or `(event, None)` for unsigned format.
    ///
    /// # Returns
    /// `Some((event, optional_signature))` if valid format, `None` if malformed.
    pub fn decode_auto(data: &[u8]) -> Option<(Self, Option<[u8; SIGNATURE_SIZE]>)> {
        if Self::is_signed_format(data) {
            Self::decode_signed(data).map(|(e, s)| (e, Some(s)))
        } else if Self::is_unsigned_format(data) {
            Self::decode(data).map(|e| (e, None))
        } else {
            None
        }
    }
}

/// A CannedMessage event with distributed ACK tracking (CRDT).
///
/// This extends [`CannedMessageEvent`] with a map of acknowledgments from other nodes.
/// The ACK map uses OR-set semantics: once a node has acknowledged, it stays acknowledged.
///
/// # CRDT Merge Semantics
///
/// When merging two `CannedMessageAckEvent` instances:
/// - If they represent the same event (same source + timestamp): merge ACK maps with OR semantics
/// - If they represent different events: higher timestamp wins (LWW)
///
/// # Wire Format
///
/// ```text
/// ┌──────┬──────────┬──────────┬──────────┬───────────┬──────┬──────────┬───────────────┐
/// │ 0xAF │ msg_code │ src_node │ tgt_node │ timestamp │ seq  │ num_acks │ acks[N]...    │
/// │ 1B   │ 1B       │ 4B       │ 4B       │ 8B        │ 4B   │ 2B       │ 12B each      │
/// └──────┴──────────┴──────────┴──────────┴───────────┴──────┴──────────┴───────────────┘
/// ```
///
/// Each ACK entry is 12 bytes: acker_node_id (4B LE) + ack_timestamp (8B LE).
#[derive(Debug, Clone)]
pub struct CannedMessageAckEvent {
    /// The message type
    pub message: CannedMessage,
    /// Source node that sent this message
    pub source_node: NodeId,
    /// Target node (if directed, e.g., ACK to specific node)
    pub target_node: Option<NodeId>,
    /// Timestamp when message was sent
    pub timestamp: u64,
    /// Sequence number for deduplication
    pub sequence: u32,
    /// ACK tracking: acker_node_id -> ack_timestamp
    acks: FnvIndexMap<NodeId, u64, MAX_CANNED_ACKS>,
}

impl CannedMessageAckEvent {
    /// Create a new event. The source node auto-acknowledges.
    pub fn new(
        message: CannedMessage,
        source_node: NodeId,
        target_node: Option<NodeId>,
        timestamp: u64,
    ) -> Self {
        let mut acks = FnvIndexMap::new();
        // Source node implicitly acknowledges their own message
        let _ = acks.insert(source_node, timestamp);

        Self {
            message,
            source_node,
            target_node,
            timestamp,
            sequence: 0,
            acks,
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
        let mut event = Self::new(message, source_node, target_node, timestamp);
        event.sequence = sequence;
        event
    }

    /// Record an ACK from a node.
    ///
    /// Returns `true` if this is a new ACK or updates an existing one to a later timestamp.
    /// Returns `false` if the ACK was already recorded with an equal or later timestamp,
    /// or if the ACK map is full.
    pub fn ack(&mut self, node_id: NodeId, ack_timestamp: u64) -> bool {
        if node_id == NodeId::NULL {
            return false;
        }

        match self.acks.get(&node_id) {
            Some(&existing_ts) if existing_ts >= ack_timestamp => false,
            Some(_) => {
                // Update existing entry with newer timestamp
                let _ = self.acks.insert(node_id, ack_timestamp);
                true
            }
            None => {
                // New ACK entry
                self.acks.insert(node_id, ack_timestamp).is_ok()
            }
        }
    }

    /// Check if a node has acknowledged this message.
    pub fn has_acked(&self, node_id: NodeId) -> bool {
        self.acks.contains_key(&node_id)
    }

    /// Get all node IDs that have acknowledged this message.
    pub fn acked_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.acks.keys().copied()
    }

    /// Get the ACK timestamp for a specific node.
    pub fn ack_timestamp(&self, node_id: NodeId) -> Option<u64> {
        self.acks.get(&node_id).copied()
    }

    /// Number of ACKs received (including source's implicit ACK).
    pub fn ack_count(&self) -> usize {
        self.acks.len()
    }

    /// CRDT merge with another event.
    ///
    /// - Same event (source + timestamp match): merge ACK maps with OR semantics
    /// - Different event: higher timestamp wins (LWW)
    ///
    /// Returns `true` if this event's state changed.
    pub fn merge(&mut self, other: &Self) -> bool {
        // Different message identity - higher timestamp wins
        if self.source_node != other.source_node || self.timestamp != other.timestamp {
            if other.timestamp > self.timestamp {
                *self = other.clone();
                return true;
            }
            return false;
        }

        // Same message - merge ACK maps with OR, keep latest timestamp per acker
        let mut changed = false;
        for (node_id, &other_ts) in other.acks.iter() {
            match self.acks.get(node_id) {
                Some(&existing_ts) if existing_ts >= other_ts => {}
                _ => {
                    if self.acks.insert(*node_id, other_ts).is_ok() {
                        changed = true;
                    }
                }
            }
        }
        changed
    }

    /// Encode to wire format.
    ///
    /// Returns a buffer containing the base event (22 bytes) plus ACK state.
    /// Format: 24 base bytes + (12 bytes per ACK entry).
    pub fn encode(&self) -> heapless::Vec<u8, 792> {
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

        // Number of ACKs (2 bytes LE)
        let num_acks = self.acks.len() as u16;
        for b in num_acks.to_le_bytes() {
            let _ = buf.push(b);
        }

        // ACK entries (12 bytes each: 4B node_id + 8B timestamp)
        for (node_id, &ack_ts) in self.acks.iter() {
            for b in node_id.to_le_bytes() {
                let _ = buf.push(b);
            }
            for b in ack_ts.to_le_bytes() {
                let _ = buf.push(b);
            }
        }

        buf
    }

    /// Decode from wire format.
    ///
    /// Handles both base format (22 bytes, no ACKs) and extended format (24+ bytes with ACKs).
    /// Returns `None` if data is malformed.
    pub fn decode(data: &[u8]) -> Option<Self> {
        // Minimum: 22 bytes for base event (backward compat)
        if data.len() < 22 {
            return None;
        }

        if data[0] != CANNED_MESSAGE_MARKER {
            return None;
        }

        let message = CannedMessage::from_u8(data[1])?;

        let source_node = NodeId::from_le_bytes([data[2], data[3], data[4], data[5]]);

        // Security check: reject NULL source
        if source_node == NodeId::NULL {
            return None;
        }

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

        // Build base ACK map with source's implicit ACK
        let mut acks = FnvIndexMap::new();
        let _ = acks.insert(source_node, timestamp);

        // Check for extended format with ACK state
        if data.len() >= 24 {
            let num_acks = u16::from_le_bytes([data[22], data[23]]);

            // Security check: reject excessive ACK counts
            if num_acks as usize > MAX_CANNED_ACKS {
                return None;
            }

            let expected_len = 24 + (num_acks as usize * 12);
            if data.len() < expected_len {
                return None;
            }

            // Parse ACK entries
            let mut offset = 24;
            for _ in 0..num_acks {
                let acker_node = NodeId::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                let ack_ts = u64::from_le_bytes([
                    data[offset + 4],
                    data[offset + 5],
                    data[offset + 6],
                    data[offset + 7],
                    data[offset + 8],
                    data[offset + 9],
                    data[offset + 10],
                    data[offset + 11],
                ]);
                offset += 12;

                // Skip NULL node IDs (invalid entries)
                if acker_node != NodeId::NULL {
                    let _ = acks.insert(acker_node, ack_ts);
                }
            }
        }

        Some(Self {
            message,
            source_node,
            target_node,
            timestamp,
            sequence,
            acks,
        })
    }

    /// Convert to base [`CannedMessageEvent`] (without ACK state).
    pub fn as_event(&self) -> CannedMessageEvent {
        CannedMessageEvent {
            message: self.message,
            source_node: self.source_node,
            target_node: self.target_node,
            timestamp: self.timestamp,
            sequence: self.sequence,
        }
    }

    /// Create from a base [`CannedMessageEvent`] (no ACKs except source).
    pub fn from_event(event: CannedMessageEvent) -> Self {
        let mut acks = FnvIndexMap::new();
        let _ = acks.insert(event.source_node, event.timestamp);

        Self {
            message: event.message,
            source_node: event.source_node,
            target_node: event.target_node,
            timestamp: event.timestamp,
            sequence: event.sequence,
            acks,
        }
    }
}

impl PartialEq for CannedMessageAckEvent {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
            && self.source_node == other.source_node
            && self.target_node == other.target_node
            && self.timestamp == other.timestamp
            && self.sequence == other.sequence
            && self.acks.len() == other.acks.len()
            && self.acks.iter().all(|(k, v)| other.acks.get(k) == Some(v))
    }
}

impl Eq for CannedMessageAckEvent {}

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

        store.insert(CannedMessageEvent::new(
            CannedMessage::Ack,
            node,
            None,
            1000,
        ));
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

    // ===== CannedMessageAckEvent tests =====

    #[test]
    fn test_ack_event_creation() {
        let source = NodeId::new(0x12345678);
        let event = CannedMessageAckEvent::new(CannedMessage::CheckIn, source, None, 1706234567000);

        // Source should auto-ack
        assert!(event.has_acked(source));
        assert_eq!(event.ack_count(), 1);
        assert_eq!(event.ack_timestamp(source), Some(1706234567000));
    }

    #[test]
    fn test_ack_recording() {
        let source = NodeId::new(0x111);
        let acker = NodeId::new(0x222);

        let mut event = CannedMessageAckEvent::new(CannedMessage::Emergency, source, None, 1000);

        // New ACK returns true
        assert!(event.ack(acker, 1500));
        assert!(event.has_acked(acker));
        assert_eq!(event.ack_count(), 2);

        // Same ACK with same timestamp returns false
        assert!(!event.ack(acker, 1500));

        // Same ACK with older timestamp returns false
        assert!(!event.ack(acker, 1400));

        // Same ACK with newer timestamp returns true (updates)
        assert!(event.ack(acker, 1600));
        assert_eq!(event.ack_timestamp(acker), Some(1600));

        // NULL node ID rejected
        assert!(!event.ack(NodeId::NULL, 2000));
    }

    #[test]
    fn test_ack_merge_same_event() {
        let source = NodeId::new(0x111);
        let node_a = NodeId::new(0x222);
        let node_b = NodeId::new(0x333);

        // Event 1: source + node_a acked
        let mut event1 = CannedMessageAckEvent::new(CannedMessage::CheckIn, source, None, 1000);
        event1.ack(node_a, 1100);

        // Event 2 (same message): source + node_b acked
        let mut event2 = CannedMessageAckEvent::new(CannedMessage::CheckIn, source, None, 1000);
        event2.ack(node_b, 1200);

        // Merge should combine ACKs (OR semantics)
        let changed = event1.merge(&event2);
        assert!(changed);
        assert!(event1.has_acked(source));
        assert!(event1.has_acked(node_a));
        assert!(event1.has_acked(node_b));
        assert_eq!(event1.ack_count(), 3);

        // Merging again should not change
        assert!(!event1.merge(&event2));
    }

    #[test]
    fn test_ack_merge_different_event() {
        let source = NodeId::new(0x111);
        let acker = NodeId::new(0x222);

        // Older event with ACK
        let mut older = CannedMessageAckEvent::new(CannedMessage::CheckIn, source, None, 1000);
        older.ack(acker, 1100);

        // Newer event without that ACK
        let newer = CannedMessageAckEvent::new(
            CannedMessage::Alert, // Different message type, same source
            source,
            None,
            2000,
        );

        // Higher timestamp wins (LWW)
        let changed = older.merge(&newer);
        assert!(changed);
        assert_eq!(older.timestamp, 2000);
        assert_eq!(older.message, CannedMessage::Alert);
        // The old ACK is gone, only source's implicit ACK remains
        assert!(!older.has_acked(acker));
        assert_eq!(older.ack_count(), 1);

        // Merging older into newer should not change
        let mut newer2 = CannedMessageAckEvent::new(CannedMessage::Alert, source, None, 2000);
        let older2 = CannedMessageAckEvent::new(CannedMessage::CheckIn, source, None, 1000);
        assert!(!newer2.merge(&older2));
    }

    #[test]
    fn test_ack_encode_decode() {
        let source = NodeId::new(0x12345678);
        let target = NodeId::new(0xDEADBEEF);
        let acker1 = NodeId::new(0xAAAA);
        let acker2 = NodeId::new(0xBBBB);

        let mut event = CannedMessageAckEvent::with_sequence(
            CannedMessage::Emergency,
            source,
            Some(target),
            1706234567000,
            42,
        );
        event.ack(acker1, 1706234568000);
        event.ack(acker2, 1706234569000);

        let encoded = event.encode();
        // 24 base + 3 ACKs * 12 = 60 bytes
        assert_eq!(encoded.len(), 24 + 3 * 12);
        assert_eq!(encoded[0], CANNED_MESSAGE_MARKER);

        let decoded = CannedMessageAckEvent::decode(&encoded).unwrap();
        assert_eq!(decoded.message, event.message);
        assert_eq!(decoded.source_node, event.source_node);
        assert_eq!(decoded.target_node, event.target_node);
        assert_eq!(decoded.timestamp, event.timestamp);
        assert_eq!(decoded.sequence, event.sequence);
        assert_eq!(decoded.ack_count(), 3);
        assert!(decoded.has_acked(source));
        assert!(decoded.has_acked(acker1));
        assert!(decoded.has_acked(acker2));
        assert_eq!(decoded.ack_timestamp(acker1), Some(1706234568000));
        assert_eq!(decoded.ack_timestamp(acker2), Some(1706234569000));
    }

    #[test]
    fn test_ack_decode_base_event() {
        // Create a base CannedMessageEvent (22 bytes)
        let base_event = CannedMessageEvent::with_sequence(
            CannedMessage::CheckIn,
            NodeId::new(0x12345678),
            None,
            1706234567000,
            5,
        );

        let encoded = base_event.encode();
        assert_eq!(encoded.len(), 22);

        // CannedMessageAckEvent should decode it with implicit source ACK
        let decoded = CannedMessageAckEvent::decode(&encoded).unwrap();
        assert_eq!(decoded.message, base_event.message);
        assert_eq!(decoded.source_node, base_event.source_node);
        assert_eq!(decoded.timestamp, base_event.timestamp);
        assert_eq!(decoded.sequence, base_event.sequence);
        // Only source's implicit ACK
        assert_eq!(decoded.ack_count(), 1);
        assert!(decoded.has_acked(base_event.source_node));
    }

    #[test]
    fn test_ack_max_limit() {
        let source = NodeId::new(0x111);
        let mut event = CannedMessageAckEvent::new(CannedMessage::Emergency, source, None, 1000);

        // Fill up to MAX_CANNED_ACKS - 1 (since source already has one)
        for i in 1..MAX_CANNED_ACKS {
            let acker = NodeId::new(i as u32 + 1000);
            assert!(
                event.ack(acker, 2000 + i as u64),
                "ack {} should succeed",
                i
            );
        }

        assert_eq!(event.ack_count(), MAX_CANNED_ACKS);

        // Next ACK should fail (map full)
        let overflow_acker = NodeId::new(0xFFFFFF);
        assert!(!event.ack(overflow_acker, 9999));
        assert!(!event.has_acked(overflow_acker));
    }

    #[test]
    fn test_ack_validation() {
        // Too short
        assert!(CannedMessageAckEvent::decode(&[0xAF]).is_none());
        assert!(CannedMessageAckEvent::decode(&[0xAF; 21]).is_none());

        // Wrong marker
        let mut bad_marker = [0u8; 22];
        bad_marker[0] = 0x00;
        assert!(CannedMessageAckEvent::decode(&bad_marker).is_none());

        // NULL source node
        let mut null_source = [0u8; 22];
        null_source[0] = CANNED_MESSAGE_MARKER;
        null_source[1] = CannedMessage::Ack.as_u8();
        // source bytes 2-5 are 0 (NULL)
        assert!(CannedMessageAckEvent::decode(&null_source).is_none());

        // Invalid message code
        let mut bad_code = [0u8; 22];
        bad_code[0] = CANNED_MESSAGE_MARKER;
        bad_code[1] = 0xEE; // Invalid code
        bad_code[2] = 1; // Non-null source
        assert!(CannedMessageAckEvent::decode(&bad_code).is_none());

        // Excessive num_acks
        let mut excessive_acks = [0u8; 24];
        excessive_acks[0] = CANNED_MESSAGE_MARKER;
        excessive_acks[1] = CannedMessage::Ack.as_u8();
        excessive_acks[2] = 1; // Non-null source
                               // num_acks = 0xFFFF (65535) at bytes 22-23
        excessive_acks[22] = 0xFF;
        excessive_acks[23] = 0xFF;
        assert!(CannedMessageAckEvent::decode(&excessive_acks).is_none());

        // num_acks declares more than data provides
        let mut truncated = [0u8; 24];
        truncated[0] = CANNED_MESSAGE_MARKER;
        truncated[1] = CannedMessage::Ack.as_u8();
        truncated[2] = 1; // Non-null source
        truncated[22] = 5; // Claims 5 ACKs but no data follows
        truncated[23] = 0;
        assert!(CannedMessageAckEvent::decode(&truncated).is_none());
    }

    #[test]
    fn test_ack_event_as_event_roundtrip() {
        let source = NodeId::new(0x12345678);
        let target = NodeId::new(0xDEADBEEF);

        let ack_event = CannedMessageAckEvent::with_sequence(
            CannedMessage::NeedMedic,
            source,
            Some(target),
            1706234567000,
            99,
        );

        let base = ack_event.as_event();
        assert_eq!(base.message, CannedMessage::NeedMedic);
        assert_eq!(base.source_node, source);
        assert_eq!(base.target_node, Some(target));
        assert_eq!(base.timestamp, 1706234567000);
        assert_eq!(base.sequence, 99);

        // Convert back
        let restored = CannedMessageAckEvent::from_event(base);
        assert_eq!(restored.message, ack_event.message);
        assert_eq!(restored.source_node, ack_event.source_node);
        assert_eq!(restored.target_node, ack_event.target_node);
        assert_eq!(restored.timestamp, ack_event.timestamp);
        assert_eq!(restored.sequence, ack_event.sequence);
        // Only source ACK restored
        assert_eq!(restored.ack_count(), 1);
        assert!(restored.has_acked(source));
    }

    #[test]
    fn test_ack_event_acked_nodes_iterator() {
        let source = NodeId::new(0x111);
        let mut event = CannedMessageAckEvent::new(CannedMessage::CheckIn, source, None, 1000);
        event.ack(NodeId::new(0x222), 1100);
        event.ack(NodeId::new(0x333), 1200);

        let nodes: heapless::Vec<NodeId, 8> = event.acked_nodes().collect();
        assert_eq!(nodes.len(), 3);
        assert!(nodes.contains(&source));
        assert!(nodes.contains(&NodeId::new(0x222)));
        assert!(nodes.contains(&NodeId::new(0x333)));
    }

    // ===== Signed CannedMessageEvent tests =====

    #[test]
    fn test_signed_event_encode_decode() {
        let event = CannedMessageEvent::with_sequence(
            CannedMessage::Emergency,
            NodeId::new(0x12345678),
            Some(NodeId::new(0xDEADBEEF)),
            1706234567000,
            42,
        );

        // Create a dummy signature (in real use, this comes from identity.sign())
        let signature = [0xABu8; 64];

        let encoded = event.encode_signed(&signature);
        assert_eq!(encoded.len(), 86);
        assert_eq!(encoded[0], CANNED_MESSAGE_MARKER);

        let (decoded, decoded_sig) = CannedMessageEvent::decode_signed(&encoded).unwrap();
        assert_eq!(decoded.message, event.message);
        assert_eq!(decoded.source_node, event.source_node);
        assert_eq!(decoded.target_node, event.target_node);
        assert_eq!(decoded.timestamp, event.timestamp);
        assert_eq!(decoded.sequence, event.sequence);
        assert_eq!(decoded_sig, signature);
    }

    #[test]
    fn test_signed_format_detection() {
        let event = CannedMessageEvent::new(
            CannedMessage::CheckIn,
            NodeId::new(0x12345678),
            None,
            1706234567000,
        );

        // Unsigned format (22 bytes)
        let unsigned = event.encode();
        assert!(CannedMessageEvent::is_unsigned_format(&unsigned));
        assert!(!CannedMessageEvent::is_signed_format(&unsigned));

        // Signed format (86 bytes)
        let signature = [0x00u8; 64];
        let signed = event.encode_signed(&signature);
        assert!(CannedMessageEvent::is_signed_format(&signed));
        assert!(!CannedMessageEvent::is_unsigned_format(&signed));

        // Invalid formats
        assert!(!CannedMessageEvent::is_signed_format(&[0xAF; 50]));
        assert!(!CannedMessageEvent::is_unsigned_format(&[0xAF; 50]));
        assert!(!CannedMessageEvent::is_signed_format(&[0x00; 86])); // Wrong marker
    }

    #[test]
    fn test_decode_auto() {
        let event = CannedMessageEvent::with_sequence(
            CannedMessage::Alert,
            NodeId::new(0xAAAA),
            None,
            1000,
            5,
        );

        // Test unsigned
        let unsigned = event.encode();
        let (decoded, sig_opt) = CannedMessageEvent::decode_auto(&unsigned).unwrap();
        assert_eq!(decoded.message, event.message);
        assert!(sig_opt.is_none());

        // Test signed
        let signature = [0xFFu8; 64];
        let signed = event.encode_signed(&signature);
        let (decoded, sig_opt) = CannedMessageEvent::decode_auto(&signed).unwrap();
        assert_eq!(decoded.message, event.message);
        assert_eq!(sig_opt, Some(signature));

        // Test invalid
        assert!(CannedMessageEvent::decode_auto(&[0xAF; 50]).is_none());
    }

    #[test]
    fn test_signable_payload() {
        let event = CannedMessageEvent::new(
            CannedMessage::Moving,
            NodeId::new(0x12345678),
            None,
            1706234567000,
        );

        let payload = event.signable_payload();
        let encoded = event.encode();

        // Should be identical to unsigned encode
        assert_eq!(payload.as_slice(), encoded.as_slice());
        assert_eq!(payload.len(), 22);
    }

    #[test]
    fn test_signed_decode_wrong_size() {
        // Too short
        assert!(CannedMessageEvent::decode_signed(&[0xAF; 85]).is_none());

        // Too long
        assert!(CannedMessageEvent::decode_signed(&[0xAF; 87]).is_none());

        // Wrong marker
        let mut bad = [0u8; 86];
        bad[0] = 0x00;
        assert!(CannedMessageEvent::decode_signed(&bad).is_none());
    }

    #[test]
    fn test_wire_size_constants() {
        use crate::wire::{
            CANNED_MESSAGE_SIGNED_SIZE, CANNED_MESSAGE_UNSIGNED_SIZE, SIGNATURE_SIZE,
        };

        assert_eq!(CANNED_MESSAGE_UNSIGNED_SIZE, 22);
        assert_eq!(SIGNATURE_SIZE, 64);
        assert_eq!(CANNED_MESSAGE_SIGNED_SIZE, 86);
        assert_eq!(
            CANNED_MESSAGE_SIGNED_SIZE,
            CANNED_MESSAGE_UNSIGNED_SIZE + SIGNATURE_SIZE
        );
    }
}
