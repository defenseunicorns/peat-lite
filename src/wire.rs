// Copyright (c) 2025-2026 (r)evolve - Revolve Team LLC
// SPDX-License-Identifier: Apache-2.0

//! Wire format constants and error types.
//!
//! Defines markers and error handling for over-the-air message encoding.

/// Marker byte for canned message events on the wire.
///
/// Format: `0xAF` followed by message payload.
/// This allows receivers to distinguish canned messages from other data.
pub const CANNED_MESSAGE_MARKER: u8 = 0xAF;

/// Wire format error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError {
    /// Data too short to contain required fields.
    TooShort,
    /// Invalid marker byte.
    InvalidMarker,
    /// Unknown message code.
    UnknownCode,
    /// Checksum mismatch.
    ChecksumMismatch,
    /// Buffer capacity exceeded.
    BufferFull,
}

#[cfg(feature = "std")]
impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "data too short"),
            Self::InvalidMarker => write!(f, "invalid marker byte"),
            Self::UnknownCode => write!(f, "unknown message code"),
            Self::ChecksumMismatch => write!(f, "checksum mismatch"),
            Self::BufferFull => write!(f, "buffer capacity exceeded"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WireError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_value() {
        // Verify marker is in the "reserved" range and unlikely to collide
        assert_eq!(CANNED_MESSAGE_MARKER, 0xAF);
    }
}
