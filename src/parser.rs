use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, none_of, u64},
    combinator::{all_consuming, cut, opt, value, verify},
    error::{context, VerboseError},
    multi::many0,
    sequence::{delimited, preceded, separated_pair, tuple},
    Err,
    IResult,
    Parser,
};

use crate::Pattern;

type Result<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

/// Parses a pattern.
pub(crate) fn pattern(input: &str) -> Result<Pattern> {
    let (input, list) = all_consuming(sequence).parse(input)?;

    Ok((input, list))
}

fn sequence(input: &str) -> Result<Pattern> {
    context(
        "sequence",
        many0(atom).map(|s| Pattern::Sequence(s.into()))
    )(input)
}

/// Any singular element of a pattern but not a sequence.
fn atom(input: &str) -> Result<Pattern> {
    let (mut input, mut exp) = context("atom", alt((
        group,
        class,
        literal,
    )))(input)?;

    loop {
        let res = alt((
            preceded(char('|'), cut(atom))
                .map(|b| Pattern::Or(exp.clone().into(), b.into())),
            quantifier.map(|(min, max)| Pattern::Quantification {
                pattern: exp.clone().into(),
                min,
                max
            })
        ))(input);

        (input, exp) = match res {
            Ok(a) => a,
            Err(Err::Failure(e)) => break Err(Err::Failure(e)),
            _ => break Ok((input, exp))
        }
    }
}

fn literal(input: &str) -> Result<Pattern> {
    context(
        "literal",
        alt((
            preceded(char('\\'), none_of("dDwWsSntr")),
            value('\n', tag("\\n")),
            value('\t', tag("\\t")),
            value('\r', tag("\\r")),
            none_of("\\()[]{}|?*+"),
        ))
    )
        .map(Pattern::Literal)
        .parse(input)
}

fn quantifier(input: &str) -> Result<(u64, u64)> {
    context(
        "quantifier",
        alt((
            value((0, 1), char('?')),
            value((0, u64::MAX), char('*')),
            value((1, u64::MAX), char('+')),
            delimited(
                char('{'),
                cut(alt((
                    separated_pair(cut(u64), char(','), cut(u64)),
                    u64.map(|a| (a, a)),
                ))),
                cut(char('}'))
            ),
        ))
    )(input)
}

/// Parses a group.
fn group(input: &str) -> Result<Pattern> {
    context(
        "group",
        delimited(char('('), cut(sequence), cut(char(')')))
    )
        .map(|s| Pattern::Group(s.into()))
        .parse(input)
}

/// Parses a character class.
fn class(input: &str) -> Result<Pattern> {
    delimited(
        char('['),
        cut(alt((
            preceded(char('^'), class_characters).map(Pattern::InvertedClass),
            class_characters.map(Pattern::Class),
        ))),
        cut(char(']'))
    )
        .parse(input)
}

fn class_characters(input: &str) -> Result<Box<[char]>> {
    tuple((
        opt(char(']')).map(|s| s.map_or(vec![], |c| vec![c])),
        opt(char('-')).map(|s| s.map_or(vec![], |c| vec![c])),
        many0(alt((
            range,
            none_of("-]").map(|c| [c].into()),
        ))).map(|a| a.into_iter().flatten()),
        opt(char('-')).map(|s| s.map_or(vec![], |c| vec![c])),
    ))
        .map(|a| a.0.into_iter().chain(a.1).chain(a.2).chain(a.3).collect())
        .parse(input)
}

/// Parses a range of characters.
fn range(input: &str) -> Result<Vec<char>> {
    context("range", verify(separated_pair(
        none_of("\\]"),
        char('-'),
        cut(none_of("\\]"))
    ), |(start, end)| start <= end))
        .map(|(start, end)| (start..=end).collect())
        .parse(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_atom() {
        assert_eq!(atom("a|b|c"), Ok(("", Pattern::Or(
            Pattern::Literal('a').into(),
            Pattern::Or(Pattern::Literal('b').into(),
                        Pattern::Literal('c').into()).into()
        ))));
    }

    #[test]
    fn parse_literal() {
        assert_eq!(literal("a"), Ok(("", Pattern::Literal('a'))));
        assert_eq!(literal("\\n"), Ok(("", Pattern::Literal('\n'))));
    }
}
