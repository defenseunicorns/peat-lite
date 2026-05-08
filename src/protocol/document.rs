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
/// referenced by `(collection, doc_id)`. Body MUST be empty; the
/// encoder rejects `tombstone | body.len() > 0` to prevent
/// publisher-side write-then-delete contract violations.
pub const DOC_FLAG_TOMBSTONE: u8 = 0x01;
/// Bit 1: body is encrypted (per-document, key established
/// out-of-band). **Reserved for a future encryption layer**; today's
/// encoder rejects the flag entirely (returns
/// `MessageError::InvalidFlags`) so non-conforming senders can't ship
/// frames downstream consumers don't know how to decrypt. Decoders
/// see the flag via `DocumentRef::is_encrypted()` for forward-compat
/// inspection but MUST treat the body as opaque.
pub const DOC_FLAG_ENCRYPTED: u8 = 0x02;
/// Mask of currently-defined flag bits. Bits set outside this mask
/// are reserved for future protocol versions; today's encoder
/// rejects them with `MessageError::InvalidFlags` so that legacy
/// frames can't enable not-yet-implemented behaviors. Round-2 of
/// peat-lite#26 added this reservation contract.
pub const DOC_FLAGS_MASK: u8 = DOC_FLAG_TOMBSTONE | DOC_FLAG_ENCRYPTED;

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
///
/// Marked `#[non_exhaustive]` so future protocol amendments can add
/// fields (e.g. fragmentation sequence number) without breaking
/// downstream `DocumentRef { ... }` literal constructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
///
/// Returns `None` on `usize` overflow — relevant on 32-bit targets
/// (ESP32, LoRa MCUs) when callers pass user-controlled lengths. With
/// today's per-field maxima the worst case sums to ~131 KB which fits
/// 32-bit `usize` comfortably; the guard exists to keep that
/// invariant explicit if a future field expansion ever drives the
/// total higher.
#[inline]
pub const fn encoded_len(
    collection_len: usize,
    doc_id_len: usize,
    body_len: usize,
) -> Option<usize> {
    // flags(1) + collection_len(1) + collection(N)
    //        + doc_id_len(2) + doc_id(M)
    //        + timestamp_ms(8)
    //        + body_len(2) + body(K)
    let fixed = 1 + 1 + 2 + 8 + 2;
    let Some(t) = collection_len.checked_add(doc_id_len) else {
        return None;
    };
    let Some(t) = t.checked_add(body_len) else {
        return None;
    };
    let Some(t) = t.checked_add(fixed) else {
        return None;
    };
    Some(t)
}

/// Encode a Document envelope into `buf`. Returns the number of bytes
/// written, or an error if any field exceeds its width, the buffer is
/// too small, or the flags / tombstone-vs-body invariants are
/// violated.
///
/// `buf` is the payload region — callers should already have written
/// the 16-byte peat-lite header (with [`MessageType::Document`]) into
/// the preceding bytes via [`super::header::encode_header`].
///
/// ## Validity contracts enforced
///
/// - `collection`: non-empty, ≤ [`MAX_COLLECTION_LEN`] bytes, must
///   not contain NUL bytes (NUL would create routing-key ambiguity
///   downstream where consumers may treat `"markers\0"` and
///   `"markers"` as different OR the same depending on path).
/// - `doc_id`: ≤ [`MAX_DOC_ID_LEN`] bytes; empty signals
///   publisher-delegates-id.
/// - `body`: ≤ [`MAX_BODY_LEN`] bytes; **MUST be empty when
///   `flags & DOC_FLAG_TOMBSTONE != 0`**. The encoder rejects
///   tombstone-with-body to prevent publisher contract violations
///   that downstream consumers might race write-then-delete against.
/// - `flags`: only bits in [`DOC_FLAGS_MASK`] may be set. Reserved
///   bits 2–7 plus the encrypted bit (today not implemented) all
///   produce [`MessageError::InvalidFlags`].
pub fn encode(
    flags: u8,
    collection: &str,
    doc_id: &str,
    timestamp_ms: i64,
    body: &[u8],
    buf: &mut [u8],
) -> Result<usize, MessageError> {
    // Flag validity: only bits in DOC_FLAGS_MASK are defined today,
    // AND the encrypted bit is reserved for a future encryption layer
    // we haven't wired through. Reject either condition so legacy
    // encoders can't ship frames newer consumers don't know how to
    // process. Tombstone (bit 0) is the only flag actually emitted.
    if flags & !DOC_FLAGS_MASK != 0 || flags & DOC_FLAG_ENCRYPTED != 0 {
        return Err(MessageError::InvalidFlags);
    }

    let coll_bytes = collection.as_bytes();
    let id_bytes = doc_id.as_bytes();

    if coll_bytes.is_empty() {
        return Err(MessageError::EmptyCollection);
    }
    // NUL bytes in collection name create routing-key ambiguity: some
    // string-comparison paths treat the embedded NUL as a terminator,
    // others as a literal byte. Reject so the wire is unambiguous.
    if coll_bytes.contains(&0) {
        return Err(MessageError::InvalidUtf8);
    }
    if coll_bytes.len() > MAX_COLLECTION_LEN
        || id_bytes.len() > MAX_DOC_ID_LEN
        || body.len() > MAX_BODY_LEN
    {
        return Err(MessageError::FieldTooLarge);
    }

    // Tombstone-vs-body invariant: a tombstone is a deletion sentinel,
    // not a "delete then create" combo. Carrying a body alongside the
    // flag invites publisher-side contract violations (publisher
    // forgot to clear the body buffer; adversarial sender combining
    // ops). Reject at encode so downstream consumers can trust the
    // invariant unconditionally.
    if flags & DOC_FLAG_TOMBSTONE != 0 && !body.is_empty() {
        return Err(MessageError::TombstoneWithBody);
    }

    let needed = encoded_len(coll_bytes.len(), id_bytes.len(), body.len())
        .ok_or(MessageError::FieldTooLarge)?;
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

    // Runtime assertion (not debug-only): a logic bug here would
    // produce a wrong wire length silently in release builds, which
    // is exactly the catastrophic divergence the QA round-2 review
    // flagged. Cheap one-branch check on the encode slow path.
    assert_eq!(
        o, needed,
        "Document encode wrote {} bytes, expected {} — encoder logic bug",
        o, needed
    );
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
        // Mirror the encode-time rejection. Distinct error variant
        // from `TruncatedField` (which is for buffer-shorter-than-
        // declared) so callers get an unambiguous diagnosis.
        return Err(MessageError::EmptyCollection);
    }
    if collection.as_bytes().contains(&0) {
        // NUL bytes in routing keys create downstream ambiguity
        // (string-comparison paths may treat NUL as terminator vs
        // literal). Reject for parity with the encoder check.
        return Err(MessageError::InvalidUtf8);
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
    /// route the document. Encoder + decoder both surface
    /// `EmptyCollection` (round-2 of peat-lite#26 added the variant
    /// to fix the previous asymmetric `FieldTooLarge` /
    /// `TruncatedField` reporting).
    #[test]
    fn empty_collection_is_rejected_with_distinct_variant() {
        let mut buf = [0u8; 64];
        assert_eq!(
            encode(0, "", "id", 0, b"x", &mut buf),
            Err(MessageError::EmptyCollection),
            "encode-side empty-collection must surface EmptyCollection"
        );

        // Hand-craft a wire payload with coll_len=0 and verify decode
        // also surfaces EmptyCollection — symmetric with encode.
        let mut wire = [0u8; 16];
        wire[0] = 0; // flags
        wire[1] = 0; // coll_len = 0
                     // doc_id_len = 0 follows at [2..4]; rest unused
        assert_eq!(
            decode(&wire[..4]),
            Err(MessageError::EmptyCollection),
            "decode-side empty-collection must surface EmptyCollection"
        );
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
            encoded_len("tracks".len(), "id-1".len(), body.len()).expect("no overflow"),
            "encoded_len drift",
        );
    }

    /// `encoded_len` guards against `usize` overflow on 32-bit
    /// targets. With the value-cap maxima in place today the worst
    /// case fits 32-bit, but the guard exists so future field
    /// expansions can't silently wrap.
    #[test]
    fn encoded_len_returns_none_on_overflow() {
        // Pass values that sum past `usize::MAX`; on 64-bit hosts
        // this requires `usize::MAX` itself, on 32-bit ESP32 just two
        // 2^31-ish values suffice.
        assert_eq!(encoded_len(usize::MAX, 1, 0), None);
        assert_eq!(encoded_len(0, usize::MAX, 1), None);
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
        let needed = encoded_len("c".len(), "id".len(), body_too_large.len()).expect("no overflow");
        let mut huge = vec![0u8; needed];
        assert_eq!(
            encode(0, "c", "id", 0, &body_too_large, &mut huge),
            Err(MessageError::FieldTooLarge),
        );
    }

    // -------------------------------------------------------------
    // Round-2 review additions: adversarial-input invariants
    // -------------------------------------------------------------

    /// `DOC_FLAG_ENCRYPTED` is reserved — encoder rejects any frame
    /// with bit 1 set. Forward-compat: prevents a non-conforming
    /// sender from shipping frames downstream consumers can't
    /// decrypt while the encryption layer is still being designed.
    #[test]
    fn encrypted_flag_is_rejected_today() {
        let mut buf = [0u8; 64];
        assert_eq!(
            encode(DOC_FLAG_ENCRYPTED, "markers", "id", 0, b"x", &mut buf),
            Err(MessageError::InvalidFlags),
        );
    }

    /// Reserved flag bits 2-7 are rejected. Adding a new flag in a
    /// future protocol version is then opt-in (set the bit + bump
    /// peat-lite version) rather than implicit; legacy encoders
    /// can't accidentally enable behaviors they don't implement.
    #[test]
    fn reserved_flag_bits_are_rejected() {
        let mut buf = [0u8; 64];
        for bit in 2..=7 {
            let flags = 1u8 << bit;
            assert_eq!(
                encode(flags, "markers", "id", 0, b"x", &mut buf),
                Err(MessageError::InvalidFlags),
                "bit {} should be rejected as reserved",
                bit
            );
        }
    }

    /// Tombstone with a non-empty body is a publisher contract
    /// violation — encoder rejects so downstream consumers don't
    /// have to handle write-then-delete races.
    #[test]
    fn tombstone_with_non_empty_body_is_rejected() {
        let mut buf = [0u8; 64];
        assert_eq!(
            encode(
                DOC_FLAG_TOMBSTONE,
                "tracks",
                "id",
                0,
                b"unwanted body",
                &mut buf
            ),
            Err(MessageError::TombstoneWithBody),
        );
    }

    /// NUL bytes in a collection name create routing-key ambiguity
    /// (string-comparison paths may treat embedded NUL as terminator
    /// vs literal byte). Encoder + decoder both reject for parity.
    #[test]
    fn nul_bytes_in_collection_are_rejected() {
        let mut buf = [0u8; 64];
        assert_eq!(
            encode(0, "mark\0ers", "id", 0, b"x", &mut buf),
            Err(MessageError::InvalidUtf8),
        );

        // Hand-craft a wire payload with a NUL-bearing collection
        // name and verify decode also rejects.
        let mut wire = [0u8; 32];
        wire[0] = 0; // flags
        wire[1] = 8; // coll_len = 8
        wire[2..10].copy_from_slice(b"mark\0ers");
        wire[10] = 0; // doc_id_len lo
        wire[11] = 0; // doc_id_len hi
                      // timestamp 12..20
                      // body_len 20..22
        assert_eq!(
            decode(&wire[..22]),
            Err(MessageError::InvalidUtf8),
            "decode must reject NUL-bearing collection for symmetry with encode"
        );
    }

    /// Randomised round-trip: encode-decode-assert-eq across a wide
    /// input matrix. Catches the entire class of issues that fixed-
    /// fixture tests miss — boundary values, character ranges,
    /// length-prefix arithmetic. Deterministic seed keeps CI stable.
    #[test]
    fn fuzzy_roundtrip_random_inputs() {
        // Tiny LCG — no external dep, deterministic, good enough for
        // input matrix coverage in a unit test. Seed chosen so all
        // four field-length axes (small/medium collection, small/large
        // doc_id, varied body) get exercised.
        let mut state: u32 = 0xCAFEBABE;
        let mut next = || {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            state
        };

        for _ in 0..200 {
            let coll_choices = ["m", "markers", "platforms", "alerts", "company_summaries"];
            let coll = coll_choices[(next() as usize) % coll_choices.len()];

            // doc_id: 0..256 ASCII bytes (UTF-8-clean by construction)
            let id_len = (next() as usize) % 257;
            let mut id = String::with_capacity(id_len);
            for _ in 0..id_len {
                id.push(char::from(0x21 + ((next() & 0x3F) as u8))); // '!'..'`'
            }

            let body_len = (next() as usize) % 1024;
            let mut body = vec![0u8; body_len];
            for byte in body.iter_mut() {
                *byte = next() as u8;
            }

            let ts = next() as i64;
            let needed = encoded_len(coll.len(), id.len(), body.len()).expect("no overflow");
            let mut buf = vec![0u8; needed];
            let n = encode(0, coll, &id, ts, &body, &mut buf).expect("encode random input");
            assert_eq!(n, needed);

            let view = decode(&buf).expect("decode random input");
            assert_eq!(view.collection, coll);
            assert_eq!(view.doc_id, id);
            assert_eq!(view.timestamp_ms, ts);
            assert_eq!(view.body, body.as_slice());
        }
    }
}
