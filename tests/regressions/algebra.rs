use rzozowski::{Count, Regex};

use super::allocations::count_allocations;

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

#[test]
fn nested_lower_bounds_match_without_state_growth() {
    let regex = Regex::new("(((a){2,}){2,}){2,}").unwrap();
    assert!(!regex.matches(&"a".repeat(7)));
    let text = "a".repeat(12);
    let (matched, allocations) = count_allocations(|| regex.matches(&text));
    assert!(matched);
    // Leave headroom for implementation changes while rejecting derivative churn.
    assert!(
        allocations <= 1_000,
        "matching made {allocations} allocations"
    );
}

#[test]
fn zero_minimum_outer_count_preserves_the_gap() {
    let regex = Regex::new("((a){2,})*").unwrap();
    assert!(regex.matches(""));
    assert!(!regex.matches("a"));
    assert!(regex.matches("aa"));
    assert!(regex.matches("aaa"));
}

#[test]
fn nested_nullable_lower_bounds_collapse() {
    let regex = Regex::new("((a*){2,})*").unwrap();
    let text = "a".repeat(8);
    let (matched, allocations) = count_allocations(|| regex.matches(&text));
    assert!(matched);
    assert!(
        allocations <= 1_000,
        "matching made {allocations} allocations"
    );
}
