use rzozowski::{Count, Regex};

#[test]
fn rendering_and_reparsing_nested_counts_preserves_matching() {
    // The display artifacts expose the nullable-count fault through round-tripping.
    let original = Regex::Count(
        Box::new(Regex::Count(Box::new(Regex::Literal('a')), Count::Exact(0))),
        Count::Exact(2),
    );
    let rendered = original.to_string();
    let reparsed = Regex::new(&rendered).unwrap();
    assert!(reparsed.matches(""));
    assert_eq!(original.matches(""), reparsed.matches(""), "{rendered}");
}
