use nom::bytes::complete::take_until1;
use nom::error::Error;

fn main() {
    let example = "SOMETHING {{SOMETHING}}";

    let check = take_until1::<&str, &str, Error<&str>>("(")(&example);
    dbg!(check);
}
