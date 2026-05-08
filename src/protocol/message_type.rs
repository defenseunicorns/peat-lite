//! Peat-Lite message type identifiers.

/// Message types for the gossip protocol.
///
/// Marked `#[non_exhaustive]` so future protocol amendments can add
/// variants without breaking exhaustive-match consumers in downstream
/// crates (peat-mesh, peat-btle, peat-atak-plugin). Match arms must
/// include a `_ =>` fall-through to compile against the latest
/// peat-lite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u8)]
pub enum MessageType {
    /// Announce presence and capabilities.
    Announce = 0x01,
    /// Heartbeat / keep-alive.
    Heartbeat = 0x02,
    /// Data update (CRDT state).
    Data = 0x03,
    /// Query for specific state.
    Query = 0x04,
    /// Acknowledge receipt.
    Ack = 0x05,
    /// Leave notification.
    Leave = 0x06,
    /// Universal Document carrier (peat_mesh::Document, transport-agnostic).
    ///
    /// Distinct from [`MessageType::Data`]'s typed-CRDT-primitive payload —
    /// `Document` carries an opaque envelope: collection name + doc id +
    /// timestamp + length-prefixed body bytes whose interpretation is owned
    /// by peat-mesh (or any consumer). Adding a new collection on the
    /// network requires zero codec changes downstream of peat-lite.
    /// See `protocol::document` for the wire layout.
    Document = 0x07,
    /// OTA firmware offer (Full -> Lite).
    OtaOffer = 0x10,
    /// OTA accept (Lite -> Full).
    OtaAccept = 0x11,
    /// OTA data chunk (Full -> Lite).
    OtaData = 0x12,
    /// OTA chunk acknowledgement (Lite -> Full).
    OtaAck = 0x13,
    /// OTA transfer complete (Full -> Lite).
    OtaComplete = 0x14,
    /// OTA result (Lite -> Full).
    OtaResult = 0x15,
    /// OTA abort (either direction).
    OtaAbort = 0x16,
}

impl MessageType {
    /// Convert a raw byte to a `MessageType`, if valid.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Announce),
            0x02 => Some(Self::Heartbeat),
            0x03 => Some(Self::Data),
            0x04 => Some(Self::Query),
            0x05 => Some(Self::Ack),
            0x06 => Some(Self::Leave),
            0x07 => Some(Self::Document),
            0x10 => Some(Self::OtaOffer),
            0x11 => Some(Self::OtaAccept),
            0x12 => Some(Self::OtaData),
            0x13 => Some(Self::OtaAck),
            0x14 => Some(Self::OtaComplete),
            0x15 => Some(Self::OtaResult),
            0x16 => Some(Self::OtaAbort),
            _ => None,
        }
    }
}
