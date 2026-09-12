#![no_main]

libfuzzer_sys::fuzz_target!(|input: (String, String)| {
    let (pattern, text) = input;
    if pattern.len() > 256 || text.len() > 16 {
        return;
    }
    let Ok(value) = rzozowski::Regex::new(&pattern) else {
        return;
    };

    let simplified = value.simplify();
    assert_eq!(value.matches(&text), simplified.matches(&text));
    assert_eq!(simplified.simplify(), simplified);
    assert_eq!(
        value.is_nullable() == rzozowski::Regex::Epsilon,
        value.matches("")
    );
    if let Some(first) = text.chars().next() {
        assert_eq!(
            value.derivative(first).matches(&text[first.len_utf8()..]),
            value.matches(&text)
        );
    }
});
