pub mod jsonish;

#[cfg(test)]
mod tests {
    use super::jsonish;
    use jsonish::ParseOptions;

    #[test]
    fn test_valid_json_object() {
        let input = r#"{\"name\": \"Alice\", \"age\": 30}"#;
        assert!(jsonish::parse(input, ParseOptions::default()).is_ok());
    }

    #[test]
    fn test_invalid_json_missing_comma() {
        let input = r#"{\"name\": \"Bob\" \"age\": 25}"#;
        let value = jsonish::parse(input, ParseOptions::default());
        assert!(value.is_ok());
    }

    #[test]
    fn test_nested_json_structures() {
        let input = r#"{\"users\": [{\"id\": 1}, {\"id\": 2}]}"#;
        assert!(jsonish::parse(input, ParseOptions::default()).is_ok());
    }

    #[test]
    fn test_unclosed_array() {
        let input = r#"[1, 2, 3"#;
        assert!(jsonish::parse(input, ParseOptions::default()).is_ok());
    }

    // taken from commit https://github.com/boundaryml/baml/commit/8f758ef29cee811c124c234304d65bca281ee8d6
    #[test]
    fn test_unquotes_strings(){
        let input = r#"  { rec_one: "and then i said \"hi\", and also \"bye\"", rec_two: "and then i said "hi", and also "bye"", "also_rec_one": ok }"#;
        // let output = jsonish::parse(input, ParseOptions::default()).unwrap();
        // let printable = jsonish::jsonish_to_serde(&output);
        // println!("{printable:?}");
        assert!(jsonish::parse(input, ParseOptions::default()).is_ok());
    }
}
