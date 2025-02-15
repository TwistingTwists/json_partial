// mod iterative_parser;

// #[cfg(test)]
// mod test_iterative_parser;

mod value;
pub use value::{Fixes, Value};

// pub use iterative_parser::{parse_jsonish_value, JSONishOptions};
mod parser;
pub use parser::{parse, ParseOptions};

mod to_serde;
pub use to_serde::jsonish_to_serde;

/// Converts a `jsonish::Value` into a compact JSON string.
///
/// Returns an error if serialization via `serde_json` fails.
pub fn to_json_string(value: &Value) -> Result<String, serde_json::Error> {
    let serde_value = jsonish_to_serde(value);
    serde_json::to_string(&serde_value)
}

/// Converts a `jsonish::Value` into a pretty printed JSON string.
///
/// Returns an error if serialization via `serde_json` fails.
pub fn to_json_string_pretty(value: &Value) -> Result<String, serde_json::Error> {
    let serde_value = jsonish_to_serde(value);
    serde_json::to_string_pretty(&serde_value)
}

#[cfg(test)]
mod tests {
    // Bring the conversion functions and jsonish module into scope.
    use super::{parse, to_json_string, ParseOptions};

    /// Test that `to_json_string` converts a malformed JSON (with missing comma,
    /// misnamed key, superfluous key, and extra whitespace) into a compact JSON string.
    #[test]
    fn test_to_json_string_compact() {
        let input = r#" Here is your json 
        ```json
        {naem:"Alice", age:30, extra:"remove me", yap:"   noisy message   "}
        ```
        "#;
        let value = parse(input, ParseOptions::default())
            .expect("Parser should handle the errors and return a Value");
        let json_output = to_json_string(&value).expect("to_json_string conversion should succeed");

        // And that keys are output in sorted order, the expected compact JSON is:
        let expected =
            r#"{"age":30,"naem":"Alice", extra:"remove me","yap":"   noisy message   "}"#;
        assert_eq!(json_output, expected);
    }
}
