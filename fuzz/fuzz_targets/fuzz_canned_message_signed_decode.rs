#![no_main]
use libfuzzer_sys::fuzz_target;
use peat_lite::CannedMessageEvent;

fuzz_target!(|data: &[u8]| {
    // Fuzz the signed decode path
    if let Some((event, sig)) = CannedMessageEvent::decode_signed(data) {
        // Roundtrip: encode_signed and decode_signed again
        let encoded = event.encode_signed(&sig);
        let (decoded, decoded_sig) = CannedMessageEvent::decode_signed(&encoded)
            .expect("roundtrip decode_signed must succeed");
        assert_eq!(event.encode(), decoded.encode());
        assert_eq!(sig, decoded_sig);
    }
});
