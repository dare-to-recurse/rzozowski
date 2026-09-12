#![no_main]

libfuzzer_sys::fuzz_target!(|input: (String, String)| {
    let (pattern, text) = input;
    if pattern.len() > 256
        || text.len() > 256
        || !pattern
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"()|*+?".contains(&byte))
    {
        return;
    }

    let (Ok(candidate), Ok(reference)) = (
        rzozowski::Regex::new(&pattern),
        regex::Regex::new(&format!(r"\A(?:{pattern})\z")),
    ) else {
        return;
    };
    assert_eq!(candidate.matches(&text), reference.is_match(&text));
});
