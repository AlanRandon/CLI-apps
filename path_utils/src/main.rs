#![warn(clippy::pedantic, clippy::nursery)]

use clap::{Parser, Subcommand};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

lazy_static! {
    #[cfg(target_os = "windows")]
    static ref WINDOWS_PATHEXT: String = {
        env::var("PATHEXT")
            .expect("could not get windows PATHEXT")
            .split(';')
            .map(|ext| ext.trim_start_matches('.'))
            .join("|")
    };
    static ref FILE_EXT_IS_EXECUTABLE_REGEX: Regex =
        RegexBuilder::new(if cfg!(target_os = "windows") {
            &WINDOWS_PATHEXT
        } else {
            ".*"
        })
        .case_insensitive(true)
        .build()
        .unwrap();
}

/// A tool to analyze the $PATH environment variable
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Find all matches of the executable name in $PATH
    All {
        /// The name of the executable to search for
        #[arg(short, long)]
        name: String,
    },
    /// Find the first match of the executable name in $PATH
    First {
        /// The name of the executable to search for
        #[arg(short, long)]
        name: String,
    },
    /// Find all matches of the executable name in $PATH from a regex
    AllRegex {
        /// The pattern the executable name must match
        #[arg(short, long)]
        pattern: String,
    },
    /// Find the first match of the executable name in $PATH from a regex
    FirstRegex {
        /// The pattern the executable name must match
        #[arg(short, long)]
        pattern: String,
    },
}

type AnyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const PATH_SEPERATOR: &str = if cfg!(target_os = "windows") {
    ";"
} else {
    ":"
};

fn get_programs() -> AnyResult<Vec<PathBuf>> {
    Ok(env::var("PATH")?
        .split(PATH_SEPERATOR)
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(|path| {
            path.ok().and_then(|path| {
                let path = path.path();
                let file_ext = &path.extension()?.to_string_lossy();
                FILE_EXT_IS_EXECUTABLE_REGEX
                    .is_match(file_ext)
                    .then(|| path)
            })
        })
        .unique()
        .collect())
}

fn path_matches_program_name(path: &Path, search_name: &OsStr) -> Option<String> {
    path.file_stem().and_then(|name| {
        if search_name == name {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    })
}

fn path_matches_regex(path: &Path, search_pattern: &Regex) -> Option<String> {
    path.file_stem().and_then(|name| {
        let path_name = name.to_string_lossy().to_string();
        if search_pattern.is_match(&path_name) {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    })
}

impl Action {
    fn run(&self) -> AnyResult<()> {
        match self {
            Self::All { name } => {
                let search_program = OsString::from(name);
                println!("Searching for all occurences of {} in $PATH", name);
                let programs = get_programs()?;
                for path in programs {
                    if let Some(path) = path_matches_program_name(&path, &search_program) {
                        println!("{}", path);
                    }
                }
                return Ok(());
            }
            Self::First { name } => {
                let search_program = OsString::from(name);
                println!("Searching for first occurence of {} in $PATH", name);
                let programs = get_programs()?;
                for path in programs {
                    if let Some(path) = path_matches_program_name(&path, &search_program) {
                        println!("{}", path);
                        return Ok(());
                    }
                }
                println!("No items found.");
            }
            Self::AllRegex { pattern } => {
                let search_pattern = Regex::new(pattern)?;
                println!("Searching for all occurences of {} in $PATH", pattern);
                let programs = get_programs()?;
                for path in programs {
                    if let Some(path) = path_matches_regex(&path, &search_pattern) {
                        println!("{}", path);
                    }
                }
                return Ok(());
            }
            Self::FirstRegex { pattern } => {
                let search_pattern = Regex::new(pattern)?;
                println!("Searching for first occurence of {} in $PATH", pattern);
                let programs = get_programs()?;
                for path in programs {
                    if let Some(path) = path_matches_regex(&path, &search_pattern) {
                        println!("{}", path);
                        return Ok(());
                    }
                }
                println!("No items found.");
            }
        };
        Ok(())
    }
}

fn main() -> AnyResult<()> {
    Cli::parse().action.run()?;
    Ok(())
}
