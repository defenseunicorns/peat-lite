# eche-lite

Lightweight CRDT primitives for resource-constrained Eche nodes.

## Overview

eche-lite provides bounded, `no_std`-compatible data structures suitable for devices with limited memory (256KB RAM budget):

- WearTAK on Samsung watches
- ESP32 sensor nodes
- LoRa mesh devices

## Primitives

| Type | Purpose | Memory |
|------|---------|--------|
| `NodeId` | 32-bit node identifier | 4 bytes |
| `CannedMessage` | Predefined message codes | 1 byte |
| `CannedMessageEvent` | Message with metadata | ~24 bytes |
| `CannedMessageStore` | LWW storage | ~6KB (256 entries) |
| `LwwRegister<T>` | Last-writer-wins register | sizeof(T) + 12 bytes |
| `GCounter` | Grow-only distributed counter | ~4 bytes per node |

## Usage

```rust
use eche_lite::{NodeId, CannedMessage, CannedMessageEvent};

let my_node = NodeId::new(0x12345678);
let event = CannedMessageEvent::new(
    CannedMessage::Ack,
    my_node,
    Some(NodeId::new(0xDEADBEEF)),  // target
    1706234567000,  // timestamp ms
);

// Encode for transmission (22 bytes)
let bytes = event.encode();
assert_eq!(bytes[0], 0xAF);  // CannedMessage marker
```

## CannedMessage Codes

```
0x00-0x0F  Acknowledgments   ACK, WILCO, NEGATIVE, SAY AGAIN
0x10-0x1F  Status            CHECK IN, MOVING, HOLDING, ON STATION, RTB, COMPLETE
0x20-0x2F  Alerts            EMERGENCY, ALERT, ALL CLEAR, CONTACT, UNDER FIRE
0x30-0x3F  Requests          NEED EXTRACT, NEED SUPPORT, MEDIC, RESUPPLY
0xF0-0xFF  Reserved          Custom/application-specific
```

## Wire Protocol

The `protocol` submodule provides the canonical Eche-Lite binary wire protocol (ADR-035):

- 16-byte fixed header (magic, version, type, flags, node ID, seq num)
- Message types: Announce, Heartbeat, Data, Query, Ack, Leave, OTA (0x10-0x16)
- CRDT types: LwwRegister, GCounter, PnCounter, OrSet
- Node capability flags for Full/Lite negotiation
- TTL suffix support for data expiry

All protocol types are re-exported at the crate root for ergonomic access:

```rust
use eche_lite::{MessageType, Header, encode_header, NodeCapabilities};
```

## Features

- **`std`** (default): Standard library support
- Disable for `no_std`: `--no-default-features`

```toml
# Cargo.toml - embedded usage
[dependencies]
eche-lite = { version = "0.2", default-features = false }
```

## Building

```bash
# With std (default)
cargo build

# For embedded (no_std)
cargo build --no-default-features

# Run tests
cargo test
```

## License

Apache-2.0

## OTA Updates (ESP32)

eche-lite supports over-the-air firmware updates on ESP32 targets with A/B partitioning:

- Streaming SHA256 verification during transfer
- Ed25519 signature verification (optional, compile-time)
- Boot validation with automatic rollback (3 attempts)
- Stop-and-wait reliable transfer over UDP

Build with OTA support:

```bash
SSID="your-ssid" PWD="your-password" cargo +esp build --release \
  --features m5stack-core2-wifi --target xtensa-esp32-none-elf --bin eche-lite-wifi
```
