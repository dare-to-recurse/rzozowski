#![no_main]

#[allow(dead_code)]
mod support;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() > 512 {
        return;
    }
    if let Ok(pattern) = std::str::from_utf8(data) {
        let _ = rzozowski::Regex::new(pattern);
    }

    let (generated, _) =
        support::pattern(&mut support::Bytes::new(data), support::Dialect::Native, 3);
    assert!(rzozowski::Regex::new(&generated).is_ok(), "{generated:?}");
});
