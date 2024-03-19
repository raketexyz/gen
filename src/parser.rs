use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, none_of, u64},
    combinator::{all_consuming, value, verify},
    multi::{many0, many1},
    sequence::delimited,
    IResult,
    Parser,
};

use crate::Pattern;

enum Quantifier {
    Plus,
}

/// Parses a pattern.
pub(crate) fn pattern(input: &str) -> IResult<&str, Pattern> {
    let (input, list) = all_consuming(sequence).parse(input)?;

    Ok((input, list))
}

fn sequence(input: &str) -> IResult<&str, Pattern> {
    many0(atom)
        .map(|s| Pattern::Sequence(s.into()))
        .parse(input)
}

/// Any singular element of a pattern but not a sequence.
fn atom(input: &str) -> IResult<&str, Pattern> {
    let exp = alt((
        literal,
        class,
        group,
    ))(input)?;

    alt((char('|').and(atom), quantifier))
}

fn literal(input: &str) -> IResult<&str, char> {
    none_of("\\").or(value('\t', tag("\\t"))).parse(input)
}

fn quantifier(input: &str) -> IResult<&str, Quantifier> {
    value(Quantifier::Plus, char('+'))
}

/// Parses a binary or.
fn or(input: &str) -> IResult<&str, Pattern> {
    separated_pair(, sep, second)
        .map(|a, b| Pattern::Or(a, b))
        .parse(input)
}

/// Parses a group.
fn group(input: &str) -> IResult<&str, Pattern> {
    delimited(char('('), sequence, char(')'))(input)
}

/// Parses a character class.
fn class(input: &str) -> IResult<&str, Pattern> {
    delimited(
        char('['),
        alt((
            char('^').and(class_characters).map(|(_, c)| Pattern::InvertedClass(c)),
            class_characters.map(Pattern::Class),
        )),
        char(']')
    )
        .parse(input)
}

/// Parses a range of characters.
fn range(input: &str) -> IResult<&str, Vec<char>> {
    let (input, start) = none_of("\\")(input)?;
    let (input, _) = char('-')(input)?;
    let (input, end) = verify(none_of("\\"), |c| start <= *c)(input)?;

    Ok((input, (start..=end).collect()))
}
