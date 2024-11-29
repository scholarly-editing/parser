use nom::{
    bytes::complete::{ is_not, tag, take_until, take_until1, take_while1 },
    character::complete::{ digit1, line_ending, multispace0, multispace1, space0, space1 },
    sequence::{ pair, preceded, terminated, tuple },
    IResult,
};
use crate::config::Chars;

fn is_char_in_range(code: u32, range: &(u32, u32)) -> bool {
    code >= range.0 && code <= range.1
}

pub fn is_valid_char(c: char, (range, additional_chars): &Chars) -> bool {
    let code = c as u32;
    is_char_in_range(code, range) || additional_chars.contains(&code)
}

pub fn parse_line_break(input: &str) -> IResult<&str, &str> {
    line_ending(input)
}

pub fn parse_spaces(input: &str) -> IResult<&str, &str> {
    space1(input)
}

pub fn parse_default_page_break(input: &str) -> IResult<&str, (&str, (&str, &str), &str)> {
    tuple((space0, pair(digit1, line_ending), space0))(input)
}

pub fn parse_page_break_from_config<'a>(
    input: &'a str,
    prefix: &'a str,
    suffix: &'a str
) -> IResult<&'a str, (&'a str, ((&'a str, &'a str, &'a str), &'a str), &'a str)> {
    tuple((space0, pair(tuple((tag(prefix), digit1, tag(suffix))), line_ending), space0))(input)
}

pub fn parse_tag<'a>(seq: &'a str) -> Box<dyn (Fn(&str) -> IResult<&str, &str>) + 'a> {
    Box::new(move |input: &str| { tag(seq)(input) })
}

pub fn word_parser<'a>(input: &'a str, chars: &Chars) -> IResult<&'a str, &'a str> {
    take_while1(|c| is_valid_char(c, chars))(input)
}

pub fn word_suffixed_with_unwanted_chars_parser<'a>(
    input: &'a str,
    chars: &Chars
) -> IResult<&'a str, (&'a str, &'a str)> {
    tuple((
        take_while1(|c| is_valid_char(c, chars)),
        take_while1(|c: char| !c.is_whitespace() && !is_valid_char(c, chars)),
    ))(input)
}

pub fn word_prefixed_with_unwanted_chars_parser<'a>(
    input: &'a str,
    chars: &Chars
) -> IResult<&'a str, (&'a str, &'a str)> {
    tuple((
        take_while1(|c: char| !c.is_whitespace() && !is_valid_char(c, chars)),
        take_while1(|c| is_valid_char(c, chars)),
    ))(input)
}

pub fn word_infixed_with_unwanted_chars_parser<'a>(
    input: &'a str,
    chars: &Chars
) -> IResult<&'a str, (&'a str, &'a str, &'a str)> {
    tuple((
        take_while1(|c| is_valid_char(c, chars)),
        take_while1(|c: char| !c.is_whitespace() && !is_valid_char(c, chars)),
        take_while1(|c| is_valid_char(c, chars)),
    ))(input)
}

pub fn prefixed_word_parser<'a>(
    input: &'a str,
    prefix: &'a str,
    chars: &Chars
) -> IResult<&'a str, &'a str> {
    preceded(
        tag(prefix),
        take_while1(|c| is_valid_char(c, chars))
    )(input)
}

pub fn wrong_prefix_use_parser<'a>(
    input: &'a str,
    prefix: &'a str,
    chars: &Chars
) -> IResult<&'a str, (&'a str, &'a str, &'a str)> {
    tuple((tag(prefix), multispace1, take_while1(|c| is_valid_char(c, chars))))(input)
}

pub fn suffixed_word_parser<'a>(
    input: &'a str,
    suffix: &'a str,
    chars: &Chars
) -> IResult<&'a str, &'a str> {
    terminated(
        take_while1(|c| is_valid_char(c, chars)),
        tag(suffix)
    )(input)
}

pub fn wrong_suffix_use_parser<'a>(input: &'a str, suffix: &'a str) -> IResult<&'a str, &'a str> {
    tag(suffix)(input)
}

pub fn bracketed_text_parser<'a>(
    input: &'a str,
    open: &'a str,
    close: &'a str
) -> IResult<&'a str, (&'a str, &'a str, &'a str)> {
    tuple((tag(open), take_until1(close), tag(close)))(input)
}

pub fn empty_beackets_error_parser<'a>(
    input: &'a str,
    open: &'a str,
    close: &'a str
) -> IResult<&'a str, (&'a str, &'a str, &'a str)> {
    tuple((tag(open), multispace0, tag(close)))(input)
}

pub fn closing_bracket_without_opening_tag_parser<'a>(
    input: &'a str,
    open: &'a str,
    close: &'a str
) -> IResult<&'a str, (&'a str, &'a str, &'a str)> {
    tuple((multispace1, is_not(open), take_until(close)))(input)
}
