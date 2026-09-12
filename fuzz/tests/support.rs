#[path = "../fuzz_targets/support.rs"]
mod support;

use rzozowski::{CharRange, Count, Regex};
use support::{Bytes, Dialect};

#[test]
fn generated_patterns_are_valid_and_shared_witnesses_match() {
    for first in 0..=u8::MAX {
        let input = [first, 4, 5, 7, 8, 3, 0, 2, 9, 6, 1, 4, 0, 5, 2, 1];
        let (shared, witness) = support::pattern(&mut Bytes::new(&input), Dialect::Shared, 3);
        let reference = regex::Regex::new(&format!(r"\A(?:{shared})\z")).unwrap();
        assert!(reference.is_match(&witness), "{shared:?}, {witness:?}");
        assert!(Regex::new(&shared).is_ok(), "{shared:?}");

        let (native, _) = support::pattern(&mut Bytes::new(&input), Dialect::Native, 3);
        assert!(Regex::new(&native).is_ok(), "{native:?}");

        let mut bytes = Bytes::new(&input);
        let (tree, tree_witness) = support::tree(&mut bytes, 3);
        if let Some(tree_witness) = tree_witness {
            assert!(support::reference_matches(&tree, &tree_witness));
        }
        assert!(bytes.text().chars().count() <= 8);
    }
}

#[test]
fn reference_matcher_handles_nullable_repetitions_and_unicode_splits() {
    let optional_a = Regex::Or(Box::new(Regex::Epsilon), Box::new(Regex::Literal('a')));
    let twice = Regex::Count(Box::new(optional_a), Count::Exact(2));
    for text in ["", "a", "aa"] {
        assert!(support::reference_matches(&twice, text));
    }
    assert!(!support::reference_matches(&twice, "aaa"));
    let at_least_twice = Regex::Count(Box::new(Regex::Literal('a')), Count::AtLeast(2));
    assert!(!support::reference_matches(&at_least_twice, "a"));
    assert!(support::reference_matches(&at_least_twice, "aaa"));

    let unicode = Regex::Concat(
        Box::new(Regex::Literal('é')),
        Box::new(Regex::Class(vec![CharRange::Range('a', 'c')])),
    );
    assert!(support::reference_matches(&unicode, "éc"));
    assert!(!support::reference_matches(&unicode, "é"));
}
