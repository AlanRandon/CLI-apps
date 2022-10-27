use inquire::{
    autocompletion::{Autocomplete, Replacement},
    CustomUserError,
};
use std::{
    io,
    path::{self, Path, PathBuf},
};

pub fn get_subdirectories(path: &Path) -> io::Result<Vec<PathBuf>> {
    path.read_dir()?
        .filter_map(|child_path| match child_path {
            Ok(child_path) => {
                let path = child_path.path();
                if path.is_dir() {
                    Some(Ok(path))
                } else {
                    None
                }
            }
            Err(err) => Some(Err(err)),
        })
        .collect()
}

fn common_start_chars(path_1: &Path, path_2: &Path) -> usize {
    path_1
        .to_string_lossy()
        .to_string()
        .chars()
        .zip(path_2.to_string_lossy().to_string().chars())
        .take_while(|(a, b)| a == b)
        .count()
}

#[derive(Clone, Default)]
pub struct DirectoriesAutocomplete {
    input: String,
    paths: Vec<PathBuf>,
    suggested_path: String,
    search_dir: PathBuf,
    input_has_trailing_slash: bool,
}

impl DirectoriesAutocomplete {
    fn update(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input {
            return Ok(());
        }

        let input_path = PathBuf::from(input);

        if let Some(search_dir) = if input.ends_with(path::is_separator) {
            self.input_has_trailing_slash = true;
            Some(input_path.as_path())
        } else {
            self.input_has_trailing_slash = false;
            input_path.parent()
        } {
            self.search_dir = search_dir.to_path_buf();
            let mut subdirectories = get_subdirectories(search_dir).unwrap_or_default();
            subdirectories.sort_by(|a, b| {
                common_start_chars(b, &input_path).cmp(&common_start_chars(a, &input_path))
            });
            let subdirectories: Vec<_> = subdirectories.into_iter().collect();
            if let Some(suggested_path) = subdirectories.get(0) {
                self.suggested_path = suggested_path.to_string_lossy().to_string();
            };
            self.paths = subdirectories;
        };

        self.input = input.to_string();
        Ok(())
    }
}

impl Autocomplete for DirectoriesAutocomplete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        self.update(input)?;
        let mut suggestions: Vec<_> = self
            .paths
            .iter()
            .filter_map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
            })
            .collect();
        suggestions.splice(0..0, [String::from("..")]);
        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        self.update(input)?;

        Ok(match highlighted_suggestion {
            Some(suggestion) => {
                if suggestion == ".." {
                    match self.search_dir.parent() {
                        Some(path) => {
                            Replacement::Some(format!("{}{}", path.display(), path::MAIN_SEPARATOR))
                        }
                        None => Replacement::None,
                    }
                } else {
                    Replacement::Some(format!(
                        "{}{}{}{}",
                        self.search_dir.display(),
                        if self.input_has_trailing_slash {
                            String::new()
                        } else {
                            path::MAIN_SEPARATOR.to_string()
                        },
                        suggestion,
                        path::MAIN_SEPARATOR
                    ))
                }
            }
            None => match self.suggested_path.is_empty() {
                true => Replacement::None,
                false => Replacement::Some(self.suggested_path.clone()),
            },
        })
    }
}
