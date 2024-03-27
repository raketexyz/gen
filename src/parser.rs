use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, none_of, u64},
    combinator::{all_consuming, opt, value, verify},
    multi::{many0, many1},
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
    Parser,
};

use crate::Pattern;

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
    let (mut input, mut exp) = alt((
        literal,
        class,
        group,
    ))(input)?;

    loop {
        let res = alt((
            preceded(char('|'), atom)
                .map(|b| Pattern::Or(exp.clone().into(), b.into())),
            quantifier.map(|(min, max)| Pattern::Quantification {
                pattern: exp.clone().into(),
                min,
                max
            })
        ))(input);

        (input, exp) = match res {
            Ok(a) => a,
            _ => break Ok((input, exp))
        }
    }
}

fn literal(input: &str) -> IResult<&str, Pattern> {
    alt((
        none_of("\\()[]{}"),
        preceded(char('\\'), none_of("dDwWsSntr")),
        value('\n', tag("\\n")),
        value('\t', tag("\\t")),
        value('\r', tag("\\r")),
    ))
        .map(Pattern::Literal)
        .parse(input)
}

fn quantifier(input: &str) -> IResult<&str, (u64, u64)> {
    alt((
        value((0, 1), char('?')),
        value((0, u64::MAX), char('*')),
        value((1, u64::MAX), char('+')),
        delimited(char('{'), separated_pair(u64, char(','), u64), char('}')),
    ))(input)
}

/// Parses a group.
fn group(input: &str) -> IResult<&str, Pattern> {
    delimited(char('('), sequence, char(')'))
        .map(|s| Pattern::Group(s.into()))
        .parse(input)
}

/// Parses a character class.
fn class(input: &str) -> IResult<&str, Pattern> {
    delimited(
        char('['),
        alt((
            preceded(char('^'), class_characters).map(Pattern::InvertedClass),
            class_characters.map(Pattern::Class),
        )),
        char(']')
    )
        .parse(input)
}

fn class_characters(input: &str) -> IResult<&str, Box<[char]>> {
    tuple((
        opt(char('-')).map(|s| s.map_or(vec![], |c| vec![c])),
        many1(alt((
            range,
            none_of("\\-").map(|c| [c].into()),
        ))).map(|a| a.into_iter().flatten()),
    ))
        .map(|a| a.0.into_iter().chain(a.1).collect())
        .parse(input)
}

/// Parses a range of characters.
fn range(input: &str) -> IResult<&str, Vec<char>> {
    let (input, (start, end)) = verify(separated_pair(
        none_of("\\"),
        char('-'),
        none_of("\\")
    ), |(start, end)| start <= end)(input)?;

    Ok((input, (start..=end).collect()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_literal() {
        assert_eq!(literal("a"), Ok(("", Pattern::Literal('a'))));
        assert_eq!(literal("\\n"), Ok(("", Pattern::Literal('\n'))));
    }
}
