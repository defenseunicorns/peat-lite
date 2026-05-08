//! Message encoding/decoding errors.

/// Errors that can occur when encoding or decoding a Peat-Lite message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}
