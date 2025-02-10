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
