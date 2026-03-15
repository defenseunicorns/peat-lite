// Copyright (c) 2025-2026 Defense Unicorns, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Wire protocol edge-case tests.
//!
//! Verify that decode functions return errors (or `None`) on malformed input
//! rather than panicking.

use peat_lite::protocol::constants::{HEADER_SIZE, MAGIC, MAX_PACKET_SIZE, PROTOCOL_VERSION};
use peat_lite::protocol::header::{decode_header, encode_header, Header};
use peat_lite::protocol::{MessageError, MessageType};
use peat_lite::{
    CannedMessage, CannedMessageAckEvent, CannedMessageEvent, GCounter, NodeId,
    CANNED_MESSAGE_MARKER, MAX_CANNED_ACKS,
};

// ---------------------------------------------------------------------------
// Helper: build a valid 16-byte header buffer
// ---------------------------------------------------------------------------
fn valid_header_buf(msg_type: MessageType, node_id: u32, seq: u32) -> [u8; HEADER_SIZE] {
    let hdr = Header {
        msg_type,
        flags: 0,
        node_id,
        seq_num: seq,
    };
    let mut buf = [0u8; HEADER_SIZE];
    encode_header(&hdr, &mut buf).unwrap();
    buf
}

// ===== Truncated packets =====

#[test]
fn decode_header_single_byte() {
    assert_eq!(decode_header(&[0x50]), Err(MessageError::TooShort));
}

#[test]
fn decode_header_empty() {
    assert_eq!(decode_header(&[]), Err(MessageError::TooShort));
}

#[test]
fn decode_header_exactly_header_size_is_ok() {
    let buf = valid_header_buf(MessageType::Heartbeat, 1, 0);
    let (hdr, payload) = decode_header(&buf).unwrap();
    assert_eq!(hdr.msg_type, MessageType::Heartbeat);
    assert!(payload.is_empty());
}

#[test]
fn decode_header_fifteen_bytes() {
    let buf = valid_header_buf(MessageType::Data, 1, 0);
    assert_eq!(decode_header(&buf[..15]), Err(MessageError::TooShort));
}

// ===== Invalid magic bytes =====

#[test]
fn decode_header_wrong_magic_all_zeroes() {
    let mut buf = [0u8; HEADER_SIZE];
    buf[4] = PROTOCOL_VERSION;
    buf[5] = MessageType::Announce as u8;
    assert_eq!(decode_header(&buf), Err(MessageError::InvalidMagic));
}

#[test]
fn decode_header_wrong_magic_off_by_one() {
    let mut buf = valid_header_buf(MessageType::Announce, 1, 0);
    buf[0] = MAGIC[0].wrapping_add(1); // corrupt first magic byte
    assert_eq!(decode_header(&buf), Err(MessageError::InvalidMagic));
}

#[test]
fn decode_header_wrong_magic_reversed() {
    let mut buf = valid_header_buf(MessageType::Announce, 1, 0);
    buf[0..4].copy_from_slice(&[MAGIC[3], MAGIC[2], MAGIC[1], MAGIC[0]]);
    assert_eq!(decode_header(&buf), Err(MessageError::InvalidMagic));
}

// ===== Unsupported version numbers =====

#[test]
fn decode_header_version_zero() {
    let mut buf = valid_header_buf(MessageType::Heartbeat, 1, 0);
    buf[4] = 0;
    assert_eq!(decode_header(&buf), Err(MessageError::UnsupportedVersion));
}

#[test]
fn decode_header_version_255() {
    let mut buf = valid_header_buf(MessageType::Heartbeat, 1, 0);
    buf[4] = 255;
    assert_eq!(decode_header(&buf), Err(MessageError::UnsupportedVersion));
}

#[test]
fn decode_header_version_next() {
    let mut buf = valid_header_buf(MessageType::Heartbeat, 1, 0);
    buf[4] = PROTOCOL_VERSION + 1;
    assert_eq!(decode_header(&buf), Err(MessageError::UnsupportedVersion));
}

// ===== Invalid message type =====

#[test]
fn decode_header_invalid_message_type_0xff() {
    let mut buf = valid_header_buf(MessageType::Heartbeat, 1, 0);
    buf[5] = 0xFF;
    assert_eq!(decode_header(&buf), Err(MessageError::InvalidMessageType));
}

#[test]
fn decode_header_invalid_message_type_0x00() {
    let mut buf = valid_header_buf(MessageType::Heartbeat, 1, 0);
    buf[5] = 0x00;
    assert_eq!(decode_header(&buf), Err(MessageError::InvalidMessageType));
}

// ===== Maximum-size payloads =====

#[test]
fn decode_header_max_packet_size_payload() {
    let mut buf = vec![0u8; MAX_PACKET_SIZE];
    let hdr = Header {
        msg_type: MessageType::Data,
        flags: 0,
        node_id: 42,
        seq_num: 1,
    };
    encode_header(&hdr, &mut buf).unwrap();
    // Fill payload area with non-zero data
    for b in &mut buf[HEADER_SIZE..] {
        *b = 0xCC;
    }
    let (decoded, payload) = decode_header(&buf).unwrap();
    assert_eq!(decoded.msg_type, MessageType::Data);
    assert_eq!(payload.len(), MAX_PACKET_SIZE - HEADER_SIZE);
    assert!(payload.iter().all(|&b| b == 0xCC));
}

// ===== Zero-length payloads =====

#[test]
fn decode_header_zero_length_payload() {
    let buf = valid_header_buf(MessageType::Leave, 99, 7);
    let (hdr, payload) = decode_header(&buf).unwrap();
    assert_eq!(hdr.node_id, 99);
    assert_eq!(hdr.seq_num, 7);
    assert!(payload.is_empty());
}

// ===== CannedMessageEvent edge cases =====

#[test]
fn canned_event_decode_empty() {
    assert!(CannedMessageEvent::decode(&[]).is_none());
}

#[test]
fn canned_event_decode_single_byte() {
    assert!(CannedMessageEvent::decode(&[CANNED_MESSAGE_MARKER]).is_none());
}

#[test]
fn canned_event_decode_21_bytes() {
    // One byte short of the minimum 22-byte unsigned format
    let mut buf = [0u8; 21];
    buf[0] = CANNED_MESSAGE_MARKER;
    buf[1] = CannedMessage::Ack as u8;
    assert!(CannedMessageEvent::decode(&buf).is_none());
}

#[test]
fn canned_event_decode_wrong_marker() {
    let event = CannedMessageEvent::new(CannedMessage::CheckIn, NodeId::new(1), None, 1000);
    let mut encoded = event.encode();
    encoded[0] = 0x00; // corrupt marker
    assert!(CannedMessageEvent::decode(&encoded).is_none());
}

#[test]
fn canned_event_decode_invalid_message_code() {
    let event = CannedMessageEvent::new(CannedMessage::Ack, NodeId::new(1), None, 1000);
    let mut encoded = event.encode();
    encoded[1] = 0x99; // undefined canned message code
    assert!(CannedMessageEvent::decode(&encoded).is_none());
}

#[test]
fn canned_event_roundtrip_zero_length_payload_fields() {
    // No target, zero timestamp, zero sequence
    let event = CannedMessageEvent::with_sequence(CannedMessage::Ack, NodeId::new(1), None, 0, 0);
    let encoded = event.encode();
    let decoded = CannedMessageEvent::decode(&encoded).unwrap();
    assert_eq!(decoded, event);
}

// ===== CannedMessageAckEvent malformed data =====

#[test]
fn ack_event_decode_num_acks_exceeds_max() {
    let event = CannedMessageAckEvent::new(CannedMessage::Ack, NodeId::new(1), None, 1000);
    let mut encoded: Vec<u8> = event.encode().to_vec();

    // Overwrite num_acks field (bytes 22-23) to exceed MAX_CANNED_ACKS
    let bogus_count = (MAX_CANNED_ACKS as u16) + 1;
    encoded[22..24].copy_from_slice(&bogus_count.to_le_bytes());

    // Pad buffer so it's long enough for the claimed count
    let needed = 24 + (bogus_count as usize) * 12;
    encoded.resize(needed, 0);

    assert!(CannedMessageAckEvent::decode(&encoded).is_none());
}

#[test]
fn ack_event_decode_num_acks_buffer_too_short() {
    // Encode a valid event, then claim more ACKs than the buffer can hold
    let event = CannedMessageAckEvent::new(CannedMessage::Ack, NodeId::new(1), None, 1000);
    let mut encoded: Vec<u8> = event.encode().to_vec();

    // Set num_acks to 5 but don't provide enough trailing bytes
    encoded[22..24].copy_from_slice(&5u16.to_le_bytes());
    // Buffer is only 24 + 12 = 36 bytes (has 1 ack entry), not 24 + 60
    assert!(CannedMessageAckEvent::decode(&encoded).is_none());
}

#[test]
fn ack_event_decode_null_source_rejected() {
    // Build raw wire bytes with source_node = 0 (NULL)
    let mut buf = vec![0u8; 24];
    buf[0] = CANNED_MESSAGE_MARKER;
    buf[1] = CannedMessage::Ack as u8;
    // source_node bytes [2..6] = 0 (NULL)
    // num_acks = 0
    assert!(CannedMessageAckEvent::decode(&buf).is_none());
}

// ===== GCounter corrupted / truncated encoding =====

#[test]
fn gcounter_decode_empty() {
    assert!(GCounter::<8>::decode(&[]).is_none());
}

#[test]
fn gcounter_decode_single_byte() {
    assert!(GCounter::<8>::decode(&[0x01]).is_none());
}

#[test]
fn gcounter_decode_header_only_no_entries() {
    // num_entries = 0 encoded as 2 LE bytes
    let data = [0x00, 0x00];
    let counter = GCounter::<8>::decode(&data).unwrap();
    assert_eq!(counter.value(), 0);
}

#[test]
fn gcounter_decode_claims_entries_but_truncated() {
    // Header says 3 entries but only 1 entry of data follows (8 bytes)
    let mut data = vec![0u8; 10]; // 2 header + 8 data = room for 1 entry
    data[0..2].copy_from_slice(&3u16.to_le_bytes()); // claims 3 entries
                                                     // Fill one entry with valid data
    data[2..6].copy_from_slice(&1u32.to_le_bytes()); // node_id
    data[6..10].copy_from_slice(&42u32.to_le_bytes()); // count
    assert!(GCounter::<8>::decode(&data).is_none());
}

#[test]
fn gcounter_decode_zero_count_entry() {
    // Build manually: 1 entry, node_id = 5, count = 0
    let mut data = vec![0u8; 10];
    data[0..2].copy_from_slice(&1u16.to_le_bytes());
    data[2..6].copy_from_slice(&5u32.to_le_bytes());
    data[6..10].copy_from_slice(&0u32.to_le_bytes());
    let decoded = GCounter::<8>::decode(&data).unwrap();
    assert_eq!(decoded.node_count(NodeId::new(5)), 0);
}

#[test]
fn gcounter_encode_decode_roundtrip_max_entries() {
    let mut counter = GCounter::<32>::new();
    for i in 1..=32u32 {
        counter.increment(NodeId::new(i), i * 100);
    }
    let encoded = counter.encode();
    let decoded = GCounter::<32>::decode(&encoded).unwrap();
    assert_eq!(counter, decoded);
}

#[test]
fn gcounter_decode_extra_trailing_bytes_ignored() {
    // Encode a valid counter, then append garbage
    let mut counter = GCounter::<8>::new();
    counter.increment(NodeId::new(1), 10);
    let mut data: Vec<u8> = counter.encode().to_vec();
    data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // trailing garbage
                                                       // Should still decode the valid portion
    let decoded = GCounter::<8>::decode(&data).unwrap();
    assert_eq!(decoded.node_count(NodeId::new(1)), 10);
}

// ===== NodeId boundary values =====

#[test]
fn node_id_zero_boundary() {
    let node = NodeId::new(0);
    assert!(node.is_null());
    assert_eq!(node.as_u32(), 0);
    let bytes = node.to_le_bytes();
    assert_eq!(bytes, [0, 0, 0, 0]);
    assert_eq!(NodeId::from_le_bytes(bytes), node);
}

#[test]
fn node_id_max_boundary() {
    let node = NodeId::new(u32::MAX);
    assert!(!node.is_null());
    assert_eq!(node.as_u32(), u32::MAX);
    let bytes = node.to_le_bytes();
    assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(NodeId::from_le_bytes(bytes), node);
}

#[test]
fn canned_event_with_node_id_max() {
    let event = CannedMessageEvent::new(
        CannedMessage::Emergency,
        NodeId::new(u32::MAX),
        Some(NodeId::new(u32::MAX)),
        u64::MAX,
    );
    let encoded = event.encode();
    let decoded = CannedMessageEvent::decode(&encoded).unwrap();
    assert_eq!(decoded.source_node, NodeId::new(u32::MAX));
    assert_eq!(decoded.target_node, Some(NodeId::new(u32::MAX)));
    assert_eq!(decoded.timestamp, u64::MAX);
}

#[test]
fn header_with_node_id_zero() {
    let buf = valid_header_buf(MessageType::Heartbeat, 0, 0);
    let (hdr, _) = decode_header(&buf).unwrap();
    assert_eq!(hdr.node_id, 0);
}

#[test]
fn header_with_node_id_max() {
    let buf = valid_header_buf(MessageType::Data, u32::MAX, u32::MAX);
    let (hdr, _) = decode_header(&buf).unwrap();
    assert_eq!(hdr.node_id, u32::MAX);
    assert_eq!(hdr.seq_num, u32::MAX);
}

#[test]
fn gcounter_with_node_id_zero() {
    let mut counter = GCounter::<8>::new();
    counter.increment(NodeId::new(0), 42);
    let encoded = counter.encode();
    let decoded = GCounter::<8>::decode(&encoded).unwrap();
    assert_eq!(decoded.node_count(NodeId::new(0)), 42);
}

#[test]
fn gcounter_with_node_id_max() {
    let mut counter = GCounter::<8>::new();
    counter.increment(NodeId::new(u32::MAX), 999);
    let encoded = counter.encode();
    let decoded = GCounter::<8>::decode(&encoded).unwrap();
    assert_eq!(decoded.node_count(NodeId::new(u32::MAX)), 999);
}
