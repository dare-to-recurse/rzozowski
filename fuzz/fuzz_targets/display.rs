#![no_main]

#[allow(dead_code)]
mod support;

use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use rzozowski::Regex;

fn has_constants(regex: &Regex) -> bool {
    match regex {
        Regex::Empty | Regex::Epsilon => true,
        Regex::Concat(left, right) | Regex::Or(left, right) => {
            has_constants(left) || has_constants(right)
        }
        Regex::Count(inner, _) => has_constants(inner),
        _ => false,
    }
}

fn check(value: &Regex, text: &str) {
    let rendered = value.to_string();
    assert!(!rendered.is_empty());
    let _ = value.derivative('a').to_string();

    // Empty and Epsilon intentionally render as mathematical symbols rather
    // than parseable regex syntax.
    if !has_constants(value) {
        let reparsed = Regex::new(&rendered).expect(&rendered);
        for text in ["", "a", "é", text] {
            assert_eq!(
                value.matches(text),
                reparsed.matches(text),
                "{rendered:?}, {text:?}"
            );
        }
    }
}

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if let Ok(pattern) = String::arbitrary_take_rest(Unstructured::new(data)) {
        if pattern.len() <= 512 {
            if let Ok(value) = Regex::new(&pattern) {
                check(&value, "b");
            }
        }
    }

    if data.len() > 512 {
        return;
    }

    let mut bytes = support::Bytes::new(data);
    let (pattern, witness) = support::pattern(&mut bytes, support::Dialect::Native, 3);
    let value = Regex::new(&pattern).expect(&pattern);
    check(&value, &witness);
    let (tree, tree_witness) = support::tree(&mut bytes, 2);
    check(&tree, tree_witness.as_deref().unwrap_or("b"));
});
