# Claude Code Project Guide - eche-lite

## Project Overview

eche-lite provides lightweight CRDT primitives and wire protocol for resource-constrained Eche nodes. It is a **leaf crate** with no dependencies on the Eche ecosystem, designed for devices with 256KB RAM budget. The `protocol` submodule is the single source of truth for the Eche-Lite binary wire protocol (ADR-035).

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
| `src/protocol/mod.rs` | Protocol re-exports (wire protocol types) |
| `src/protocol/constants.rs` | Magic bytes, header size, port, multicast |
| `src/protocol/header.rs` | Header encode/decode (16-byte fixed) |
| `src/protocol/message_type.rs` | MessageType enum (Announce, Data, OTA, etc.) |
| `src/protocol/capabilities.rs` | NodeCapabilities bitflags |
| `src/protocol/crdt_type.rs` | CrdtType enum |
| `src/protocol/ota.rs` | OTA firmware update constants |
| `src/protocol/ttl.rs` | TTL append/strip helpers |

## Related Repositories

- **eche-btle**: BLE mesh transport (optionally depends on eche-lite)
- **eche-mesh**: Full Eche mesh library (uses eche-lite for embedded nodes)
