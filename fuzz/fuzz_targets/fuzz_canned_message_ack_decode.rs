#![no_main]
use libfuzzer_sys::fuzz_target;
use peat_lite::CannedMessageAckEvent;

fuzz_target!(|data: &[u8]| {
    if let Some(ack) = CannedMessageAckEvent::decode(data) {
        // Roundtrip: encode and decode again, must match
        let encoded = ack.encode();
        let decoded = CannedMessageAckEvent::decode(&encoded)
            .expect("roundtrip decode must succeed");
        assert_eq!(ack.encode(), decoded.encode());
    }
});
