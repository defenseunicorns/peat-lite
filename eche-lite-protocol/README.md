# eche-lite-protocol

Single source of truth for the Eche-Lite binary wire protocol (ADR-035).

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
eche-lite-protocol = "0.2"
```

```rust
use eche_lite_protocol::{MessageType, CrdtType, Header, decode_header, encode_header};
```

## Part of the Eche project

Eche is a mesh networking platform for tactical edge computing.
This crate is consumed by both embedded firmware (`eche-lite`) and
the hosted mesh library (`eche-mesh`).

License: Apache-2.0
