use nom::{
    bytes::complete::{ tag, take_while1 },
    sequence::{ delimited, pair, preceded, terminated },
    IResult,
};

pub type Chars = ((u32, u32), Vec<u32>);

fn is_char_in_range(code: u32, range: &(u32, u32)) -> bool {
    code >= range.0 && code <= range.1
}

pub fn is_valid_char(c: char, (range, additional_chars): &Chars) -> bool {
    let code = c as u32;
    is_char_in_range(code, range) || additional_chars.contains(&code)
}

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

pub fn word_parser<'a>(input: &'a str, chars: &Chars) -> IResult<&'a str, &'a str> {
    take_while1(|c| is_valid_char(c, chars))(input)
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

pub fn enclosed_word_parser<'a>(
    input: &'a str,
    open: &'a str,
    close: &'a str,
    chars: &Chars
) -> IResult<&'a str, &'a str> {
    delimited(
        tag(open),
        take_while1(|c| is_valid_char(c, chars)),
        tag(close)
    )(input)
}
