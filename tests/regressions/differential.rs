use rzozowski::Regex;

#[test]
fn parsed_nested_nullable_repetition_matches_empty_and_suffix() {
    // All differential artifacts reduce to repetition of a nullable inner regex.
    assert!(Regex::new("(a*){2}").unwrap().matches(""));
    assert!(Regex::new("((a*){1,2})b").unwrap().matches("b"));
}
