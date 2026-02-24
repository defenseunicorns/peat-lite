# Claude Code Project Guide - hive-lite

## Project Overview

hive-lite provides lightweight CRDT primitives for resource-constrained HIVE nodes. It is a **leaf crate** with no dependencies on the HIVE ecosystem, designed for devices with 256KB RAM budget.

## Radicle Workflow

This project uses [Radicle](https://radicle.xyz) for decentralized code collaboration.

**Repository ID**: `rad:z4Bhrn1aB8T5vp6Vg42xxvAXx5TJx`
**Web UI**: https://app.radicle.xyz/nodes/seed.radicle.garden/rad:z4Bhrn1aB8T5vp6Vg42xxvAXx5TJx

### Quick Reference

```bash
# Sync before starting work
rad sync --fetch

# Check for open patches
rad patch list

# Create a patch (from feature branch)
git push rad HEAD:refs/patches -o patch.message="feat: My change"
```

## Build Commands

```bash
# Build with std (default)
cargo build

# Build for no_std (embedded)
cargo build --no-default-features

# Run tests
cargo test

# Check all CI requirements
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --no-default-features
```

## CI Status

GOA runs CI automatically on patches:
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. `cargo build --no-default-features` (no_std verification)

## Architecture

### Primitives

| Type | Purpose | Memory |
|------|---------|--------|
| `NodeId` | 32-bit node identifier | 4 bytes |
| `CannedMessage` | Predefined message codes | 1 byte |
| `CannedMessageEvent` | Message with metadata | ~24 bytes |
| `CannedMessageStore` | LWW storage | ~6KB (256 entries) |
| `LwwRegister<T>` | Last-writer-wins register | sizeof(T) + 12 bytes |
| `GCounter` | Grow-only counter | ~4 bytes per node |

### CannedMessage Code Ranges

- `0x00-0x0F`: Acknowledgments (ACK, WILCO, NEGATIVE, SAY AGAIN)
- `0x10-0x1F`: Status (CHECK IN, MOVING, HOLDING, ON STATION, RTB, COMPLETE)
- `0x20-0x2F`: Alerts (EMERGENCY, ALERT, ALL CLEAR, CONTACT, UNDER FIRE)
- `0x30-0x3F`: Requests (NEED EXTRACT, NEED SUPPORT, MEDIC, RESUPPLY)
- `0xF0-0xFF`: Reserved/custom

### Wire Format

CannedMessageEvent uses a 22-byte wire format:
```
┌──────┬──────────┬──────────┬──────────┬───────────┬──────┐
│ 0xAF │ msg_code │ src_node │ tgt_node │ timestamp │ seq  │
│ 1B   │ 1B       │ 4B       │ 4B (opt) │ 8B        │ 4B   │
└──────┴──────────┴──────────┴──────────┴───────────┴──────┘
```

## Key Files

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Public API exports |
| `src/node_id.rs` | 32-bit node identifier |
| `src/canned.rs` | CannedMessage, CannedMessageEvent, CannedMessageStore |
| `src/lww.rs` | LwwRegister, Position |
| `src/counter.rs` | GCounter (grow-only counter) |
| `src/wire.rs` | Wire format constants and errors |

## Related Repositories

- **hive-btle**: BLE mesh transport (optionally depends on hive-lite)
- **hive-protocol**: Full HIVE protocol (uses hive-lite for embedded nodes)
