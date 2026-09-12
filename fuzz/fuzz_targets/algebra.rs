#![no_main]

#[allow(dead_code)]
mod support;

use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use rzozowski::Regex;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if let Ok((pattern, text)) = <(String, String)>::arbitrary_take_rest(Unstructured::new(data)) {
        if pattern.len() <= 256 && text.len() <= 16 {
            if let Ok(value) = Regex::new(&pattern) {
                let simplified = value.simplify();
                assert_eq!(value.matches(&text), simplified.matches(&text));
                assert_eq!(simplified.simplify(), simplified);
            }
        }
    }

    if data.len() > 512 {
        return;
    }

    let mut bytes = support::Bytes::new(data);
    let (value, witness) = support::tree(&mut bytes, 3);
    let sample = bytes.text();
    let simplified = value.simplify();
    let mut texts = vec!["", "a", "b", sample.as_str()];
    if let Some(witness) = witness.as_ref().filter(|text| text.chars().count() <= 8) {
        assert!(
            support::reference_matches(&value, witness),
            "{value:?}, {witness:?}"
        );
        texts.push(witness);
    }

    for text in texts {
        let expected = support::reference_matches(&value, text);
        assert_eq!(value.matches(text), expected, "{value:?}, {text:?}");
        assert_eq!(simplified.matches(text), expected, "{value:?}, {text:?}");
        if let Some(first) = text.chars().next() {
            assert_eq!(
                value.derivative(first).matches(&text[first.len_utf8()..]),
                expected,
                "{value:?}, {text:?}"
            );
        }
    }
    assert_eq!(
        value.is_nullable() == Regex::Epsilon,
        support::reference_matches(&value, "")
    );
    assert_eq!(simplified.simplify(), simplified);
});
