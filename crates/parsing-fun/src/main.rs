use nom::{
    bytes::complete::take, character::complete::anychar, combinator::map_opt, multi::many1,
    IResult, Parser,
};
use nom_supreme::ParserExt;

mod error;

trait Parse<O = Self> {
    fn parse(input: &str) -> IResult<&str, O>;
}

trait Base {
    const DIGITS: &'static [char];
}

impl<T> Parse<u8> for T
where
    T: Base,
{
    fn parse(input: &str) -> IResult<&str, usize> {
        map_opt(anychar, |character| {
            Self::DIGITS
                .into_iter()
                .position(|digit| *digit == character)
        })(input)
    }
}

struct Base10;

struct Number(f32);

impl Parse for Number {
    fn parse(input: &str) -> IResult<&str, Self> {}
}

fn main() {
    println!("Hello World");
}
