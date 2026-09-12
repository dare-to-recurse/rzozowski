#![no_main]

#[allow(dead_code)]
mod support;

use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    // Continue interpreting existing corpus entries as the original tuple.
    if let Ok((pattern, text)) = <(String, String)>::arbitrary_take_rest(Unstructured::new(data)) {
        if pattern.len() <= 256
            && text.len() <= 256
            && pattern
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"()|*+?".contains(&byte))
        {
            if let (Ok(candidate), Ok(reference)) = (
                rzozowski::Regex::new(&pattern),
                regex::Regex::new(&format!(r"\A(?:{pattern})\z")),
            ) {
                assert_eq!(candidate.matches(&text), reference.is_match(&text));
            }
        }
    }

    if data.len() > 512 {
        return;
    }

    let mut bytes = support::Bytes::new(data);
    let (pattern, witness) = support::pattern(&mut bytes, support::Dialect::Shared, 3);
    let candidate = rzozowski::Regex::new(&pattern).expect(&pattern);
    let reference = regex::Regex::new(&format!(r"\A(?:{pattern})\z")).expect(&pattern);
    assert!(reference.is_match(&witness), "{pattern:?}, {witness:?}");

    let longer = format!("{witness}x");
    let sample = bytes.text();
    for text in [&witness, &longer, &sample] {
        assert_eq!(
            candidate.matches(text),
            reference.is_match(text),
            "{pattern:?}, {text:?}"
        );
    }
});
