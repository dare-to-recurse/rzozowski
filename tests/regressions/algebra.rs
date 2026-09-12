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

#[test]
fn repeated_nullable_operand_still_matches_empty() {
    // The original Count checks only tested whether the minimum was zero.
    let optional_a = Regex::Or(Box::new(Regex::Epsilon), Box::new(Regex::Literal('a')));
    for operand in [Regex::Epsilon, optional_a] {
        for count in [Count::Exact(1), Count::Range(1, 3), Count::AtLeast(1)] {
            let repeated = Regex::Count(Box::new(operand.clone()), count);
            assert_eq!(
                repeated.is_nullable(),
                Regex::Epsilon,
                "{operand:?}, {count:?}"
            );
            assert!(repeated.matches(""), "{operand:?}, {count:?}");
        }
    }
}

#[test]
fn reversed_count_range_does_not_become_epsilon() {
    // A range with min > max cannot choose any repetition count.
    let repeated = Regex::Count(Box::new(Regex::Epsilon), Count::Range(1, 0));
    assert_eq!(repeated.simplify(), Regex::Empty);
    assert!(!repeated.matches(""));
}
