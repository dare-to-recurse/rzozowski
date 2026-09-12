#![no_main]

libfuzzer_sys::fuzz_target!(|pattern: String| {
    if pattern.len() > 512 {
        return;
    }
    if let Ok(value) = rzozowski::Regex::new(&pattern) {
        assert!(!value.to_string().is_empty());
    }
});
