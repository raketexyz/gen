use rand::{seq::IteratorRandom, Rng};

use crate::parser::parse_pattern;

/// A string pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// Two alternative patterns.
    Or(Box<Pattern>, Box<Pattern>),
    /// A group.
    Group(Box<Pattern>),
    /// A quantification of a pattern.
    Quantification {
        pattern: Box<Pattern>,
        min: u64,
        max: u64,
    },
    /// A sequence of patterns.
    Sequence(Box<[Pattern]>),
    /// A character.
    Character(char),
    /// A class of characters.
    Class(Box<[char]>),
    /// An inverted class of characters.
    InvertedClass(Box<[char]>),
    /// A wildcard.
    Wildcard,
}

impl Pattern {
    /// Parses a pattern.
    pub fn parse(input: &str)
    -> Result<Pattern, nom::Err<nom::error::Error<&str>>> {
        Ok(parse_pattern(input)?.1)
    }

    /// Generates a string matching this pattern.
    pub fn generate<R: Rng>(&self, rng: &mut R) -> String {
        match self {
            Self::Or(a, b) => if rng.gen_bool(0.5) { a } else { b }.generate(rng),
            Self::Group(pattern) => pattern.generate(rng),
            Self::Quantification { pattern, min, max } =>
                (0..rng.gen_range(*min..=*max))
                    .map(|_| pattern.generate(rng))
                    .collect(),
            Self::Sequence(patterns) => patterns.iter()
                .fold(String::new(), |s, p| s + &p.generate(rng)),
            Self::Character(c) => (*c).into(),
            Self::Class(c) => (*c.iter().choose(rng).unwrap()).into(),
            Self::InvertedClass(c) =>
                (char::from(0u8)..=char::from_u32(0x10ffff).unwrap())
                    .filter(|char| !c.contains(char))
                    .choose(rng).unwrap()
                    .into(),
            Self::Wildcard => rng.gen::<char>().to_string()
        }
    }
}