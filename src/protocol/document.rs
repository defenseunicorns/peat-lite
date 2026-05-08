// Copyright (c) 2025-2026 Defense Unicorns, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Universal Document envelope for cross-transport sync.
//!
//! Carries an opaque document body (collection name + id + timestamp +
//! body bytes) so adding a new collection to the network requires zero
//! changes to peat-lite, peat-btle, or future LoRa transports — only
//! the publisher and the consumer agree on the body shape. This
//! formalizes the universal half of ADR-059 Amendment 4's
//! universal-vs-application-domain split: typed sensor primitives
//! (peripheral health, BLE position, canned messages) keep their
//! domain-specific carriers in peat-btle / peat-protocol; arbitrary
//! Documents (markers, platforms, tracks, future collections) flow
//! through this envelope.
//!
//! ## Wire layout
//!
//! Carried as the payload of a [`MessageType::Document`] frame
//! (header byte 0x07). The header (16 bytes, [`super::header`]) is
//! followed immediately by:
//!
//! ```text
//! ┌──────────────┬────────────────┬───────────────┬────────────┬──────────────┬──────┐
//! │    flags     │ collection_len │  collection   │  doc_id_len│   doc_id     │ ...  │
//! │   1 byte     │    1 byte      │  N bytes      │  2 bytes LE│  M bytes     │      │
//! ├──────────────┼────────────────┼───────────────┴────────────┴──────────────┘      │
//! │ timestamp_ms │   body_len     │             body                                 │
//! │  8 bytes LE  │  2 bytes LE    │           K bytes (opaque)                       │
//! └──────────────┴────────────────┴──────────────────────────────────────────────────┘
//! ```
//!
//! - `flags`: bit 0 = deletion-tombstone (body MAY be empty);
//!   bit 1 = encrypted body; bits 2–7 reserved (must encode 0).
//! - `collection`: UTF-8, length-prefixed by 1 byte (0–255). Empty
//!   collection name is invalid and rejected on decode.
//! - `doc_id`: UTF-8, length-prefixed by 2-byte LE length (0–65535).
//!   `doc_id_len = 0` means the publisher delegates id assignment to
//!   the receiving doc store (matching `peat_mesh::Node::publish`'s
//!   contract for Documents with `id: None`).
//! - `timestamp_ms`: Unix epoch milliseconds, 8-byte LE i64. Used by
//!   the receiving CRDT layer for last-writer-wins resolution where
//!   applicable; lower layers don't interpret it.
//! - `body`: opaque bytes, length-prefixed by 2-byte LE length
//!   (0–65535). peat-lite does not interpret the body; consumers
//!   (peat-mesh, peat-atak-plugin, M5Stack firmware) own the body
//!   schema. Postcard-encoded `peat_mesh::Document.fields` is the
//!   conventional choice on the host side, but the wire is agnostic.
//!
//! ## Size limits and fragmentation
//!
//! Field maxima:
//!
//! - `collection`: 255 bytes — comfortably above all current
//!   collection names (`markers`, `platforms`, `tracks`,
//!   `company_summaries`, `canned_messages`, …).
//! - `doc_id`: 65535 bytes — UUIDs (36) and CoT UIDs (~64) fit easily.
//! - `body`: 65535 bytes — well above peat-lite's
//!   [`MAX_PAYLOAD_SIZE`] (496 bytes). For LoRa-class transports
//!   carrying envelopes that fit in a single packet, the spec is
//!   bounded by the transport, not the codec.
//!
//! Total framed envelope on the wire:
//! `header(16) + 1 + 1 + N + 2 + M + 8 + 2 + K = 30 + N + M + K`
//!
//! When the framed envelope exceeds `MAX_PAYLOAD_SIZE`, the
//! **transport** is responsible for fragmenting (peat-btle's
//! `chunk_data` / `ChunkReassembler` over GATT, LoRa's scheduler,
//! etc.). peat-lite envelopes stay opaque to the chunking layer; this
//! keeps the codec transport-agnostic and lets each radio choose the
//! fragmentation strategy that fits its MTU + duty-cycle constraints.

use super::error::MessageError;

/// Bit 0: deletion tombstone — the publisher is deleting the document
/// referenced by `(collection, doc_id)`. Body MAY be empty; receivers
/// MUST tolerate `body_len = 0` when this flag is set.
pub const DOC_FLAG_TOMBSTONE: u8 = 0x01;
/// Bit 1: body is encrypted (per-document, key established
/// out-of-band). Reserved for future use; current encoders must NOT
/// set this flag and current decoders MUST treat it as opaque
/// pass-through.
pub const DOC_FLAG_ENCRYPTED: u8 = 0x02;

/// Maximum length of a collection name on the wire (1-byte length
/// prefix limit).
pub const MAX_COLLECTION_LEN: usize = u8::MAX as usize;
/// Maximum length of a doc id on the wire (2-byte length prefix
/// limit).
pub const MAX_DOC_ID_LEN: usize = u16::MAX as usize;
/// Maximum body byte length on the wire (2-byte length prefix
/// limit). Transport-level fragmentation may carry envelopes larger
/// than peat-lite's [`MAX_PAYLOAD_SIZE`], up to this hard cap.
pub const MAX_BODY_LEN: usize = u16::MAX as usize;

/// Decoded view of a Document envelope, borrowing from the input
/// buffer. Hot-path-friendly for `no_std` consumers — no allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentRef<'a> {
    /// Flags byte. See [`DOC_FLAG_TOMBSTONE`] / [`DOC_FLAG_ENCRYPTED`].
    pub flags: u8,
    /// Collection name as UTF-8 bytes (validated UTF-8 on decode).
    pub collection: &'a str,
    /// Document id as UTF-8 bytes; empty slice means "publisher
    /// delegated id assignment to the receiver."
    pub doc_id: &'a str,
    /// Unix epoch milliseconds.
    pub timestamp_ms: i64,
    /// Opaque body bytes — peat-lite does not interpret.
    pub body: &'a [u8],
}

impl<'a> DocumentRef<'a> {
    /// Returns true if the deletion-tombstone flag is set.
    #[inline]
    pub fn is_tombstone(&self) -> bool {
        self.flags & DOC_FLAG_TOMBSTONE != 0
    }

    /// Returns true if the encrypted-body flag is set.
    #[inline]
    pub fn is_encrypted(&self) -> bool {
        self.flags & DOC_FLAG_ENCRYPTED != 0
    }
}

/// Encoded length of a Document envelope with the given field sizes,
/// excluding the 16-byte peat-lite header. Useful for callers
/// preflight-checking a buffer against [`MAX_PAYLOAD_SIZE`] before
/// committing to a single-packet send vs. fragmentation.
#[inline]
pub const fn encoded_len(collection_len: usize, doc_id_len: usize, body_len: usize) -> usize {
    // flags(1) + collection_len(1) + collection(N)
    //        + doc_id_len(2) + doc_id(M)
    //        + timestamp_ms(8)
    //        + body_len(2) + body(K)
    1 + 1 + collection_len + 2 + doc_id_len + 8 + 2 + body_len
}

/// Encode a Document envelope into `buf`. Returns the number of bytes
/// written, or an error if any field exceeds its width or the buffer
/// is too small.
///
/// `buf` is the payload region — callers should already have written
/// the 16-byte peat-lite header (with [`MessageType::Document`]) into
/// the preceding bytes via [`super::header::encode_header`].
pub fn encode(
    flags: u8,
    collection: &str,
    doc_id: &str,
    timestamp_ms: i64,
    body: &[u8],
    buf: &mut [u8],
) -> Result<usize, MessageError> {
    let coll_bytes = collection.as_bytes();
    let id_bytes = doc_id.as_bytes();

    if coll_bytes.is_empty() {
        // Empty collection is structurally invalid — receivers can't
        // route a Document with no collection. Reject at encode time
        // so the bug doesn't reach the wire.
        return Err(MessageError::FieldTooLarge);
    }
    if coll_bytes.len() > MAX_COLLECTION_LEN
        || id_bytes.len() > MAX_DOC_ID_LEN
        || body.len() > MAX_BODY_LEN
    {
        return Err(MessageError::FieldTooLarge);
    }

    let needed = encoded_len(coll_bytes.len(), id_bytes.len(), body.len());
    if buf.len() < needed {
        return Err(MessageError::BufferTooSmall);
    }

    let mut o = 0;
    buf[o] = flags;
    o += 1;
    buf[o] = coll_bytes.len() as u8;
    o += 1;
    buf[o..o + coll_bytes.len()].copy_from_slice(coll_bytes);
    o += coll_bytes.len();
    buf[o..o + 2].copy_from_slice(&(id_bytes.len() as u16).to_le_bytes());
    o += 2;
    buf[o..o + id_bytes.len()].copy_from_slice(id_bytes);
    o += id_bytes.len();
    buf[o..o + 8].copy_from_slice(&timestamp_ms.to_le_bytes());
    o += 8;
    buf[o..o + 2].copy_from_slice(&(body.len() as u16).to_le_bytes());
    o += 2;
    buf[o..o + body.len()].copy_from_slice(body);
    o += body.len();

    debug_assert_eq!(o, needed);
    Ok(o)
}

/// Decode a Document envelope from `buf` (the payload region after
/// the 16-byte peat-lite header). Returns a borrowing view; bumps any
/// length-related parse error to a single error variant rather than
/// returning partial data, matching how the rest of the protocol
/// module surfaces malformed wire input.
pub fn decode(buf: &[u8]) -> Result<DocumentRef<'_>, MessageError> {
    let mut o = 0;
    if buf.len() < 1 + 1 {
        return Err(MessageError::TooShort);
    }
    let flags = buf[o];
    o += 1;
    let coll_len = buf[o] as usize;
    o += 1;
    if buf.len() < o + coll_len {
        return Err(MessageError::TruncatedField);
    }
    let collection =
        core::str::from_utf8(&buf[o..o + coll_len]).map_err(|_| MessageError::InvalidUtf8)?;
    if collection.is_empty() {
        // Mirror the encode-time rejection: an empty collection is
        // structurally invalid input from a non-conforming sender.
        return Err(MessageError::TruncatedField);
    }
    o += coll_len;

    if buf.len() < o + 2 {
        return Err(MessageError::TruncatedField);
    }
    let id_len = u16::from_le_bytes([buf[o], buf[o + 1]]) as usize;
    o += 2;
    if buf.len() < o + id_len {
        return Err(MessageError::TruncatedField);
    }
    let doc_id =
        core::str::from_utf8(&buf[o..o + id_len]).map_err(|_| MessageError::InvalidUtf8)?;
    o += id_len;

    if buf.len() < o + 8 {
        return Err(MessageError::TruncatedField);
    }
    let mut ts = [0u8; 8];
    ts.copy_from_slice(&buf[o..o + 8]);
    let timestamp_ms = i64::from_le_bytes(ts);
    o += 8;

    if buf.len() < o + 2 {
        return Err(MessageError::TruncatedField);
    }
    let body_len = u16::from_le_bytes([buf[o], buf[o + 1]]) as usize;
    o += 2;
    if buf.len() < o + body_len {
        return Err(MessageError::TruncatedField);
    }
    let body = &buf[o..o + body_len];

    Ok(DocumentRef {
        flags,
        collection,
        doc_id,
        timestamp_ms,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip a Document envelope with non-empty collection +
    /// doc_id + body and verify every field matches.
    #[test]
    fn roundtrip_full() {
        let mut buf = [0u8; 512];
        let body = b"opaque-body-bytes";
        let n = encode(
            0,
            "markers",
            "uuid-abc-123",
            1_700_000_000_000,
            body,
            &mut buf,
        )
        .expect("encode");
        let view = decode(&buf[..n]).expect("decode");
        assert_eq!(view.flags, 0);
        assert_eq!(view.collection, "markers");
        assert_eq!(view.doc_id, "uuid-abc-123");
        assert_eq!(view.timestamp_ms, 1_700_000_000_000);
        assert_eq!(view.body, body);
        assert!(!view.is_tombstone());
        assert!(!view.is_encrypted());
    }

    /// Empty doc_id signals "backend-assigned id" per spec — must
    /// round-trip cleanly, not be conflated with malformed input.
    #[test]
    fn roundtrip_empty_doc_id() {
        let mut buf = [0u8; 256];
        let n = encode(0, "markers", "", 0, b"x", &mut buf).expect("encode");
        let view = decode(&buf[..n]).expect("decode");
        assert_eq!(view.doc_id, "");
        assert_eq!(view.body, b"x");
    }

    /// Tombstone flag is preserved + helper accessor returns true.
    /// Body MAY be empty for tombstones (publisher signals deletion
    /// without re-shipping content).
    #[test]
    fn roundtrip_tombstone_empty_body() {
        let mut buf = [0u8; 64];
        let n = encode(
            DOC_FLAG_TOMBSTONE,
            "platforms",
            "ANDROID-x",
            42,
            &[],
            &mut buf,
        )
        .expect("encode");
        let view = decode(&buf[..n]).expect("decode");
        assert!(view.is_tombstone());
        assert!(!view.is_encrypted());
        assert_eq!(view.body.len(), 0);
    }

    /// Empty collection is structurally invalid — receivers can't
    /// route the document. Encoder rejects; decoder treats a wire
    /// payload claiming `coll_len = 0` as truncated input.
    #[test]
    fn empty_collection_is_rejected() {
        let mut buf = [0u8; 64];
        assert_eq!(
            encode(0, "", "id", 0, b"x", &mut buf),
            Err(MessageError::FieldTooLarge)
        );

        // Hand-craft a wire payload with coll_len=0 and verify decode
        // surfaces it as malformed rather than a valid empty
        // collection name.
        let mut wire = [0u8; 16];
        wire[0] = 0; // flags
        wire[1] = 0; // coll_len = 0
                     // doc_id_len = 0 follows at [2..4]; rest unused
        assert_eq!(decode(&wire[..4]), Err(MessageError::TruncatedField));
    }

    /// Truncated wire input (each field one byte short) surfaces as
    /// `TruncatedField` rather than panicking on slice bounds. Locks
    /// the malformed-input contract that LoRa-class consumers depend
    /// on.
    #[test]
    fn truncated_input_returns_error_not_panic() {
        let mut buf = [0u8; 256];
        let n = encode(0, "markers", "id", 1, b"body", &mut buf).expect("encode");

        // Truncate at every byte boundary inside the envelope; every
        // shorter slice must Err with TruncatedField (or TooShort for
        // the very smallest cases).
        for trunc in 0..n {
            let res = decode(&buf[..trunc]);
            assert!(
                matches!(
                    res,
                    Err(MessageError::TooShort) | Err(MessageError::TruncatedField),
                ),
                "trunc {} should be TooShort/TruncatedField, got {:?}",
                trunc,
                res
            );
        }
    }

    /// Non-UTF-8 bytes in the collection field surface as
    /// `InvalidUtf8`. The wire contract declares UTF-8; rejecting
    /// non-conforming input keeps the consumer side from having to
    /// guard against arbitrary byte slices in routing keys.
    #[test]
    fn non_utf8_collection_rejected() {
        // Hand-build wire bytes: flags=0, coll_len=3, coll=[0xFF, 0xFE,
        // 0xFD] (invalid UTF-8 continuation), then minimum trailing.
        let mut wire = [0u8; 32];
        wire[0] = 0;
        wire[1] = 3;
        wire[2] = 0xFF;
        wire[3] = 0xFE;
        wire[4] = 0xFD;
        wire[5] = 0; // doc_id_len lo
        wire[6] = 0; // doc_id_len hi
                     // timestamp 7..15
                     // body_len 15..17
        assert_eq!(decode(&wire[..17]), Err(MessageError::InvalidUtf8));
    }

    /// `encoded_len` matches what `encode` actually writes — the
    /// const helper is the size oracle callers use to preflight
    /// against [`MAX_PAYLOAD_SIZE`]; drift would be a silent
    /// fragmentation-decision bug.
    #[test]
    fn encoded_len_matches_encode() {
        let mut buf = [0u8; 1024];
        let body = vec![0xAB; 300];
        let n = encode(0, "tracks", "id-1", 7, &body, &mut buf).expect("encode");
        assert_eq!(
            n,
            encoded_len("tracks".len(), "id-1".len(), body.len()),
            "encoded_len drift",
        );
    }

    /// Field-size limits enforced at encode time. Collection > 255
    /// and body > 65535 must fail-loud rather than silently wrap.
    #[test]
    fn oversize_fields_are_rejected() {
        let mut buf = [0u8; 512];

        // Collection > 255 bytes.
        let big_coll = "x".repeat(MAX_COLLECTION_LEN + 1);
        assert_eq!(
            encode(0, &big_coll, "id", 0, b"x", &mut buf),
            Err(MessageError::FieldTooLarge),
        );

        // Body > 65535 bytes — caller would need a 64KB buffer to
        // even attempt this; verify the size check fires before any
        // copy.
        let body_too_large = vec![0u8; MAX_BODY_LEN + 1];
        let mut huge = vec![0u8; encoded_len("c".len(), "id".len(), body_too_large.len())];
        assert_eq!(
            encode(0, "c", "id", 0, &body_too_large, &mut huge),
            Err(MessageError::FieldTooLarge),
        );
    }
}
