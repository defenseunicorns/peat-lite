# hive-lite-protocol

Single source of truth for the HIVE-Lite binary wire protocol (ADR-035).

`#![no_std]`, zero-dependency crate providing:

- **Constants** — magic bytes, protocol version, default port, header/packet sizes
- **MessageType** / **CrdtType** enums with `from_u8` conversion
- **Header codec** — `encode_header` / `decode_header` for the fixed 16-byte packet header
- **NodeCapabilities** — capability bitflags for handshake negotiation
- **OTA constants** — chunk sizes, flags, result/abort codes for firmware updates
- **TTL helpers** — `append_ttl` / `strip_ttl` for data expiry

## Usage

```toml
[dependencies]
hive-lite-protocol = "0.1"
```

```rust
use hive_lite_protocol::{MessageType, CrdtType, Header, decode_header, encode_header};
```

## Part of the HIVE project

HIVE is a mesh networking platform for tactical edge computing.
This crate is consumed by both embedded firmware (`hive-lite`) and
the hosted mesh library (`hive-mesh`).

License: Apache-2.0
