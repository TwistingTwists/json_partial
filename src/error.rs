use thiserror::Error;
use regex::Error as RegexError;
use serde_json::Error as SerdeError;

#[derive(Error, Debug)]
pub enum JsonishError {
    #[error("Parsing error: {0}")]
    ParseError(String),

    #[error("Regex error: {0}")]
    RegexError(#[from] RegexError),

    #[error("Serde error: {0}")]
    SerdeError(#[from] SerdeError),

    // You can add other variants as needed:
    #[error("Mismatched brackets")]
    MismatchedBrackets,

    #[error("Unexpected state: {0}")]
    UnexpectedState(String),

    // A catch-all variant, if needed:
    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, JsonishError>;
````

src/jsonish/parser/entry.rs
````rust
<<<<<<< SEARCH
use anyhow::Result;

use crate::jsonish::{
    parser::{
        fixing_parser,
