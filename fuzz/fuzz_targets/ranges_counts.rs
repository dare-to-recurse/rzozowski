#![no_main]

use rzozowski::{CharRange, Count, Regex};

fn check_count(count: Count, lengths: &[usize]) {
    let value = Regex::Count(Box::new(Regex::Literal('a')), count);
    let simplified = value.simplify();
    for &length in lengths {
        let text = "a".repeat(length);
        let expected = match count {
            Count::Exact(n) => length == n,
            Count::Range(min, max) => min <= length && length <= max,
            Count::AtLeast(min) => min <= length,
        };
        assert_eq!(value.matches(&text), expected, "{count:?}, {length}");
        assert_eq!(simplified.matches(&text), expected, "{count:?}, {length}");
        if length > 0 {
            assert_eq!(
                value.derivative('a').matches(&text[1..]),
                expected,
                "{count:?}, {length}"
            );
        }
    }
    let nullable = match count {
        Count::Exact(n) => n == 0,
        Count::Range(min, _) | Count::AtLeast(min) => min == 0,
    };
    assert_eq!(value.is_nullable() == Regex::Epsilon, nullable);
}

libfuzzer_sys::fuzz_target!(|input: (u8, u8, u8, char, char, char)| {
    let (min, max, length, start, end, probe) = input;
    let small_min = usize::from(min % 9);
    let small_max = usize::from(max % 9);
    let mut lengths = vec![
        0,
        small_min.saturating_sub(1),
        small_min,
        small_min + 1,
        small_max,
        small_max + 1,
        usize::from(length % 11),
    ];
    lengths.sort_unstable();
    lengths.dedup();
    for count in [
        Count::Exact(small_min),
        Count::Range(small_min, small_max),
        Count::AtLeast(small_min),
    ] {
        check_count(count, &lengths);
    }
    check_count(
        Count::Range(usize::from(min), usize::from(max)),
        &[usize::from(length % 17)],
    );

    let class = Regex::Class(vec![CharRange::Range(start, end)]);
    let simplified = class.simplify();
    let mixed = Regex::Class(vec![
        CharRange::Single(probe),
        CharRange::Range(start, end),
        CharRange::Range(end, start),
    ]);
    let mixed_simplified = mixed.simplify();
    let mut probes = vec![probe, start, end, '\0', char::MAX];
    for boundary in [start, end] {
        if let Some(previous) = char::from_u32((boundary as u32).saturating_sub(1)) {
            probes.push(previous);
        }
        if let Some(next) = char::from_u32((boundary as u32).saturating_add(1)) {
            probes.push(next);
        }
    }
    for character in probes {
        let expected = start <= character && character <= end;
        let text = character.to_string();
        assert_eq!(class.matches(&text), expected);
        assert_eq!(simplified.matches(&text), expected);
        assert_eq!(class.derivative(character) == Regex::Epsilon, expected);
        let expected_mixed =
            character == probe || expected || (end <= character && character <= start);
        assert_eq!(mixed.matches(&text), expected_mixed);
        assert_eq!(mixed_simplified.matches(&text), expected_mixed);
    }
});
