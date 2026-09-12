#![no_main]

use rzozowski::{CharRange, Count, Regex};

libfuzzer_sys::fuzz_target!(|input: (u8, u8, u8, char, char, char)| {
    let (min, max, length, start, end, probe) = input;
    let count = Regex::Count(
        Box::new(Regex::Literal('a')),
        Count::Range(usize::from(min), usize::from(max)),
    );
    let text = "a".repeat(usize::from(length % 17));
    assert_eq!(
        count.matches(&text),
        usize::from(min) <= text.len() && text.len() <= usize::from(max)
    );
    let _ = count.derivative('a');
    let _ = count.simplify();

    let class = Regex::Class(vec![CharRange::Range(start, end)]);
    assert_eq!(
        class.matches(&probe.to_string()),
        start <= probe && probe <= end
    );
    let _ = class.derivative(probe);
});
