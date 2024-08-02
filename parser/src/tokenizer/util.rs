use std::rc::Rc;

use nom::{
    bytes::complete::{ tag, take_while1 },
    sequence::{ delimited, pair, preceded, terminated },
    IResult,
};

pub fn line_break(input: &str) -> IResult<&str, &str> {
    take_while1(|c| c == '\n')(input)
}

pub fn spaces(input: &str) -> IResult<&str, &str> {
    take_while1(|c| c == ' ')(input)
}

fn find_page_break(input: &str) -> IResult<&str, (&str, &str)> {
    let is_valid_char = |c: char| c.is_ascii_alphanumeric();
    pair(tag("fol."), take_while1(is_valid_char))(input)
}

pub fn page_break(input: &str) -> IResult<&str, String> {
    let (rest, (marker, page_number)) = find_page_break(input)?;
    let repr = format!("{}{}", marker, page_number);
    Ok((rest, repr))
}

pub fn parse_sqeuence<'a>(seq: &'a str) -> Box<dyn (Fn(&str) -> IResult<&str, &str>) + 'a> {
    Box::new(move |input: &str| { tag(seq)(input) })
}

pub fn word_parser<'a>(
    input: &'a str,
    is_valid_char: &Rc<dyn Fn(char) -> bool>
) -> IResult<&'a str, &'a str> {
    take_while1(|c| is_valid_char(c))(input)
}

pub fn prefixed_word_parser<'a>(
    input: &'a str,
    prefix: &'a str,
    is_valid_char: &Rc<dyn Fn(char) -> bool>
) -> IResult<&'a str, &'a str> {
    preceded(
        tag(prefix),
        take_while1(|c| is_valid_char(c))
    )(input)
}

pub fn suffixed_word_parser<'a>(
    input: &'a str,
    suffix: &'a str,
    is_valid_char: &Rc<dyn Fn(char) -> bool>
) -> IResult<&'a str, &'a str> {
    terminated(
        take_while1(|c| is_valid_char(c)),
        tag(suffix)
    )(input)
}

pub fn enclosed_word_parser<'a>(
    input: &'a str,
    open: &'a str,
    close: &'a str,
    is_valid_char: &Rc<dyn Fn(char) -> bool>
) -> IResult<&'a str, &'a str> {
    delimited(
        tag(open),
        take_while1(|c| is_valid_char(c)),
        tag(close)
    )(input)
}
