// Copyright (c) 2025-2026 (r)evolve - Revolve Team LLC
// SPDX-License-Identifier: Apache-2.0

//! Android FFI crate for hive-lite.
//!
//! This crate provides UniFFI bindings for Android/Kotlin.
//! It wraps the core no_std types with std-based alternatives.

use hive_lite::canned::{CannedMessage, CannedMessageEvent};
use hive_lite::node_id::NodeId;
use hive_lite::wire::CANNED_MESSAGE_MARKER;

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
}
