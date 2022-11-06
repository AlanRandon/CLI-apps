#![warn(clippy::pedantic, clippy::nursery)]

use clap::{Parser, Subcommand};
use itertools::Itertools;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
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
                    .then_some(path)
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

static SEARCH_PROGRAM: OnceCell<OsString> = OnceCell::new();
static SEARCH_REGEX: OnceCell<Regex> = OnceCell::new();

impl Action {
    #[allow(clippy::type_complexity)]
    fn run(&self) -> AnyResult<()> {
        let (should_search_all, mut check_path_callback): (
            _,
            Box<dyn FnMut(&Path) -> Option<String>>,
        ) = match self {
            Self::All { name } => (
                true,
                Box::new(|path| {
                    path_matches_program_name(
                        path,
                        SEARCH_PROGRAM.get_or_init(|| OsString::from(name.clone())),
                    )
                }),
            ),
            Self::First { name } => (
                false,
                Box::new(|path| {
                    path_matches_program_name(
                        path,
                        SEARCH_PROGRAM.get_or_init(|| OsString::from(name.clone())),
                    )
                }),
            ),
            Self::AllRegex { pattern } => (
                true,
                Box::new(|path| {
                    path_matches_regex(
                        path,
                        SEARCH_REGEX.get_or_init(|| Regex::new(pattern).expect("Invaild regex")),
                    )
                }),
            ),
            Self::FirstRegex { pattern } => (
                false,
                Box::new(|path| {
                    path_matches_regex(
                        path,
                        SEARCH_REGEX.get_or_init(|| Regex::new(pattern).expect("Invaild regex")),
                    )
                }),
            ),
        };
        let programs = get_programs()?;
        for path in programs {
            if let Some(path) = check_path_callback(&path) {
                println!("{}", path);
                if !should_search_all {
                    break;
                }
            }
        }
        Ok(())
    }
}

fn main() -> AnyResult<()> {
    Cli::parse().action.run()?;
    Ok(())
}
