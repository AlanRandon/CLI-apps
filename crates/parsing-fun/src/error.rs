use miette::Diagnostic;
use nom::Parser;
use nom_supreme::{error::ErrorTree, final_parser::final_parser};
use std::iter;
use thiserror::Error;

fn parser<'a, P, O>(mut parser: P) -> impl FnMut(&'a str) -> Result<O, ParseError>
where
    P: Parser<&'a str, O, ErrorTree<&'a str>>,
{
    move |input| {
        final_parser(|input| parser.parse(input))(input)
            .map_err(|error| ParseError::new(input, error))
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Parse Error")]
struct ParseError {
    #[source_code]
    source_code: String,
    #[related]
    annotations: Vec<Annotation>,
}

impl ParseError {
    fn new(source: &str, error_tree: ErrorTree<&str>) -> Self {
        let annotations = Annotation::new_iter(source, error_tree).collect();
        Self {
            source_code: source.to_string(),
            annotations,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Parse Error: {message}")]
struct Annotation {
    #[label("{message}")]
    location: (usize, usize),
    message: String,
}

impl Annotation {
    fn new_iter<'a>(
        source: &'a str,
        error_tree: ErrorTree<&'a str>,
    ) -> impl Iterator<Item = Self> + 'a {
        // location is the remaining input
        match error_tree {
            ErrorTree::Alt(error_trees) => Box::new(
                error_trees
                    .into_iter()
                    .flat_map(|error_tree| Self::new_iter(source, error_tree)),
            ) as Box<dyn Iterator<Item = Self>>,
            ErrorTree::Stack { base, contexts } => Box::new(
                Self::new_iter(source, *base).chain(
                    contexts
                        .into_iter()
                        .map(|(location, context)| Self::from_location(source, location, context)),
                ),
            ),
            ErrorTree::Base { location, kind } => {
                Box::new(iter::once(Self::from_location(source, location, kind)))
            }
        }
    }

    fn from_location(source: &str, location: &str, message: impl std::fmt::Display) -> Self {
        Self {
            location: if location.is_empty() {
                (0, source.len() + 2)
            } else {
                (source.len() - location.len(), location.len())
            },
            message: message.to_string(),
        }
    }
}
