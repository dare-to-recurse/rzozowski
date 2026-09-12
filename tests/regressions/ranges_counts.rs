use rzozowski::{Count, Regex};

#[test]
fn zero_max_range_rejects_one_repetition() {
    // Minimized from fuzz/artifacts/ranges-counts/crash-554bb8566565e107f0f922e1feae5825f00e3582.
    let regex = Regex::Count(Box::new(Regex::Literal('a')), Count::Range(0, 0));

    assert!(!regex.matches("a"));
}
