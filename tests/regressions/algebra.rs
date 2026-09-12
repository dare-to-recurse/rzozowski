use rzozowski::{Count, Regex};

#[test]
fn zero_repetitions_of_empty_still_match_empty() {
    // Reduced from 140 algebra artifacts to the zero-count simplification fault.
    for count in [Count::Exact(0), Count::Range(0, 0), Count::Range(0, 2)] {
        let repeated = Regex::Count(Box::new(Regex::Empty), count);
        assert_eq!(repeated.simplify(), Regex::Epsilon, "{count:?}");
        assert!(repeated.matches(""), "{count:?}");
        assert!(
            Regex::Concat(Box::new(Regex::Literal('a')), Box::new(repeated)).matches("a"),
            "{count:?}"
        );
    }
}
