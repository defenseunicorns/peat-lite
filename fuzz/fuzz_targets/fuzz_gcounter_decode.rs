#![no_main]
use libfuzzer_sys::fuzz_target;
use peat_lite::GCounter;

fuzz_target!(|data: &[u8]| {
    // Use default capacity of 32 nodes
    if let Some(counter) = GCounter::<32>::decode(data) {
        // Roundtrip: encode and decode again, must match
        let encoded = counter.encode();
        let decoded: GCounter<32> = GCounter::decode(&encoded)
            .expect("roundtrip decode must succeed");
        assert_eq!(counter.encode(), decoded.encode());
    }
});
