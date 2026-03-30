#![no_main]
use libfuzzer_sys::fuzz_target;
use peat_lite::{decode_header, encode_header};

fuzz_target!(|data: &[u8]| {
    if let Ok((header, _rest)) = decode_header(data) {
        // Roundtrip: encode and decode again, must match
        let mut buf = [0u8; 16];
        encode_header(&header, &mut buf).expect("roundtrip encode must succeed");
        let (decoded, _) = decode_header(&buf).expect("roundtrip decode must succeed");
        assert_eq!(header, decoded);
    }
});
