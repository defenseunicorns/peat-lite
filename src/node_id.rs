// Copyright (c) 2025-2026 Defense Unicorns, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Node identifier type.
//!
//! A 32-bit identifier for nodes in the mesh. This is intentionally smaller
//! than a full UUID to fit memory constraints on embedded devices.

/// A 32-bit node identifier.
///
/// Derived from BLE MAC address or assigned during provisioning.
/// Collision probability is acceptable for tactical mesh sizes (<1000 nodes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct NodeId(u32);

impl NodeId {
    /// Create a new NodeId from a 32-bit value.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Create a NodeId from bytes (little-endian).
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(bytes))
    }

    /// Create a NodeId from bytes (big-endian).
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_be_bytes(bytes))
    }

    /// Get the raw u32 value.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Convert to little-endian bytes.
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    /// Convert to big-endian bytes.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    /// Check if this is the null/invalid node ID.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// The null/invalid node ID.
    pub const NULL: Self = Self(0);
}

impl From<u32> for NodeId {
    #[inline]
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<NodeId> for u32 {
    #[inline]
    fn from(id: NodeId) -> Self {
        id.0
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for NodeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:08X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_roundtrip() {
        let id = NodeId::new(0xDEADBEEF);
        let bytes = id.to_le_bytes();
        let recovered = NodeId::from_le_bytes(bytes);
        assert_eq!(id, recovered);
    }

    #[test]
    fn test_node_id_null() {
        assert!(NodeId::NULL.is_null());
        assert!(NodeId::new(0).is_null());
        assert!(!NodeId::new(1).is_null());
    }
}
