#![no_main]
use libfuzzer_sys::fuzz_target;
use peat_lite::CannedMessageEvent;

fuzz_target!(|data: &[u8]| {
    // Fuzz the unsigned decode path
    if let Some(event) = CannedMessageEvent::decode(data) {
        // Roundtrip: encode and decode again, must match
        let encoded = event.encode();
        let decoded = CannedMessageEvent::decode(&encoded)
            .expect("roundtrip decode must succeed");
        assert_eq!(event.encode(), decoded.encode());
    }

    // Fuzz the auto-detect decode path (unsigned or signed)
    let _ = CannedMessageEvent::decode_auto(data);
});
