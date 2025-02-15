# jsonish

**jsonish** is a resilient JSON parsing library written in Rust that goes beyond the strict JSON specification. It’s designed to parse not only valid JSON but also "JSON‐like" input that may include common syntax errors, multiple JSON objects, or JSON embedded in markdown code blocks.

---

## Usage

Here’s a simple example that demonstrates how to use jsonish to parse a JSON string:

```rust
use json_partial::jsonish::{parse, Value, ParseOptions};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Person {
    name: String,
    age: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A string given by LLM
    let input = r#"
    Here is your text 
    
    {
        "name": "Alice",
        "age": 30
    }
    "#;

    // Parse the JSON using default options.
    let value = parse(input, ParseOptions::default())?;
    println!("Parsed value: {:#?}", value);

    // Convert to serde_json::Value if needed.
    let serde_value = jsonish::jsonish_to_serde(&value);
    let person: Person = serde_json::from_value(serde_value).unwrap();
    println!("Serde JSON value: {}", serde_value);
    println!("Person: {:?}", person);

    Ok(())
}
```

### Parsing Imperfect JSON

jsonish is built to recover from common mistakes. For example:

- **Missing Commas:**  
  ```rust
  let input = r#"{ "name": "Bob" "age": 25 }"#;
  let value = parse(input, ParseOptions::default())?;
  // The parser applies fixes and still returns a valid result.
  ```

- **Unclosed Arrays or Objects:**  
  ```rust
  let input = r#"[1, 2, 3"#;
  let value = parse(input, ParseOptions::default())?;
  // Returns an array containing the numbers 1, 2, and 3.
  ```

- **JSON in Markdown Code Blocks:**  
  When given markdown text with fenced code blocks containing JSON, jsonish will extract and parse the JSON:

  ```rust
  let input = r#"
  ```json
  { "key": "value" }
  ```
  Some additional text.
  ```"#;

  let value = parse(input, ParseOptions::default())?;
  // The returned value will include a Markdown variant wrapping the parsed JSON.
  ```

---

## Features

- **Standard JSON Parsing:**  
  Uses `serde_json` under the hood to parse valid JSON strings quickly and reliably.

- **Error-Tolerant Parsing:**  
  When given imperfect JSON (e.g. missing commas, unquoted keys, unclosed arrays or objects), jsonish will attempt to fix and recover the input rather than immediately failing.

- **Markdown Code Block Extraction:**  
  Supports extracting and parsing JSON from markdown code blocks (e.g. fenced with triple backticks). This is especially useful when working with documents or logs that embed JSON in markdown.

- **Multi-Object Handling:**  
  Can detect and extract multiple JSON objects from a single input, returning them as a combined result.

- **Custom Value Representation:**  
  The parsed output is provided as a custom `Value` enum that includes variants for:
  - **Primitives:** Strings, Numbers, Booleans, and Null.
  - **Complex Structures:** Arrays and Objects.
  - **Special Cases:**  
    - `Markdown`: Represents a code block with a tag and its parsed inner value.
    - `FixedJson`: Wraps JSON that was fixed during parsing, along with a list of applied fixes.
    - `AnyOf`: Holds multiple possible parsed values (useful when multiple parsing strategies succeed).

- **Serde Conversion:**  
  Easily convert jsonish’s custom `Value` to a standard [`serde_json::Value`](https://docs.serde.rs/serde_json/) using the provided `jsonish_to_serde` function.

- **Configurable Parsing Options:**  
  Fine-tune the parsing behavior via the [`ParseOptions`](./jsonish/parser/mod.rs) struct, allowing you to enable or disable specific parsing strategies (e.g. markdown parsing, fixing errors, or treating input as a plain string).

---

## Installation

Add this library as a dependency in your `Cargo.toml`:

```toml
[dependencies]
json_partial = { git = "https://github.com/TwistingTwists/json_partial" }
```

## API Overview

- **`jsonish::parse`**  
  Main entry point for parsing a JSON (or JSON‐like) string. It applies a series of strategies:
  1. Attempt standard JSON parsing.
  2. If that fails and markdown JSON is allowed, try to extract and parse markdown code blocks.
  3. If enabled, attempt to locate multiple JSON objects.
  4. Apply automatic fixes for common syntax errors.
  5. Fallback to treating the input as a raw string if all else fails.

- **`jsonish::Value`**  
  A custom enum that represents the parsed JSON data with variants for primitive types, objects, arrays, markdown code blocks, fixed JSON (with applied fixes), and a collection of multiple possible parsed values.

- **`jsonish::ParseOptions`**  
  A configurable struct that controls which parsing strategies are enabled. It allows you to adjust settings like whether to allow markdown JSON, auto-fixing, multi-object parsing, and more.

- **`jsonish::to_serde::jsonish_to_serde`**  
  Converts a `jsonish::Value` into a [`serde_json::Value`](https://docs.serde.rs/serde_json/), making it easy to work with other libraries that use serde.

---

## Testing

jsonish comes with a comprehensive suite of tests that verify its ability to handle:

- Valid JSON objects
- JSON with missing commas or unclosed structures
- Nested JSON structures
- JSON embedded within markdown
- Multiple JSON objects within a single input

You can run the tests with:

```bash
cargo test
```

---

## Contributing

Contributions, bug reports, and feature requests are welcome! Feel free to open issues or submit pull requests on [GitHub](https://github.com/TwistingTwists/json_partial).

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## Thank you Note 

Lot of the code has been taken from [baml repository](https://github.com/BoundaryML/baml) -> [here](https://github.com/BoundaryML/baml/tree/03735feb5b9e70ad6a872e1c5d0837eea43034df/engine/baml-lib/jsonish/src/jsonish)

Thanks to awesome folks at Baml! 

*Happy parsing!*
