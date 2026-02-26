//! Primitive CRDTs for Peat-Lite
//!
//! These are lightweight, no_std compatible CRDTs suitable for
//! resource-constrained embedded devices.

pub mod lww_register;
pub mod g_counter;
pub mod pn_counter;

pub use lww_register::LwwRegister;
pub use g_counter::GCounter;
pub use pn_counter::PnCounter;

/// Trait for all Peat-Lite CRDTs
pub trait LiteCrdt: Sized {
    /// The operation type for this CRDT
    type Op;
    /// The value type this CRDT produces
    type Value;

    /// Apply a local operation
    fn apply(&mut self, op: &Self::Op);

    /// Merge with another instance of this CRDT
    fn merge(&mut self, other: &Self);

    /// Get the current value
    fn value(&self) -> Self::Value;

    /// Encode to bytes for network transmission
    /// Returns number of bytes written
    fn encode(&self, buf: &mut [u8]) -> Result<usize, CrdtError>;

    /// Decode from bytes
    fn decode(buf: &[u8]) -> Result<Self, CrdtError>;
}

/// Errors that can occur during CRDT operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrdtError {
    /// Buffer too small for encoding
    BufferTooSmall,
    /// Invalid data during decoding
    InvalidData,
    /// Node ID not found (for counters)
    NodeNotFound,
}

/// TTL tracker for an in-memory CRDT (12 bytes).
///
/// Uses ESP32 monotonic clock (boot-relative), no wall-clock needed.
#[derive(Debug, Clone, Copy)]
pub struct CrdtTtl {
    /// Monotonic timestamp (ms) when the CRDT was last updated.
    last_updated_ms: u64,
    /// TTL in seconds (0 = never expires).
    ttl_seconds: u32,
}

impl CrdtTtl {
    /// Create a new TTL tracker.
    pub const fn new(ttl_seconds: u32, now_ms: u64) -> Self {
        Self {
            last_updated_ms: now_ms,
            ttl_seconds,
        }
    }

    /// Reset the expiration timer.
    pub fn touch(&mut self, now_ms: u64) {
        self.last_updated_ms = now_ms;
    }

    /// Check whether this CRDT has expired.
    pub fn is_expired(&self, now_ms: u64) -> bool {
        if self.ttl_seconds == 0 {
            return false; // never expires
        }
        let elapsed_ms = now_ms.saturating_sub(self.last_updated_ms);
        elapsed_ms / 1000 >= self.ttl_seconds as u64
    }

    /// Remaining TTL in seconds (for wire encoding). Returns 0 if never-expires.
    pub fn remaining_seconds(&self, now_ms: u64) -> u32 {
        if self.ttl_seconds == 0 {
            return 0;
        }
        let elapsed_s = now_ms.saturating_sub(self.last_updated_ms) / 1000;
        self.ttl_seconds.saturating_sub(elapsed_s as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_ttl_never_expires() {
        let ttl = CrdtTtl::new(0, 1000);
        assert!(!ttl.is_expired(999_999_999));
        assert_eq!(ttl.remaining_seconds(999_999_999), 0);
    }

    #[test]
    fn test_crdt_ttl_expiration() {
        let ttl = CrdtTtl::new(300, 0); // 5 min
        assert!(!ttl.is_expired(299_999));
        assert!(ttl.is_expired(300_000));
    }

    #[test]
    fn test_crdt_ttl_touch_resets() {
        let mut ttl = CrdtTtl::new(10, 0);
        assert!(ttl.is_expired(10_000));
        ttl.touch(9_000);
        assert!(!ttl.is_expired(10_000));
        assert!(ttl.is_expired(19_000));
    }

    #[test]
    fn test_crdt_ttl_remaining() {
        let ttl = CrdtTtl::new(300, 0);
        assert_eq!(ttl.remaining_seconds(0), 300);
        assert_eq!(ttl.remaining_seconds(100_000), 200);
        assert_eq!(ttl.remaining_seconds(300_000), 0);
        assert_eq!(ttl.remaining_seconds(999_000), 0); // saturates at 0
    }
}
