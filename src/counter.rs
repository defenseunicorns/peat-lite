// Copyright (c) 2025-2026 (r)evolve - Revolve Team LLC
// SPDX-License-Identifier: Apache-2.0

//! Grow-only Counter (G-Counter) CRDT.
//!
//! A distributed counter that can only be incremented. Each node maintains
//! its own count, and the total is the sum of all node counts.

use crate::node_id::NodeId;
use heapless::FnvIndexMap;

/// A Grow-only Counter (G-Counter).
///
/// Each node can only increment its own count. The counter value is
/// the sum of all node counts. Merge takes the maximum of each node's count.
///
/// Memory usage: approximately `4 + (MAX_NODES * 8)` bytes.
/// Default capacity of 32 nodes ≈ 260 bytes.
///
/// # Example
///
/// ```rust
/// use hive_lite::{GCounter, NodeId};
///
/// let node1 = NodeId::new(1);
/// let node2 = NodeId::new(2);
///
/// let mut counter1 = GCounter::<8>::new();
/// counter1.increment(node1, 5);
///
/// let mut counter2 = GCounter::<8>::new();
/// counter2.increment(node2, 3);
///
/// counter1.merge(&counter2);
/// assert_eq!(counter1.value(), 8);
/// ```
#[derive(Debug, Clone)]
pub struct GCounter<const MAX_NODES: usize = 32> {
    counts: FnvIndexMap<NodeId, u32, MAX_NODES>,
}

impl<const MAX_NODES: usize> Default for GCounter<MAX_NODES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_NODES: usize> GCounter<MAX_NODES> {
    /// Create a new empty counter.
    pub const fn new() -> Self {
        Self {
            counts: FnvIndexMap::new(),
        }
    }

    /// Get the total counter value (sum of all node counts).
    pub fn value(&self) -> u64 {
        self.counts.values().map(|&v| v as u64).sum()
    }

    /// Get a specific node's count.
    pub fn node_count(&self, node: NodeId) -> u32 {
        self.counts.get(&node).copied().unwrap_or(0)
    }

    /// Increment the counter for a specific node.
    ///
    /// Returns the new count for that node, or `None` if the counter
    /// is full and can't add a new node.
    pub fn increment(&mut self, node: NodeId, delta: u32) -> Option<u32> {
        match self.counts.get_mut(&node) {
            Some(count) => {
                *count = count.saturating_add(delta);
                Some(*count)
            }
            None => {
                let new_count = delta;
                self.counts.insert(node, new_count).ok()?;
                Some(new_count)
            }
        }
    }

    /// Increment by 1.
    pub fn inc(&mut self, node: NodeId) -> Option<u32> {
        self.increment(node, 1)
    }

    /// Merge with another counter.
    ///
    /// Takes the maximum of each node's count.
    pub fn merge(&mut self, other: &Self) {
        for (&node, &other_count) in other.counts.iter() {
            match self.counts.get_mut(&node) {
                Some(count) => {
                    *count = (*count).max(other_count);
                }
                None => {
                    // Try to insert; ignore if full
                    let _ = self.counts.insert(node, other_count);
                }
            }
        }
    }

    /// Get the number of nodes that have contributed to this counter.
    pub fn node_count_total(&self) -> usize {
        self.counts.len()
    }

    /// Check if this counter is empty (all counts are 0 or no nodes).
    pub fn is_empty(&self) -> bool {
        self.counts.is_empty() || self.value() == 0
    }

    /// Clear all counts.
    pub fn clear(&mut self) {
        self.counts.clear();
    }

    /// Iterate over (node_id, count) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, u32)> + '_ {
        self.counts.iter().map(|(&node, &count)| (node, count))
    }

    /// Encode to bytes for transmission.
    ///
    /// Format: `[num_entries: u16][entry1][entry2]...`
    /// Each entry: `[node_id: 4B][count: 4B]` = 8 bytes
    pub fn encode(&self) -> heapless::Vec<u8, 258> {
        // 2 + 32*8 = 258 max
        let mut buf = heapless::Vec::new();

        let count = self.counts.len() as u16;
        let _ = buf.extend_from_slice(&count.to_le_bytes());

        for (&node, &value) in self.counts.iter() {
            let _ = buf.extend_from_slice(&node.to_le_bytes());
            let _ = buf.extend_from_slice(&value.to_le_bytes());
        }

        buf
    }

    /// Decode from bytes.
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 2 {
            return None;
        }

        let count = u16::from_le_bytes([data[0], data[1]]) as usize;

        if data.len() < 2 + count * 8 {
            return None;
        }

        let mut counter = Self::new();
        let mut offset = 2;

        for _ in 0..count {
            let node = NodeId::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            let value = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            offset += 8;

            let _ = counter.counts.insert(node, value);
        }

        Some(counter)
    }
}

impl<const MAX_NODES: usize> PartialEq for GCounter<MAX_NODES> {
    fn eq(&self, other: &Self) -> bool {
        if self.counts.len() != other.counts.len() {
            return false;
        }
        for (&node, &count) in self.counts.iter() {
            if other.node_count(node) != count {
                return false;
            }
        }
        true
    }
}

impl<const MAX_NODES: usize> Eq for GCounter<MAX_NODES> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcounter_basic() {
        let mut counter = GCounter::<8>::new();
        let node = NodeId::new(1);

        assert_eq!(counter.value(), 0);

        counter.inc(node);
        assert_eq!(counter.value(), 1);
        assert_eq!(counter.node_count(node), 1);

        counter.increment(node, 5);
        assert_eq!(counter.value(), 6);
        assert_eq!(counter.node_count(node), 6);
    }

    #[test]
    fn test_gcounter_multiple_nodes() {
        let mut counter = GCounter::<8>::new();
        let node1 = NodeId::new(1);
        let node2 = NodeId::new(2);

        counter.increment(node1, 10);
        counter.increment(node2, 20);

        assert_eq!(counter.value(), 30);
        assert_eq!(counter.node_count(node1), 10);
        assert_eq!(counter.node_count(node2), 20);
    }

    #[test]
    fn test_gcounter_merge() {
        let node1 = NodeId::new(1);
        let node2 = NodeId::new(2);

        let mut counter1 = GCounter::<8>::new();
        counter1.increment(node1, 10);
        counter1.increment(node2, 5);

        let mut counter2 = GCounter::<8>::new();
        counter2.increment(node1, 8); // Lower than counter1
        counter2.increment(node2, 15); // Higher than counter1

        counter1.merge(&counter2);

        // Should take max of each
        assert_eq!(counter1.node_count(node1), 10); // max(10, 8)
        assert_eq!(counter1.node_count(node2), 15); // max(5, 15)
        assert_eq!(counter1.value(), 25);
    }

    #[test]
    fn test_gcounter_encode_decode() {
        let mut counter = GCounter::<8>::new();
        counter.increment(NodeId::new(1), 100);
        counter.increment(NodeId::new(2), 200);

        let encoded = counter.encode();
        let decoded = GCounter::<8>::decode(&encoded).unwrap();

        assert_eq!(counter, decoded);
    }

    #[test]
    fn test_gcounter_saturating() {
        let mut counter = GCounter::<8>::new();
        let node = NodeId::new(1);

        counter.increment(node, u32::MAX - 10);
        counter.increment(node, 100); // Would overflow

        // Should saturate at MAX
        assert_eq!(counter.node_count(node), u32::MAX);
    }
}
