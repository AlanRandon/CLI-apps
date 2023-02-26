use clap::Args;
use crossterm::style::Color;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, satisfy},
    combinator::{complete, map_res, recognize, value},
    multi::count,
    sequence::preceded,
    IResult,
};
use std::str::FromStr;

#[derive(Args)]
pub struct Config {
    /// The height of the board (in rows)
    #[clap(short = 'r', long)]
    pub rows: Option<usize>,
    /// The width of the board (in columns)
    #[clap(short = 'c', long)]
    pub columns: Option<usize>,
    /// The color of an alive cell as a color (in the form ANSI-[n], #[r][g][b], or a named color)
    #[clap(long, value_parser = parse_color, default_value = "white")]
    pub alive_color: ColorWrapper,
    /// The color of an alive cell as a color (in the form ANSI-[n], #[r][g][b], or a named color)
    #[clap(long, value_parser = parse_color, default_value = "black")]
    pub dead_color: ColorWrapper,
}

#[derive(Clone)]
pub struct ColorWrapper {
    color: Color,
}

impl ColorWrapper {
    fn new(color: Color) -> Self {
        Self { color }
    }

    pub fn into_color(self) -> Color {
        self.color
    }
}

fn parse_color(input: &str) -> Result<ColorWrapper, nom::Err<nom::error::Error<String>>> {
    match complete(alt((ansi_color, named_color, hex_color)))(input) {
        Ok((_, result)) => Ok(ColorWrapper::new(result)),
        Err(err) => Err(err.to_owned()),
    }
}

fn ansi_color(input: &str) -> IResult<&str, Color> {
    map_res(preceded(tag("ANSI-"), digit1), |number: &str| {
        Ok::<_, <u8 as FromStr>::Err>(Color::AnsiValue(number.parse()?))
    })(input)
}

fn hex_char(input: &str) -> IResult<&str, char> {
    satisfy(|ch| ch.is_ascii_hexdigit())(input)
}

fn hex_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("#")(input)?;

    map_res(
        alt((
            count(recognize(count(hex_char, 2)), 3),
            count(recognize(hex_char), 3),
        )),
        |input: Vec<&str>| {
            let mut input = input.into_iter();

            Ok::<_, <u8 as FromStr>::Err>(Color::Rgb {
                r: u8::from_str_radix(input.next().unwrap(), 16)?,
                g: u8::from_str_radix(input.next().unwrap(), 16)?,
                b: u8::from_str_radix(input.next().unwrap(), 16)?,
            })
        },
    )(input)
}

fn named_color(input: &str) -> IResult<&str, Color> {
    alt((
        value(Color::Black, tag("black")),
        value(Color::Blue, tag("blue")),
        value(Color::Cyan, tag("cyan")),
        value(Color::DarkBlue, tag("dark-blue")),
        value(Color::DarkCyan, tag("dark-cyan")),
        value(Color::DarkGreen, tag("dark-green")),
        value(Color::DarkGrey, tag("dark-grey")),
        value(Color::DarkMagenta, tag("dark-magenta")),
        value(Color::DarkRed, tag("dark-red")),
        value(Color::DarkYellow, tag("dark-yellow")),
        value(Color::Green, tag("green")),
        value(Color::Grey, tag("grey")),
        value(Color::Magenta, tag("magenta")),
        value(Color::Red, tag("red")),
        value(Color::White, tag("white")),
        value(Color::Yellow, tag("yellow")),
    ))(input)
}
