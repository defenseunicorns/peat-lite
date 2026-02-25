//! Protocol-level constants.

/// Magic bytes identifying an Eche-Lite packet: ASCII "ECHE".
pub const MAGIC: [u8; 4] = [0x45, 0x43, 0x48, 0x45];

/// Protocol version for compatibility checking.
pub const PROTOCOL_VERSION: u8 = 1;

/// Default UDP port for Eche-Lite communication.
///
/// This is the canonical deployed value used by both eche-lite firmware
/// and eche-mesh transport.
pub const DEFAULT_PORT: u16 = 5555;

/// Default multicast address for discovery: 239.255.72.76 (H.L).
pub const MULTICAST_ADDR: [u8; 4] = [239, 255, 72, 76];

/// Fixed header size in bytes.
pub const HEADER_SIZE: usize = 16;

/// Maximum packet size (fits in a single UDP datagram).
pub const MAX_PACKET_SIZE: usize = 512;

/// Maximum payload size (packet minus header).
pub const MAX_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - HEADER_SIZE;

// --- TTL (Time-To-Live) constants ---

/// Header flag indicating a 4-byte TTL suffix is appended to the payload.
///
/// Does not conflict with `OTA_FLAG_SIGNED` (also 0x0001) because TTL flags
/// apply only to `Data` messages while OTA flags apply to OTA messages.
pub const FLAG_HAS_TTL: u16 = 0x0001;

/// Size of the TTL suffix in bytes (u32 little-endian).
pub const TTL_SUFFIX_SIZE: usize = 4;

/// Default TTL for LwwRegister: 5 minutes (sensor data goes stale quickly).
pub const DEFAULT_TTL_LWW_REGISTER: u32 = 300;

/// Default TTL for GCounter: 1 hour.
pub const DEFAULT_TTL_G_COUNTER: u32 = 3600;

/// Default TTL for PnCounter: 1 hour.
pub const DEFAULT_TTL_PN_COUNTER: u32 = 3600;

/// Sentinel: data never expires.
pub const TTL_NEVER_EXPIRES: u32 = 0;
