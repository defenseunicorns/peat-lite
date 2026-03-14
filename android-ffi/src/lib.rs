// Copyright (c) 2025-2026 Defense Unicorns
// SPDX-License-Identifier: Apache-2.0

//! Android FFI crate for peat-lite.
//!
//! This crate provides UniFFI bindings for Android/Kotlin.
//! It wraps the core no_std types with std-based alternatives.

use peat_lite::canned::{CannedMessage, CannedMessageAckEvent, CannedMessageEvent};
use peat_lite::node_id::NodeId;
use peat_lite::wire::CANNED_MESSAGE_MARKER;

// UniFFI scaffolding - generates the C FFI layer
uniffi::setup_scaffolding!();

/// CannedMessage enum exported to Kotlin.
///
/// Maps directly to the core CannedMessage but with UniFFI derive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum CannedMessageType {
    // Acknowledgments (0x00-0x0F)
    Ack,
    AckWilco,
    AckNegative,
    AckSayAgain,
    // Status (0x10-0x1F)
    CheckIn,
    Moving,
    Holding,
    OnStation,
    Returning,
    Complete,
    // Alerts (0x20-0x2F)
    Emergency,
    Alert,
    AllClear,
    Contact,
    UnderFire,
    // Requests (0x30-0x3F)
    NeedExtract,
    NeedSupport,
    NeedMedic,
    NeedResupply,
    // Reserved
    Custom,
}

impl From<CannedMessage> for CannedMessageType {
    fn from(msg: CannedMessage) -> Self {
        match msg {
            CannedMessage::Ack => Self::Ack,
            CannedMessage::AckWilco => Self::AckWilco,
            CannedMessage::AckNegative => Self::AckNegative,
            CannedMessage::AckSayAgain => Self::AckSayAgain,
            CannedMessage::CheckIn => Self::CheckIn,
            CannedMessage::Moving => Self::Moving,
            CannedMessage::Holding => Self::Holding,
            CannedMessage::OnStation => Self::OnStation,
            CannedMessage::Returning => Self::Returning,
            CannedMessage::Complete => Self::Complete,
            CannedMessage::Emergency => Self::Emergency,
            CannedMessage::Alert => Self::Alert,
            CannedMessage::AllClear => Self::AllClear,
            CannedMessage::Contact => Self::Contact,
            CannedMessage::UnderFire => Self::UnderFire,
            CannedMessage::NeedExtract => Self::NeedExtract,
            CannedMessage::NeedSupport => Self::NeedSupport,
            CannedMessage::NeedMedic => Self::NeedMedic,
            CannedMessage::NeedResupply => Self::NeedResupply,
            CannedMessage::Custom => Self::Custom,
        }
    }
}

impl From<CannedMessageType> for CannedMessage {
    fn from(msg: CannedMessageType) -> Self {
        match msg {
            CannedMessageType::Ack => Self::Ack,
            CannedMessageType::AckWilco => Self::AckWilco,
            CannedMessageType::AckNegative => Self::AckNegative,
            CannedMessageType::AckSayAgain => Self::AckSayAgain,
            CannedMessageType::CheckIn => Self::CheckIn,
            CannedMessageType::Moving => Self::Moving,
            CannedMessageType::Holding => Self::Holding,
            CannedMessageType::OnStation => Self::OnStation,
            CannedMessageType::Returning => Self::Returning,
            CannedMessageType::Complete => Self::Complete,
            CannedMessageType::Emergency => Self::Emergency,
            CannedMessageType::Alert => Self::Alert,
            CannedMessageType::AllClear => Self::AllClear,
            CannedMessageType::Contact => Self::Contact,
            CannedMessageType::UnderFire => Self::UnderFire,
            CannedMessageType::NeedExtract => Self::NeedExtract,
            CannedMessageType::NeedSupport => Self::NeedSupport,
            CannedMessageType::NeedMedic => Self::NeedMedic,
            CannedMessageType::NeedResupply => Self::NeedResupply,
            CannedMessageType::Custom => Self::Custom,
        }
    }
}

impl CannedMessageType {
    /// Get the wire format code for this message type.
    pub fn code(&self) -> u8 {
        CannedMessage::from(*self).as_u8()
    }

    /// Create from wire format code.
    pub fn from_code(code: u8) -> Option<Self> {
        CannedMessage::from_u8(code).map(|m| m.into())
    }

    /// Get short display name.
    pub fn short_name(&self) -> String {
        CannedMessage::from(*self).short_name().to_string()
    }

    /// Check if this is an alert/emergency type.
    pub fn is_alert(&self) -> bool {
        CannedMessage::from(*self).is_alert()
    }

    /// Check if this is an acknowledgment type.
    pub fn is_ack(&self) -> bool {
        CannedMessage::from(*self).is_ack()
    }
}

/// CannedMessageEvent exported to Kotlin.
///
/// Contains message type plus metadata (source, target, timestamp, sequence).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct CannedMessageEventData {
    /// The message type
    pub message: CannedMessageType,
    /// Source node ID (who sent this)
    pub source_node: u32,
    /// Target node ID (0 if broadcast)
    pub target_node: u32,
    /// Timestamp in milliseconds
    pub timestamp: u64,
    /// Sequence number for deduplication
    pub sequence: u32,
}

impl From<CannedMessageEvent> for CannedMessageEventData {
    fn from(event: CannedMessageEvent) -> Self {
        Self {
            message: event.message.into(),
            source_node: event.source_node.as_u32(),
            target_node: event.target_node.map(|n| n.as_u32()).unwrap_or(0),
            timestamp: event.timestamp,
            sequence: event.sequence,
        }
    }
}

impl From<CannedMessageEventData> for CannedMessageEvent {
    fn from(data: CannedMessageEventData) -> Self {
        let target = if data.target_node == 0 {
            None
        } else {
            Some(NodeId::new(data.target_node))
        };
        CannedMessageEvent::with_sequence(
            data.message.into(),
            NodeId::new(data.source_node),
            target,
            data.timestamp,
            data.sequence,
        )
    }
}

// ============================================================================
// UniFFI exported functions
// ============================================================================

/// Get the CannedMessage marker byte (0xAF).
#[uniffi::export]
pub fn canned_message_marker() -> u8 {
    CANNED_MESSAGE_MARKER
}

/// Create a new CannedMessageEvent.
#[uniffi::export]
pub fn create_canned_message_event(
    message: CannedMessageType,
    source_node: u32,
    target_node: u32,
    timestamp: u64,
    sequence: u32,
) -> CannedMessageEventData {
    CannedMessageEventData {
        message,
        source_node,
        target_node,
        timestamp,
        sequence,
    }
}

/// Encode a CannedMessageEvent to wire format bytes.
///
/// Returns 22 bytes: [0xAF][code:1][src:4][tgt:4][timestamp:8][seq:4]
#[uniffi::export]
pub fn encode_canned_message_event(event: CannedMessageEventData) -> Vec<u8> {
    let core_event: CannedMessageEvent = event.into();
    core_event.encode().to_vec()
}

/// Decode wire format bytes to a CannedMessageEvent.
///
/// Returns None if data is malformed (wrong marker, too short, etc).
#[uniffi::export]
pub fn decode_canned_message_event(data: Vec<u8>) -> Option<CannedMessageEventData> {
    CannedMessageEvent::decode(&data).map(|e| e.into())
}

/// Get the short display name for a message type.
#[uniffi::export]
pub fn canned_message_short_name(message: CannedMessageType) -> String {
    message.short_name()
}

/// Get the wire format code for a message type.
#[uniffi::export]
pub fn canned_message_code(message: CannedMessageType) -> u8 {
    message.code()
}

/// Create a CannedMessageType from its wire format code.
///
/// Returns None if the code is not recognized.
#[uniffi::export]
pub fn canned_message_from_code(code: u8) -> Option<CannedMessageType> {
    CannedMessageType::from_code(code)
}

/// Check if a message type is an alert/emergency.
#[uniffi::export]
pub fn canned_message_is_alert(message: CannedMessageType) -> bool {
    message.is_alert()
}

/// Check if a message type is an acknowledgment.
#[uniffi::export]
pub fn canned_message_is_ack(message: CannedMessageType) -> bool {
    message.is_ack()
}

// ============================================================================
// CannedMessageAckEvent - CRDT Document with Embedded ACK Tracking
// ============================================================================

/// ACK entry: which node acknowledged and when.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct AckEntry {
    /// Node ID that sent the ACK
    pub node_id: u32,
    /// Timestamp when ACK was sent
    pub timestamp: u64,
}

/// CannedMessageAckEvent - a canned message with embedded ACK tracking.
///
/// Unlike `CannedMessageEventData`, this includes a map of ACKs from other nodes.
/// ACKs are updates to the document, not separate messages.
///
/// Wire format: [0xAF][code:1][src:4][tgt:4][timestamp:8][seq:4][num_acks:2][acks...]
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct CannedMessageAckEventData {
    /// The message type
    pub message: CannedMessageType,
    /// Source node ID (who sent this)
    pub source_node: u32,
    /// Target node ID (0 if broadcast)
    pub target_node: u32,
    /// Timestamp in milliseconds
    pub timestamp: u64,
    /// Sequence number for deduplication
    pub sequence: u32,
    /// ACK entries: (node_id, ack_timestamp) pairs
    pub acks: Vec<AckEntry>,
}

impl From<CannedMessageAckEvent> for CannedMessageAckEventData {
    fn from(event: CannedMessageAckEvent) -> Self {
        let acks: Vec<AckEntry> = event
            .acked_nodes()
            .map(|node_id| AckEntry {
                node_id: node_id.as_u32(),
                timestamp: event.ack_timestamp(node_id).unwrap_or(0),
            })
            .collect();

        Self {
            message: event.message.into(),
            source_node: event.source_node.as_u32(),
            target_node: event.target_node.map(|n| n.as_u32()).unwrap_or(0),
            timestamp: event.timestamp,
            sequence: event.sequence,
            acks,
        }
    }
}

/// Create a new CannedMessageAckEvent (source auto-acknowledges).
#[uniffi::export]
pub fn create_canned_message_ack_event(
    message: CannedMessageType,
    source_node: u32,
    target_node: u32,
    timestamp: u64,
    sequence: u32,
) -> CannedMessageAckEventData {
    let target = if target_node == 0 {
        None
    } else {
        Some(NodeId::new(target_node))
    };
    let event = CannedMessageAckEvent::with_sequence(
        message.into(),
        NodeId::new(source_node),
        target,
        timestamp,
        sequence,
    );
    event.into()
}

/// Encode a CannedMessageAckEvent to wire format bytes.
///
/// Returns 24+ bytes: [0xAF][code:1][src:4][tgt:4][timestamp:8][seq:4][num_acks:2][acks...]
/// Each ACK entry is 12 bytes: [node_id:4][ack_timestamp:8]
#[uniffi::export]
pub fn encode_canned_message_ack_event(event: CannedMessageAckEventData) -> Vec<u8> {
    // Rebuild the core event with ACKs
    let target = if event.target_node == 0 {
        None
    } else {
        Some(NodeId::new(event.target_node))
    };
    let mut core_event = CannedMessageAckEvent::with_sequence(
        event.message.into(),
        NodeId::new(event.source_node),
        target,
        event.timestamp,
        event.sequence,
    );

    // Add all ACKs (source ACK is automatic, others need to be added)
    for ack in &event.acks {
        if ack.node_id != event.source_node {
            core_event.ack(NodeId::new(ack.node_id), ack.timestamp);
        }
    }

    core_event.encode().to_vec()
}

/// Decode wire format bytes to a CannedMessageAckEvent.
///
/// Handles both base format (22 bytes) and extended format (24+ bytes with ACKs).
/// Returns None if data is malformed.
#[uniffi::export]
pub fn decode_canned_message_ack_event(data: Vec<u8>) -> Option<CannedMessageAckEventData> {
    CannedMessageAckEvent::decode(&data).map(|e| e.into())
}

/// Add an ACK to a CannedMessageAckEvent.
///
/// Returns the updated event with the new ACK added.
/// If the ACK already exists with a newer or equal timestamp, returns unchanged.
#[uniffi::export]
pub fn canned_message_ack_event_add_ack(
    event: CannedMessageAckEventData,
    acker_node_id: u32,
    ack_timestamp: u64,
) -> CannedMessageAckEventData {
    // Rebuild the core event
    let target = if event.target_node == 0 {
        None
    } else {
        Some(NodeId::new(event.target_node))
    };
    let mut core_event = CannedMessageAckEvent::with_sequence(
        event.message.into(),
        NodeId::new(event.source_node),
        target,
        event.timestamp,
        event.sequence,
    );

    // Re-add existing ACKs
    for ack in &event.acks {
        if ack.node_id != event.source_node {
            core_event.ack(NodeId::new(ack.node_id), ack.timestamp);
        }
    }

    // Add the new ACK
    core_event.ack(NodeId::new(acker_node_id), ack_timestamp);

    core_event.into()
}

/// Check if a node has ACKed this message.
#[uniffi::export]
pub fn canned_message_ack_event_has_acked(event: &CannedMessageAckEventData, node_id: u32) -> bool {
    event.acks.iter().any(|a| a.node_id == node_id)
}

/// Get the number of ACKs on this message.
#[uniffi::export]
pub fn canned_message_ack_event_ack_count(event: &CannedMessageAckEventData) -> u32 {
    event.acks.len() as u32
}

/// Get ACK timestamp for a specific node, or None if not ACKed.
#[uniffi::export]
pub fn canned_message_ack_event_get_ack_timestamp(
    event: &CannedMessageAckEventData,
    node_id: u32,
) -> Option<u64> {
    event
        .acks
        .iter()
        .find(|a| a.node_id == node_id)
        .map(|a| a.timestamp)
}

/// Merge two CannedMessageAckEvents using CRDT semantics.
///
/// - Same event (source + timestamp match): merge ACK maps with OR semantics
/// - Different event: higher timestamp wins (LWW)
///
/// Returns the merged event.
#[uniffi::export]
pub fn canned_message_ack_event_merge(
    event1: CannedMessageAckEventData,
    event2: CannedMessageAckEventData,
) -> CannedMessageAckEventData {
    // Convert both to core events
    let target1 = if event1.target_node == 0 {
        None
    } else {
        Some(NodeId::new(event1.target_node))
    };
    let mut core1 = CannedMessageAckEvent::with_sequence(
        event1.message.into(),
        NodeId::new(event1.source_node),
        target1,
        event1.timestamp,
        event1.sequence,
    );
    for ack in &event1.acks {
        if ack.node_id != event1.source_node {
            core1.ack(NodeId::new(ack.node_id), ack.timestamp);
        }
    }

    let target2 = if event2.target_node == 0 {
        None
    } else {
        Some(NodeId::new(event2.target_node))
    };
    let mut core2 = CannedMessageAckEvent::with_sequence(
        event2.message.into(),
        NodeId::new(event2.source_node),
        target2,
        event2.timestamp,
        event2.sequence,
    );
    for ack in &event2.acks {
        if ack.node_id != event2.source_node {
            core2.ack(NodeId::new(ack.node_id), ack.timestamp);
        }
    }

    // Merge
    core1.merge(&core2);
    core1.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let event = create_canned_message_event(
            CannedMessageType::Emergency,
            0x12345678,
            0xDEADBEEF,
            1706234567000,
            42,
        );

        let encoded = encode_canned_message_event(event.clone());
        assert_eq!(encoded.len(), 22);
        assert_eq!(encoded[0], CANNED_MESSAGE_MARKER);

        let decoded = decode_canned_message_event(encoded).unwrap();
        assert_eq!(decoded.message, event.message);
        assert_eq!(decoded.source_node, event.source_node);
        assert_eq!(decoded.target_node, event.target_node);
        assert_eq!(decoded.timestamp, event.timestamp);
        assert_eq!(decoded.sequence, event.sequence);
    }

    #[test]
    fn test_message_type_conversion() {
        for msg in [
            CannedMessageType::Ack,
            CannedMessageType::Emergency,
            CannedMessageType::CheckIn,
        ] {
            let code = canned_message_code(msg);
            let recovered = canned_message_from_code(code).unwrap();
            assert_eq!(msg, recovered);
        }
    }

    // ===== CannedMessageAckEvent tests =====

    #[test]
    fn test_ack_event_creation() {
        let event = create_canned_message_ack_event(
            CannedMessageType::CheckIn,
            0x12345678,
            0,
            1706234567000,
            0,
        );

        // Source should auto-ack
        assert!(canned_message_ack_event_has_acked(&event, 0x12345678));
        assert_eq!(canned_message_ack_event_ack_count(&event), 1);
    }

    #[test]
    fn test_ack_event_add_ack() {
        let event =
            create_canned_message_ack_event(CannedMessageType::Emergency, 0x111, 0, 1000, 0);

        // Add ACK from another node
        let event = canned_message_ack_event_add_ack(event, 0x222, 1500);

        assert!(canned_message_ack_event_has_acked(&event, 0x111));
        assert!(canned_message_ack_event_has_acked(&event, 0x222));
        assert_eq!(canned_message_ack_event_ack_count(&event), 2);
        assert_eq!(
            canned_message_ack_event_get_ack_timestamp(&event, 0x222),
            Some(1500)
        );
    }

    #[test]
    fn test_ack_event_roundtrip() {
        let event = create_canned_message_ack_event(
            CannedMessageType::Emergency,
            0x12345678,
            0xDEADBEEF,
            1706234567000,
            42,
        );
        let event = canned_message_ack_event_add_ack(event, 0xAAAA, 1706234568000);
        let event = canned_message_ack_event_add_ack(event, 0xBBBB, 1706234569000);

        let encoded = encode_canned_message_ack_event(event.clone());
        // 24 base + 3 ACKs * 12 = 60 bytes
        assert_eq!(encoded.len(), 24 + 3 * 12);
        assert_eq!(encoded[0], CANNED_MESSAGE_MARKER);

        let decoded = decode_canned_message_ack_event(encoded).unwrap();
        assert_eq!(decoded.message, event.message);
        assert_eq!(decoded.source_node, event.source_node);
        assert_eq!(decoded.target_node, event.target_node);
        assert_eq!(decoded.timestamp, event.timestamp);
        assert_eq!(decoded.sequence, event.sequence);
        assert_eq!(canned_message_ack_event_ack_count(&decoded), 3);
        assert!(canned_message_ack_event_has_acked(&decoded, 0x12345678));
        assert!(canned_message_ack_event_has_acked(&decoded, 0xAAAA));
        assert!(canned_message_ack_event_has_acked(&decoded, 0xBBBB));
    }

    #[test]
    fn test_ack_event_merge() {
        let source = 0x111u32;

        // Event 1: source + node_a acked
        let event1 =
            create_canned_message_ack_event(CannedMessageType::CheckIn, source, 0, 1000, 0);
        let event1 = canned_message_ack_event_add_ack(event1, 0x222, 1100);

        // Event 2 (same message): source + node_b acked
        let event2 =
            create_canned_message_ack_event(CannedMessageType::CheckIn, source, 0, 1000, 0);
        let event2 = canned_message_ack_event_add_ack(event2, 0x333, 1200);

        // Merge should combine ACKs (OR semantics)
        let merged = canned_message_ack_event_merge(event1, event2);
        assert!(canned_message_ack_event_has_acked(&merged, source));
        assert!(canned_message_ack_event_has_acked(&merged, 0x222));
        assert!(canned_message_ack_event_has_acked(&merged, 0x333));
        assert_eq!(canned_message_ack_event_ack_count(&merged), 3);
    }

    #[test]
    fn test_ack_event_decode_base_format() {
        // Create a basic CannedMessageEvent (22 bytes, no ACKs)
        let base_event = create_canned_message_event(
            CannedMessageType::CheckIn,
            0x12345678,
            0,
            1706234567000,
            5,
        );

        let encoded = encode_canned_message_event(base_event.clone());
        assert_eq!(encoded.len(), 22);

        // CannedMessageAckEvent should decode it with implicit source ACK
        let decoded = decode_canned_message_ack_event(encoded).unwrap();
        assert_eq!(decoded.message, base_event.message);
        assert_eq!(decoded.source_node, base_event.source_node);
        // Only source's implicit ACK
        assert_eq!(canned_message_ack_event_ack_count(&decoded), 1);
        assert!(canned_message_ack_event_has_acked(
            &decoded,
            base_event.source_node
        ));
    }
}
