//! Integration tests for TTL wire format across the full packet lifecycle.
//!
//! These tests replicate what the firmware's Message::data_with_ttl() and
//! handle_data() do, verifying the complete encode → wire → decode → strip_ttl
//! path using the protocol crate's public API.

use hive_lite_protocol::{
    append_ttl, decode_header, default_ttl_for_crdt, encode_header, strip_ttl, CrdtType, Header,
    MessageType, FLAG_HAS_TTL, HEADER_SIZE, MAX_PACKET_SIZE, TTL_NEVER_EXPIRES,
};

/// Simulate Message::data_with_ttl() — encode a full Data packet with TTL.
///
/// Layout: [Header:16][CrdtType:1][CrdtData:N][TTL_LE_U32:4]
fn encode_data_with_ttl(
    node_id: u32,
    seq_num: u32,
    crdt_type: CrdtType,
    crdt_data: &[u8],
    ttl_seconds: u32,
    buf: &mut [u8],
) -> usize {
    let header = Header {
        msg_type: MessageType::Data,
        flags: FLAG_HAS_TTL,
        node_id,
        seq_num,
    };
    encode_header(&header, buf).unwrap();

    // Payload: crdt_type byte + crdt_data + TTL suffix
    let payload_start = HEADER_SIZE;
    buf[payload_start] = crdt_type as u8;
    let crdt_offset = payload_start + 1;
    buf[crdt_offset..crdt_offset + crdt_data.len()].copy_from_slice(crdt_data);

    let payload_len = 1 + crdt_data.len();
    let new_payload_len = append_ttl(&mut buf[payload_start..], payload_len, ttl_seconds).unwrap();

    HEADER_SIZE + new_payload_len
}

/// Simulate Message::data() — encode a Data packet WITHOUT TTL.
fn encode_data_without_ttl(
    node_id: u32,
    seq_num: u32,
    crdt_type: CrdtType,
    crdt_data: &[u8],
    buf: &mut [u8],
) -> usize {
    let header = Header {
        msg_type: MessageType::Data,
        flags: 0,
        node_id,
        seq_num,
    };
    encode_header(&header, buf).unwrap();

    let payload_start = HEADER_SIZE;
    buf[payload_start] = crdt_type as u8;
    let crdt_offset = payload_start + 1;
    buf[crdt_offset..crdt_offset + crdt_data.len()].copy_from_slice(crdt_data);

    HEADER_SIZE + 1 + crdt_data.len()
}

/// Simulate handle_data() — decode header, strip TTL, extract CRDT type.
///
/// Returns (crdt_type, crdt_data_slice, ttl_seconds).
fn decode_data_message(buf: &[u8]) -> (CrdtType, Vec<u8>, u32) {
    let (header, payload) = decode_header(buf).unwrap();
    assert_eq!(header.msg_type, MessageType::Data);

    let (crdt_payload, ttl_seconds) = strip_ttl(header.flags, payload);
    assert!(!crdt_payload.is_empty(), "CRDT payload should not be empty");

    let crdt_type = CrdtType::from_u8(crdt_payload[0]).unwrap();
    let crdt_data = crdt_payload[1..].to_vec();

    (crdt_type, crdt_data, ttl_seconds)
}

// === Full packet roundtrip tests ===

#[test]
fn test_data_with_ttl_roundtrip_lww_register() {
    let crdt_data = [0x01, 0x02, 0x03, 0x04]; // Simulated LWW register bytes
    let mut buf = [0u8; MAX_PACKET_SIZE];

    let len = encode_data_with_ttl(
        0xDEADBEEF,
        42,
        CrdtType::LwwRegister,
        &crdt_data,
        300, // 5 min TTL
        &mut buf,
    );

    // Verify packet structure
    assert_eq!(len, HEADER_SIZE + 1 + 4 + 4); // header + crdt_type + data + ttl

    // Decode and verify
    let (crdt_type, decoded_data, ttl) = decode_data_message(&buf[..len]);
    assert_eq!(crdt_type, CrdtType::LwwRegister);
    assert_eq!(decoded_data, crdt_data);
    assert_eq!(ttl, 300);
}

#[test]
fn test_data_with_ttl_roundtrip_gcounter() {
    // GCounter payload: 4 bytes node_id + 4 bytes count (simulated)
    let crdt_data = [0x78, 0x56, 0x34, 0x12, 0x0A, 0x00, 0x00, 0x00];
    let mut buf = [0u8; MAX_PACKET_SIZE];

    let len = encode_data_with_ttl(
        0x12345678,
        100,
        CrdtType::GCounter,
        &crdt_data,
        3600, // 1 hour
        &mut buf,
    );

    let (crdt_type, decoded_data, ttl) = decode_data_message(&buf[..len]);
    assert_eq!(crdt_type, CrdtType::GCounter);
    assert_eq!(decoded_data, crdt_data);
    assert_eq!(ttl, 3600);
}

#[test]
fn test_data_with_ttl_roundtrip_pncounter() {
    let crdt_data = [0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00];
    let mut buf = [0u8; MAX_PACKET_SIZE];

    let len = encode_data_with_ttl(1, 1, CrdtType::PnCounter, &crdt_data, 3600, &mut buf);

    let (crdt_type, decoded_data, ttl) = decode_data_message(&buf[..len]);
    assert_eq!(crdt_type, CrdtType::PnCounter);
    assert_eq!(decoded_data, crdt_data);
    assert_eq!(ttl, 3600);
}

#[test]
fn test_data_with_ttl_zero_means_never_expires() {
    let crdt_data = [0xAA, 0xBB];
    let mut buf = [0u8; MAX_PACKET_SIZE];

    let len = encode_data_with_ttl(
        1,
        1,
        CrdtType::LwwRegister,
        &crdt_data,
        0, // never expires
        &mut buf,
    );

    let (_, _, ttl) = decode_data_message(&buf[..len]);
    assert_eq!(ttl, TTL_NEVER_EXPIRES);
}

// === Backward compatibility ===

#[test]
fn test_data_without_ttl_backward_compatible() {
    let crdt_data = [0x01, 0x02, 0x03, 0x04];
    let mut buf = [0u8; MAX_PACKET_SIZE];

    // Old-style message without TTL flag
    let len = encode_data_without_ttl(0xDEADBEEF, 42, CrdtType::LwwRegister, &crdt_data, &mut buf);

    // Decode with TTL-aware decoder — should get full data and TTL_NEVER_EXPIRES
    let (crdt_type, decoded_data, ttl) = decode_data_message(&buf[..len]);
    assert_eq!(crdt_type, CrdtType::LwwRegister);
    assert_eq!(decoded_data, crdt_data);
    assert_eq!(ttl, TTL_NEVER_EXPIRES);
}

#[test]
fn test_old_decoder_ignores_ttl_flag() {
    let crdt_data = [0x01, 0x02, 0x03];
    let mut buf = [0u8; MAX_PACKET_SIZE];

    let len = encode_data_with_ttl(1, 1, CrdtType::LwwRegister, &crdt_data, 300, &mut buf);

    // Simulate old decoder: decode header, ignore flags, read payload as-is
    let (header, payload) = decode_header(&buf[..len]).unwrap();
    assert_eq!(header.msg_type, MessageType::Data);

    // Old decoder treats entire payload as CRDT data (crdt_type + data + TTL bytes)
    // This is safe because LwwRegister has value_len for self-delimiting
    assert_eq!(payload.len(), 1 + 3 + 4); // crdt_type + data + TTL suffix
    assert_eq!(payload[0], CrdtType::LwwRegister as u8);
}

// === Header flag preservation ===

#[test]
fn test_flag_has_ttl_survives_encode_decode() {
    let header = Header {
        msg_type: MessageType::Data,
        flags: FLAG_HAS_TTL,
        node_id: 42,
        seq_num: 1,
    };
    let mut buf = [0u8; MAX_PACKET_SIZE];
    encode_header(&header, &mut buf).unwrap();
    buf[HEADER_SIZE] = CrdtType::GCounter as u8; // minimal payload

    let (decoded, _) = decode_header(&buf[..HEADER_SIZE + 1]).unwrap();
    assert_eq!(decoded.flags & FLAG_HAS_TTL, FLAG_HAS_TTL);
}

#[test]
fn test_flag_has_ttl_combined_with_other_flags() {
    // FLAG_HAS_TTL can coexist with other flags
    let header = Header {
        msg_type: MessageType::Data,
        flags: FLAG_HAS_TTL | 0x0080, // some hypothetical other flag
        node_id: 42,
        seq_num: 1,
    };
    let mut buf = [0u8; MAX_PACKET_SIZE];
    encode_header(&header, &mut buf).unwrap();

    let (decoded, _) = decode_header(&buf[..HEADER_SIZE]).unwrap();
    assert_eq!(decoded.flags & FLAG_HAS_TTL, FLAG_HAS_TTL);
    assert_eq!(decoded.flags & 0x0080, 0x0080);
}

// === Default TTL integration ===

#[test]
fn test_default_ttl_used_in_packet() {
    // Verify that encoding with default TTL values and decoding produces
    // the expected defaults
    for crdt_type in [
        CrdtType::LwwRegister,
        CrdtType::GCounter,
        CrdtType::PnCounter,
    ] {
        let default_ttl = default_ttl_for_crdt(crdt_type);
        let crdt_data = [0x01];
        let mut buf = [0u8; MAX_PACKET_SIZE];

        let len = encode_data_with_ttl(1, 1, crdt_type, &crdt_data, default_ttl, &mut buf);

        let (decoded_type, _, decoded_ttl) = decode_data_message(&buf[..len]);
        assert_eq!(decoded_type, crdt_type);
        assert_eq!(decoded_ttl, default_ttl);
    }
}

// === Edge cases ===

#[test]
fn test_max_size_payload_with_ttl() {
    // Maximum payload = MAX_PACKET_SIZE - HEADER_SIZE = 496 bytes
    // With TTL: crdt_type(1) + data(487) + ttl(4) = 492 < 496
    let crdt_data = [0xAA; 487];
    let mut buf = [0u8; MAX_PACKET_SIZE];

    let len = encode_data_with_ttl(1, 1, CrdtType::LwwRegister, &crdt_data, 300, &mut buf);

    let (crdt_type, decoded_data, ttl) = decode_data_message(&buf[..len]);
    assert_eq!(crdt_type, CrdtType::LwwRegister);
    assert_eq!(decoded_data.len(), 487);
    assert_eq!(ttl, 300);
}

#[test]
fn test_single_byte_crdt_data_with_ttl() {
    let crdt_data = [0x42]; // Minimal CRDT data
    let mut buf = [0u8; MAX_PACKET_SIZE];

    let len = encode_data_with_ttl(1, 1, CrdtType::LwwRegister, &crdt_data, 300, &mut buf);

    assert_eq!(len, HEADER_SIZE + 1 + 1 + 4); // header + type + 1 byte + ttl

    let (_, decoded_data, ttl) = decode_data_message(&buf[..len]);
    assert_eq!(decoded_data, &[0x42]);
    assert_eq!(ttl, 300);
}

#[test]
fn test_large_ttl_value() {
    let crdt_data = [0x01];
    let mut buf = [0u8; MAX_PACKET_SIZE];

    // u32::MAX = ~136 years
    let len = encode_data_with_ttl(1, 1, CrdtType::LwwRegister, &crdt_data, u32::MAX, &mut buf);

    let (_, _, ttl) = decode_data_message(&buf[..len]);
    assert_eq!(ttl, u32::MAX);
}
