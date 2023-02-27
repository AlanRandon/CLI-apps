use clap::Args;
use crossterm::style::Color;
use nom::{
    branch::alt,
    character::complete::{digit1, satisfy},
    multi::count,
    IResult, Parser,
};
use nom_supreme::{
    error::ErrorTree, final_parser::final_parser, parser_ext::ParserExt, tag::complete::tag,
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

pub trait ToStringError {
    fn to_string_error(self) -> ErrorTree<String>;
}

impl<I> ToStringError for ErrorTree<I>
where
    I: ToString,
{
    fn to_string_error(self) -> ErrorTree<String> {
        self.map_locations(
            #[allow(clippy::redundant_clone)]
            {
                |i| i.to_string()
            },
        )
    }
}

fn parse_color(input: &str) -> Result<ColorWrapper, ErrorTree<String>> {
    final_parser(
        alt((
            named_color.context("named color"),
            ansi_color.context("ANSI color"),
            hex_color.context("hex color"),
        ))
        .map(ColorWrapper::new)
        .all_consuming(),
    )(input)
    .map_err(ErrorTree::<&str>::to_string_error)
}

fn ansi_color(input: &str) -> IResult<&str, Color, ErrorTree<&str>> {
    tag("ANSI-")
        .precedes(digit1)
        .map_res(|number: &str| number.parse().map(Color::AnsiValue))
        .parse(input)
}

fn hex_char(input: &str) -> IResult<&str, char, ErrorTree<&str>> {
    satisfy(|ch| ch.is_ascii_hexdigit())(input)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HexCodeKind {
    Long,
    Short,
}

fn hex_color(input: &str) -> IResult<&str, Color, ErrorTree<&str>> {
    let (input, _) = tag("#")(input)?;

    alt((
        count(count(hex_char, 2).recognize(), 3).map(|input| (input, HexCodeKind::Long)),
        count(hex_char.recognize(), 3).map(|input| (input, HexCodeKind::Short)),
    ))
    .map_res(|(input, kind)| {
        let mut input = input.into_iter();

        let [r, g, b] = [
            input.next().unwrap(),
            input.next().unwrap(),
            input.next().unwrap(),
        ]
        .map(|digit| {
            u8::from_str_radix(digit, 16).map(|digit| {
                if kind == HexCodeKind::Short {
                    digit * 16
                } else {
                    digit
                }
            })
        });

        Ok::<_, <u8 as FromStr>::Err>(Color::Rgb {
            r: r?,
            g: g?,
            b: b?,
        })
    })
    .parse(input)
}

fn named_color(input: &str) -> IResult<&str, Color, ErrorTree<&str>> {
    #[allow(clippy::enum_glob_use)]
    use Color::*;

    alt((
        tag("black").value(Black),
        tag("blue").value(Blue),
        tag("cyan").value(Cyan),
        tag("dark-blue").value(DarkBlue),
        tag("dark-cyan").value(DarkCyan),
        tag("dark-green").value(DarkGreen),
        tag("dark-grey").value(DarkGrey),
        tag("dark-magenta").value(DarkMagenta),
        tag("dark-red").value(DarkRed),
        tag("dark-yellow").value(DarkYellow),
        tag("green").value(Green),
        tag("grey").value(Grey),
        tag("magenta").value(Magenta),
        tag("red").value(Red),
        tag("white").value(White),
        tag("yellow").value(Yellow),
    ))
    .parse(input)
}
