use rzozowski::{CharRange, Count, Regex};
use std::collections::HashMap;

pub struct Bytes<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    fn next(&mut self) -> u8 {
        let byte = self.data.get(self.position).copied().unwrap_or(0);
        self.position += 1;
        byte
    }

    fn choose(&mut self, choices: u8) -> u8 {
        self.next() % choices
    }

    pub fn text(&mut self) -> String {
        const CHARS: &[char] = &[
            'a', 'b', 'c', '0', '1', '2', 'é', '💕', '[', '.', '\\', ' ', '\n', 'x',
        ];
        let length = usize::from(self.choose(9));
        (0..length)
            .map(|_| CHARS[usize::from(self.choose(CHARS.len() as u8))])
            .collect()
    }
}

#[derive(Clone, Copy)]
pub enum Dialect {
    Shared,
    Native,
}

// Generate a valid pattern and a string in its language. The small grammar
// reaches classes, escapes, counts, alternation, nesting and Unicode without
// requiring libFuzzer to invent balanced syntax by byte mutation alone.
pub fn pattern(bytes: &mut Bytes<'_>, dialect: Dialect, depth: u8) -> (String, String) {
    if depth == 0 {
        return atom(bytes, dialect);
    }

    match bytes.choose(8) {
        0..=3 => atom(bytes, dialect),
        4 => {
            let (left, left_text) = pattern(bytes, dialect, depth - 1);
            let (right, right_text) = pattern(bytes, dialect, depth - 1);
            (format!("({left})({right})"), left_text + &right_text)
        }
        5 => {
            let (left, left_text) = pattern(bytes, dialect, depth - 1);
            let (right, right_text) = pattern(bytes, dialect, depth - 1);
            let text = if bytes.choose(2) == 0 {
                left_text
            } else {
                right_text
            };
            (format!("({left}|{right})"), text)
        }
        _ => {
            let (inner, inner_text) = pattern(bytes, dialect, depth - 1);
            let (quantifier, repeats) = match bytes.choose(9) {
                0 => ("*", usize::from(bytes.choose(3))),
                1 => ("+", usize::from(bytes.choose(3)) + 1),
                2 => ("?", usize::from(bytes.choose(2))),
                3 => ("{0}", 0),
                4 => ("{1}", 1),
                5 => ("{2}", 2),
                6 => ("{0,2}", usize::from(bytes.choose(3))),
                7 => ("{1,2}", usize::from(bytes.choose(2)) + 1),
                _ => ("{2,}", usize::from(bytes.choose(2)) + 2),
            };
            (format!("({inner}){quantifier}"), inner_text.repeat(repeats))
        }
    }
}

fn atom(bytes: &mut Bytes<'_>, dialect: Dialect) -> (String, String) {
    const SHARED: &[(&str, &str)] = &[
        ("a", "a"),
        ("b", "b"),
        ("0", "0"),
        ("é", "é"),
        ("💕", "💕"),
        (r"\[", "["),
        (r"\.", "."),
        (r"\\", "\\"),
        ("[a-c]", "a"),
        ("[0-2]", "2"),
        ("[ab]", "b"),
    ];
    const NATIVE: &[(&str, &str)] = &[(r"\d", "0"), (r"\w", "_"), (r"\s", "\n")];

    let choice = usize::from(bytes.next())
        % (SHARED.len()
            + if matches!(dialect, Dialect::Native) {
                NATIVE.len()
            } else {
                0
            });
    let (pattern, text) = if choice < SHARED.len() {
        SHARED[choice]
    } else {
        NATIVE[choice - SHARED.len()]
    };
    (pattern.to_string(), text.to_string())
}

// Directly-built trees include constants and unsimplified expressions that the
// parser normally eliminates before the algebra target sees them.
pub fn tree(bytes: &mut Bytes<'_>, depth: u8) -> (Regex, Option<String>) {
    if depth == 0 {
        return tree_atom(bytes);
    }

    match bytes.choose(8) {
        0..=2 => tree_atom(bytes),
        3 => {
            let (left, left_text) = tree(bytes, depth - 1);
            let (right, right_text) = tree(bytes, depth - 1);
            let text = match (left_text, right_text) {
                (Some(left), Some(right)) => Some(if bytes.choose(2) == 0 { left } else { right }),
                (left, right) => left.or(right),
            };
            (Regex::Or(Box::new(left), Box::new(right)), text)
        }
        4 => {
            let (left, left_text) = tree(bytes, depth - 1);
            let (right, right_text) = tree(bytes, depth - 1);
            let text = left_text.zip(right_text).map(|(left, right)| left + &right);
            (Regex::Concat(Box::new(left), Box::new(right)), text)
        }
        5..=7 => {
            let kind = bytes.next() % 3;
            let min = usize::from(bytes.choose(4));
            let max = usize::from(bytes.choose(4));
            let (inner, inner_text) = tree(bytes, depth - 1);
            let count = match kind {
                0 => Count::Exact(min),
                1 => Count::Range(min, max),
                _ => Count::AtLeast(min),
            };
            let text = if kind == 1 && min > max {
                None
            } else if let Some(inner_text) = inner_text {
                let repetitions = match kind {
                    0 => min,
                    1 => min + usize::from(bytes.choose((max - min + 1) as u8)),
                    _ => min + usize::from(bytes.choose(3)),
                };
                Some(inner_text.repeat(repetitions))
            } else if min == 0 {
                Some(String::new())
            } else {
                None
            };
            (Regex::Count(Box::new(inner), count), text)
        }
        _ => unreachable!(),
    }
}

fn tree_atom(bytes: &mut Bytes<'_>) -> (Regex, Option<String>) {
    match bytes.choose(8) {
        0 => (Regex::Empty, None),
        1 => (Regex::Epsilon, Some(String::new())),
        2 => (Regex::Literal('a'), Some("a".to_string())),
        3 => (Regex::Literal('b'), Some("b".to_string())),
        4 => (Regex::Literal('é'), Some("é".to_string())),
        5 => (
            Regex::Class(vec![CharRange::Range('a', 'c')]),
            Some("c".to_string()),
        ),
        6 => (
            Regex::Class(vec![CharRange::Single('b'), CharRange::Range('0', '2')]),
            Some("2".to_string()),
        ),
        _ => (Regex::Class(vec![]), None),
    }
}

// An independent, bounded language oracle: concatenation tries all splits and
// repetition tracks reachable character boundaries. AtLeast needs at most
// min + text length repetitions, even when the inner expression is nullable.
pub fn reference_matches(regex: &Regex, text: &str) -> bool {
    reference_matches_cached(regex, text, &mut HashMap::new())
}

type Cache = HashMap<(*const Regex, *const u8, usize), bool>;

fn reference_matches_cached(regex: &Regex, text: &str, cache: &mut Cache) -> bool {
    let key = (regex as *const Regex, text.as_ptr(), text.len());
    if let Some(&result) = cache.get(&key) {
        return result;
    }

    let result = match regex {
        Regex::Empty => false,
        Regex::Epsilon => text.is_empty(),
        Regex::Literal(c) => text.chars().eq(std::iter::once(*c)),
        Regex::Class(ranges) => {
            let mut chars = text.chars();
            let Some(c) = chars.next() else { return false };
            chars.next().is_none()
                && ranges.iter().any(|range| match range {
                    CharRange::Single(ch) => *ch == c,
                    CharRange::Range(start, end) => *start <= c && c <= *end,
                })
        }
        Regex::Or(left, right) => {
            reference_matches_cached(left, text, cache)
                || reference_matches_cached(right, text, cache)
        }
        Regex::Concat(left, right) => boundaries(text).into_iter().any(|split| {
            reference_matches_cached(left, &text[..split], cache)
                && reference_matches_cached(right, &text[split..], cache)
        }),
        Regex::Count(inner, count) => {
            let min = match count {
                Count::Exact(n) | Count::AtLeast(n) => *n,
                Count::Range(min, _) => *min,
            };
            let max = match count {
                Count::Exact(n) => *n,
                Count::Range(_, max) => *max,
                Count::AtLeast(_) => min + text.chars().count(),
            };
            let positions = boundaries(text);
            let mut reachable = vec![0];
            for repetitions in 0..=max {
                if repetitions >= min && reachable.contains(&text.len()) {
                    return true;
                }
                if repetitions == max {
                    break;
                }
                let mut next = Vec::new();
                for start in reachable {
                    for &end in positions.iter().filter(|&&end| end >= start) {
                        if reference_matches_cached(inner, &text[start..end], cache)
                            && !next.contains(&end)
                        {
                            next.push(end);
                        }
                    }
                }
                reachable = next;
                if reachable.is_empty() {
                    break;
                }
            }
            false
        }
    };
    cache.insert(key, result);
    result
}

fn boundaries(text: &str) -> Vec<usize> {
    text.char_indices()
        .map(|(position, _)| position)
        .chain(std::iter::once(text.len()))
        .collect()
}
