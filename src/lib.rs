//! # gen
//! gen is a command-line utility for generating strings that match a given
//! pattern.
//! For example, you can use gen to generate passwords in your terminal:
//! `gen "[\w-]{20}"` returns a 20-character-long password containing
//! digits, uppercase and lowercase letters, dashes and underscores.
pub use self::pattern::*;

pub(crate) mod parser;

mod pattern;
