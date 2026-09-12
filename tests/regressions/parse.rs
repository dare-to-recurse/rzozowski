use rzozowski::Regex;

#[test]
fn overflowing_unterminated_count_returns_error() {
    // Minimized from fuzz/artifacts/parse/crash-eda11ce30c3aa75a59cad75067a8afa95d17bf02.
    let too_large = (usize::MAX as u128 + 1).to_string();
    let pattern = format!("a{{{too_large}");

    assert!(Regex::new(&pattern).is_err());
}
