#![warn(clippy::pedantic, clippy::nursery)]

use std::{env, ffi::OsString, fs};

#[derive(clap::Parser)]
enum Args {}

#[derive(clap::Subcommand)]
enum Action {
    ListMatches,
    FindPath,
}

type AnyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const PATH_SEPERATOR: &str = if cfg!(target_os = "windows") {
    ";"
} else {
    ":"
};

fn main() -> AnyResult<()> {
    let path_contents = env::var("PATH")?;
    let search_program = OsString::from("python");
    let programs = path_contents
        .split(PATH_SEPERATOR)
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(|path| {
            if let Ok(path) = path {
                path.path().file_stem().and_then(|name| {
                    if search_program == name {
                        Some(path.path().to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        });
    println!("PATH = {:?}", programs.collect::<Vec<_>>());
    Ok(())
}
