//! TTL (Time-To-Live) codec for Data messages.
//!
//! When `FLAG_HAS_TTL` is set in the header flags, the last 4 bytes of the
//! payload contain the TTL in seconds as a little-endian u32.
//!
//! Backward compatible: old decoders ignore unknown flag bits, and CRDT
//! payloads are self-delimiting (LwwRegister has `value_len`, GCounter
//! has `num_entries`).

use crate::constants::{
    DEFAULT_TTL_G_COUNTER, DEFAULT_TTL_LWW_REGISTER, DEFAULT_TTL_PN_COUNTER, FLAG_HAS_TTL,
    TTL_NEVER_EXPIRES, TTL_SUFFIX_SIZE,
};
use crate::crdt_type::CrdtType;

/// Append a 4-byte LE TTL suffix to `payload[..payload_len]`.
///
/// Returns `Some(new_len)` on success, `None` if the buffer is too small.
pub fn append_ttl(payload: &mut [u8], payload_len: usize, ttl_seconds: u32) -> Option<usize> {
    let new_len = payload_len + TTL_SUFFIX_SIZE;
    if new_len > payload.len() {
        return None;
    }
    payload[payload_len..new_len].copy_from_slice(&ttl_seconds.to_le_bytes());
    Some(new_len)
}

/// Strip the TTL suffix from a payload when `FLAG_HAS_TTL` is set.
///
/// Returns `(crdt_data, ttl_seconds)`. If the flag is not set, returns the
/// full payload with `TTL_NEVER_EXPIRES`.
pub fn strip_ttl(flags: u16, payload: &[u8]) -> (&[u8], u32) {
    if flags & FLAG_HAS_TTL != 0 && payload.len() >= TTL_SUFFIX_SIZE {
        let split = payload.len() - TTL_SUFFIX_SIZE;
        let ttl = u32::from_le_bytes([
            payload[split],
            payload[split + 1],
            payload[split + 2],
            payload[split + 3],
        ]);
        (&payload[..split], ttl)
    } else {
        (payload, TTL_NEVER_EXPIRES)
    }
}

/// Return the default TTL (in seconds) for a given CRDT type.
pub const fn default_ttl_for_crdt(crdt: CrdtType) -> u32 {
    match crdt {
        CrdtType::LwwRegister => DEFAULT_TTL_LWW_REGISTER,
        CrdtType::GCounter => DEFAULT_TTL_G_COUNTER,
        CrdtType::PnCounter => DEFAULT_TTL_PN_COUNTER,
        CrdtType::OrSet => TTL_NEVER_EXPIRES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_append_strip() {
        let crdt_data = [0x01, 0x02, 0x03];
        let mut buf = [0u8; 16];
        buf[..3].copy_from_slice(&crdt_data);

        let new_len = append_ttl(&mut buf, 3, 300).unwrap();
        assert_eq!(new_len, 7);

        let (data, ttl) = strip_ttl(FLAG_HAS_TTL, &buf[..new_len]);
        assert_eq!(data, &crdt_data);
        assert_eq!(ttl, 300);
    }

    #[test]
    fn no_flag_means_no_ttl() {
        let payload = [0x01, 0x02, 0x03, 0x04];
        let (data, ttl) = strip_ttl(0, &payload);
        assert_eq!(data, &payload);
        assert_eq!(ttl, TTL_NEVER_EXPIRES);
    }

    #[test]
    fn buffer_overflow_returns_none() {
        let mut buf = [0u8; 3];
        buf[..2].copy_from_slice(&[0x01, 0x02]);
        assert!(append_ttl(&mut buf, 2, 300).is_none());
    }

    #[test]
    fn default_ttl_per_crdt() {
        assert_eq!(default_ttl_for_crdt(CrdtType::LwwRegister), 300);
        assert_eq!(default_ttl_for_crdt(CrdtType::GCounter), 3600);
        assert_eq!(default_ttl_for_crdt(CrdtType::PnCounter), 3600);
        assert_eq!(default_ttl_for_crdt(CrdtType::OrSet), TTL_NEVER_EXPIRES);
    }

    #[test]
    fn strip_ttl_short_payload_with_flag() {
        // Payload too short for TTL suffix — returns as-is
        let payload = [0x01, 0x02];
        let (data, ttl) = strip_ttl(FLAG_HAS_TTL, &payload);
        assert_eq!(data, &payload);
        assert_eq!(ttl, TTL_NEVER_EXPIRES);
    }
}
