#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() > 512 {
        return;
    }
    if let Ok(pattern) = std::str::from_utf8(data) {
        let _ = rzozowski::Regex::new(pattern);
    }
});
