// Copyright (c) 2025-2026 Defense Unicorns, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Last-Writer-Wins Register CRDT.
//!
//! A simple register that resolves conflicts by keeping the value
//! with the highest timestamp. If timestamps are equal, the higher
//! node ID wins (deterministic tiebreaker).

use crate::node_id::NodeId;

/// A Last-Writer-Wins Register.
///
/// Stores a single value with timestamp-based conflict resolution.
/// Memory usage: `sizeof(T) + 12 bytes` (timestamp + node_id).
///
/// # Example
///
/// ```rust
/// use peat_lite::{LwwRegister, NodeId};
///
/// let mut reg = LwwRegister::new(42i32, NodeId::new(1), 1000);
///
/// // Update from same node with newer timestamp
/// assert!(reg.update(100, NodeId::new(1), 2000));
/// assert_eq!(reg.value(), &100);
///
/// // Update from different node with older timestamp - rejected
/// assert!(!reg.update(200, NodeId::new(2), 500));
/// assert_eq!(reg.value(), &100);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LwwRegister<T> {
    value: T,
    timestamp: u64,
    node_id: NodeId,
}

impl<T> LwwRegister<T> {
    /// Create a new register with initial value.
    pub const fn new(value: T, node_id: NodeId, timestamp: u64) -> Self {
        Self {
            value,
            timestamp,
            node_id,
        }
    }

    /// Get the current value.
    #[inline]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Get the timestamp of the current value.
    #[inline]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Get the node that wrote the current value.
    #[inline]
    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Update the value if the new write wins.
    ///
    /// Returns `true` if the value was updated, `false` if the
    /// existing value wins.
    ///
    /// LWW rules:
    /// 1. Higher timestamp wins
    /// 2. If timestamps equal, higher node_id wins (deterministic)
    pub fn update(&mut self, value: T, node_id: NodeId, timestamp: u64) -> bool {
        if self.should_accept(timestamp, node_id) {
            self.value = value;
            self.timestamp = timestamp;
            self.node_id = node_id;
            true
        } else {
            false
        }
    }

    /// Merge with another register.
    ///
    /// Takes the winning value based on LWW rules.
    pub fn merge(&mut self, other: Self) {
        if self.should_accept(other.timestamp, other.node_id) {
            *self = other;
        }
    }

    /// Check if an update with given timestamp/node_id should be accepted.
    fn should_accept(&self, timestamp: u64, node_id: NodeId) -> bool {
        timestamp > self.timestamp
            || (timestamp == self.timestamp && node_id.as_u32() > self.node_id.as_u32())
    }
}

impl<T: Default> Default for LwwRegister<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            timestamp: 0,
            node_id: NodeId::NULL,
        }
    }
}

impl<T: Clone> LwwRegister<T> {
    /// Get a clone of the current value.
    pub fn value_cloned(&self) -> T {
        self.value.clone()
    }
}

/// A position value suitable for LwwRegister.
///
/// Uses fixed-point representation for no_std compatibility.
/// Latitude/longitude stored as microdegrees (degrees × 1,000,000).
/// Altitude stored as centimeters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    /// Latitude in microdegrees (degrees × 1,000,000)
    pub lat_microdeg: i32,
    /// Longitude in microdegrees (degrees × 1,000,000)
    pub lon_microdeg: i32,
    /// Altitude in centimeters above WGS84 ellipsoid
    pub alt_cm: i32,
}

impl Position {
    /// Create from floating-point degrees.
    #[cfg(feature = "std")]
    pub fn from_degrees(lat: f64, lon: f64, alt_m: f32) -> Self {
        Self {
            lat_microdeg: (lat * 1_000_000.0) as i32,
            lon_microdeg: (lon * 1_000_000.0) as i32,
            alt_cm: (alt_m * 100.0) as i32,
        }
    }

    /// Convert to floating-point degrees.
    #[cfg(feature = "std")]
    pub fn to_degrees(&self) -> (f64, f64, f32) {
        (
            self.lat_microdeg as f64 / 1_000_000.0,
            self.lon_microdeg as f64 / 1_000_000.0,
            self.alt_cm as f32 / 100.0,
        )
    }

    /// Encode to 12 bytes.
    pub fn encode(&self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        buf[0..4].copy_from_slice(&self.lat_microdeg.to_le_bytes());
        buf[4..8].copy_from_slice(&self.lon_microdeg.to_le_bytes());
        buf[8..12].copy_from_slice(&self.alt_cm.to_le_bytes());
        buf
    }

    /// Decode from 12 bytes.
    pub fn decode(data: &[u8; 12]) -> Self {
        Self {
            lat_microdeg: i32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            lon_microdeg: i32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            alt_cm: i32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_basic() {
        let mut reg = LwwRegister::new(10, NodeId::new(1), 100);
        assert_eq!(reg.value(), &10);

        // Newer timestamp wins
        assert!(reg.update(20, NodeId::new(2), 200));
        assert_eq!(reg.value(), &20);

        // Older timestamp loses
        assert!(!reg.update(30, NodeId::new(3), 150));
        assert_eq!(reg.value(), &20);
    }

    #[test]
    fn test_lww_tiebreaker() {
        let mut reg = LwwRegister::new(10, NodeId::new(1), 100);

        // Same timestamp, higher node_id wins
        assert!(reg.update(20, NodeId::new(5), 100));
        assert_eq!(reg.value(), &20);
        assert_eq!(reg.node_id(), NodeId::new(5));

        // Same timestamp, lower node_id loses
        assert!(!reg.update(30, NodeId::new(3), 100));
        assert_eq!(reg.value(), &20);
    }

    #[test]
    fn test_lww_merge() {
        let mut reg1 = LwwRegister::new(10, NodeId::new(1), 100);
        let reg2 = LwwRegister::new(20, NodeId::new(2), 200);

        reg1.merge(reg2);
        assert_eq!(reg1.value(), &20);
    }

    #[test]
    fn test_position_encode_decode() {
        let pos = Position {
            lat_microdeg: 37_774_929,   // ~37.774929° (San Francisco)
            lon_microdeg: -122_419_416, // ~-122.419416°
            alt_cm: 1000,               // 10m
        };

        let encoded = pos.encode();
        let decoded = Position::decode(&encoded);
        assert_eq!(pos, decoded);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_position_degrees() {
        let pos = Position::from_degrees(37.774929, -122.419416, 10.0);
        let (lat, lon, alt) = pos.to_degrees();

        assert!((lat - 37.774929).abs() < 0.000001);
        assert!((lon - (-122.419416)).abs() < 0.000001);
        assert!((alt - 10.0).abs() < 0.01);
    }
}
