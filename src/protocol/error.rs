//! Message encoding/decoding errors.

/// Errors that can occur when encoding or decoding a Peat-Lite message.
///
/// Marked `#[non_exhaustive]` so future protocol amendments can add
/// variants without breaking exhaustive-match consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MessageError {
    /// Output buffer is too small for the encoded message.
    BufferTooSmall,
    /// Input buffer is shorter than the minimum header size.
    TooShort,
    /// Magic bytes do not match.
    InvalidMagic,
    /// Protocol version is not supported.
    UnsupportedVersion,
    /// Message type byte is not recognised.
    InvalidMessageType,
    /// Payload exceeds `MAX_PAYLOAD_SIZE`.
    PayloadTooLarge,
    /// A length-prefixed field declared more bytes than remain in the
    /// input — truncated wire encoding.
    TruncatedField,
    /// A length-prefixed string field is not valid UTF-8. peat-lite
    /// declares collection names + document ids as UTF-8 by spec; non-
    /// UTF-8 bytes indicate corruption or a non-conforming sender.
    InvalidUtf8,
    /// A length-prefixed field exceeded its declared maximum
    /// (collection over 255 bytes, doc_id or body over 65535 bytes).
    /// The wire format uses fixed-width length prefixes per field;
    /// oversized values must be rejected at encode time so receivers
    /// can trust the fixed widths.
    FieldTooLarge,
    /// A `MessageType::Document` envelope's collection name was
    /// empty (`coll_len = 0`). Empty collection is structurally
    /// invalid — receivers can't route the document. Distinct from
    /// [`Self::FieldTooLarge`] (which is for length-cap violations)
    /// and from [`Self::TruncatedField`] (which is for buffer-shorter-
    /// than-declared cases). Round-2 of peat-lite#26 introduced this
    /// to fix an asymmetric-variant trap where encoder and decoder
    /// reported the same condition with different error names.
    EmptyCollection,
    /// A reserved or not-yet-implemented flag bit was set on a
    /// `MessageType::Document` envelope. Today's encoder rejects any
    /// flag with reserved bits 2–7 set, and additionally bit 1
    /// (`DOC_FLAG_ENCRYPTED`) which is reserved for a future
    /// per-document encryption layer not yet wired through. Lets us
    /// add features without older encoders silently shipping invalid
    /// frames.
    InvalidFlags,
    /// `MessageType::Document` envelope had `DOC_FLAG_TOMBSTONE` set
    /// AND a non-empty body. Per the envelope spec, a tombstone is a
    /// deletion sentinel; carrying body bytes alongside it is a
    /// publisher contract violation that downstream consumers might
    /// process write-then-delete races against. Rejected at encode
    /// time so the bug doesn't reach the wire.
    TombstoneWithBody,
}
