use inquire::Text;
use std::{
    env, io,
    path::{self, PathBuf},
};

mod autocomplete;

type AnyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> AnyResult<()> {
    let working_directory = env::current_dir()?;
    let result = PathBuf::from(
        Text::new("Navigate to directory >")
            .with_initial_value(&format!(
                "{}{}",
                &working_directory.to_string_lossy(),
                path::MAIN_SEPARATOR
            ))
            .with_autocomplete(autocomplete::DirectoriesAutocomplete::default())
            .prompt()?,
    );
    if result.exists() {
        let options = vec!["Copy `cd` command", "Copy path"];
        let result = result.to_string_lossy().to_string();
        match inquire::Select::new("Output Method?", options).prompt()? {
            "Copy path" => {
                if cli_clipboard::set_contents(result.to_string()).is_ok() {
                    eprintln!("Copied {} to clipboard!", result);
                }
            }
            "Copy `cd` command" => {
                let command = format!("cd {}", result);
                if cli_clipboard::set_contents(command.to_string()).is_ok() {
                    eprintln!("Copied `{}` to clipboard!", command);
                }
            }
            _ => (),
        };
    } else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "This path does not exist!",
        )));
    }

    Ok(())
}
