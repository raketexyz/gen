use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, none_of, u64},
    combinator::{eof, value, verify},
    multi::{many0, many1},
    sequence::delimited,
    IResult,
    Parser,
};

use crate::Pattern;

/// Parses a pattern.
pub(crate) fn parse_pattern(input: &str) -> IResult<&str, Pattern> {
    let (input, (list, _)) = parse_sequence.and(eof).parse(input)?;

    Ok((input, list))
}

fn parse_sequence(input: &str) -> IResult<&str, Pattern> {
    many0(alt((parse_quantification, parse_subpattern)))
        .map(|s| Pattern::Sequence(s.into()))
        .parse(input)
}

/// Any singular element of a pattern but not a sequence.
fn parse_subpattern(input: &str) -> IResult<&str, Pattern> {
    alt((
        parse_character,
        parse_class,
        parse_group,
        //parse_or,
    ))(input)
}

/*///
fn parse_or(input: &str) -> IResult<&str, Pattern> {

}*/

/// Parses a group.
fn parse_group(input: &str) -> IResult<&str, Pattern> {
    delimited(
        char('('),
        parse_sequence,
        char(')')
    )(input)
}

/// Parses a literal character, escape code or wildcard.
fn parse_character(input: &str) -> IResult<&str, Pattern> {
    alt((
        value(Pattern::Wildcard, char('.')),
        value(Pattern::Class(('0'..='9').collect()),
              tag("\\d")),
        value(Pattern::Class(('a'..='z').chain('A'..='Z').chain('0'..='9')
                                 .chain("_".chars()).collect()),
              tag("\\w")),
        parse_escaped.map(Pattern::Character),
    ))
        .parse(input)
}

/// Parses a possibly escaped literal character.
fn parse_escaped(input: &str) -> IResult<&str, char> {
    alt((
        none_of("\\{}[]().|?+"),
        char('\\').and(alt((
            none_of("ntr"),
            value('\n', char('n')),
            value('\t', char('t')),
            value('\r', char('r')),
        ))).map(|(_, c)| c),
    ))
        .parse(input)
}

/// Parses a character class.
fn parse_class(input: &str) -> IResult<&str, Pattern> {
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

/// Parses a quantified pattern.
fn parse_quantification(input: &str) -> IResult<&str, Pattern> {
    parse_subpattern
        .and(delimited(char('{'), amount, char('}')))
        .map(|(pattern, (min, max))| Pattern::Quantification {
            pattern: pattern.into(),
            min,
            max,
        })
        .parse(input)
}

/// Parses the inner part of a quantification.
fn amount(input: &str) -> IResult<&str, (u64, u64)> {
    alt((
        u64.or(|a| Ok((a, 0))).and(char(',')).map(|(a, _)| a)
            .and(u64.or(|a| Ok((a, u64::MAX)))),
        u64.map(|a| (a, a)),
    ))
        .parse(input)
}

/// Parses the inner part of a character class.
fn class_characters(input: &str) -> IResult<&str, Box<[char]>> {
    let (input, buf) = many1(alt((
        parse_character_range,
        none_of("\\]-").map(|c| vec![c]),
        value(('a'..='z').chain('A'..='Z').chain('0'..='9')
                  .chain("_".chars()).collect(),
              tag("\\w")),
        value(('0'..='9').collect(), tag("\\d")),
    )))(input)?;
    let mut chars = Vec::new();

    for mut c in buf {
        chars.append(&mut c);
    }

    Ok((input, chars.into()))
}

/// Parses a range of characters.
fn parse_character_range(input: &str) -> IResult<&str, Vec<char>> {
    let (input, start) = none_of("\\")(input)?;
    let (input, _) = char('-')(input)?;
    let (input, end) = verify(none_of("\\"), |c| start <= *c)(input)?;

    Ok((input, (start..=end).collect()))
}
