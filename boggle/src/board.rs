use std::collections::HashSet;

use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use tui::widgets::{Cell, Row};

use crate::AnyResult;

const ADJACENT_INDICIES: [(i16, i16); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

lazy_static! {
    pub static ref DICE_FACES: Vec<Vec<&'static str>> = {
        include_str!("dice.txt")
            .split('\n')
            .map(|faces| faces.split(',').collect())
            .collect()
    };
}

fn get_letters(word: &str) -> Option<Vec<String>> {
    let mut letters = Vec::new();
    let word = word.to_uppercase();
    let mut chars = word.chars();

    while let Some(letter) = chars.next() {
        if letter == 'Q' {
            if chars.next() != Some('U') {
                return None;
            }
            letters.push("Qu".to_string());
        } else {
            letters.push(letter.to_string())
        };
    }

    Some(letters)
}

async fn is_real_word(word: String) -> AnyResult<bool> {
    Ok(reqwest::get(format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        word
    ))
    .await?
    .text()
    .await?
    .starts_with('['))
}

#[derive(Clone)]
pub struct BoggleBoard {
    letters: Vec<Vec<String>>,
}

impl BoggleBoard {
    pub fn new() -> Self {
        let mut dice_faces = DICE_FACES.clone();
        let mut rng = rand::thread_rng();

        dice_faces.shuffle(&mut rng);

        Self {
            letters: dice_faces
                .chunks(4)
                .map(|row| {
                    row.iter()
                        .map(|letters| letters.choose(&mut rng).unwrap().to_string())
                        .collect()
                })
                .collect(),
        }
    }

    pub fn to_rows(&self) -> Vec<Row> {
        self.letters
            .iter()
            .map(|row| Row::new(row.iter().map(|letter| Cell::from(letter.to_string()))))
            .collect()
    }

    /// tests if letters occur on this board
    pub fn test_letters(&self, test_word_letters: Vec<String>) -> bool {
        // A set of (list of coords of letters checked, list of letters left to check in word)
        let mut possible_matches = HashSet::new();

        // Find an inital match on the board
        for (x, row) in self.letters.iter().enumerate() {
            for (y, letter) in row.iter().enumerate() {
                if test_word_letters.first() == Some(letter) {
                    let word = test_word_letters[1..].to_vec();
                    possible_matches.insert((vec![(x as i16, y as i16)], word));
                }
            }
        }

        // While we havn't run out of potential matches
        while !possible_matches.is_empty() {
            // Check if any match has no letters left to check
            if possible_matches
                .iter()
                .any(|(_, test_word_letters)| test_word_letters.is_empty())
            {
                return true;
            }

            possible_matches = possible_matches
                .into_iter()
                .flat_map(|(positions, test_word_letters)| {
                    let mut possible_matches = HashSet::new();
                    let (x, y) = positions.last().unwrap();
                    for (x_change, y_change) in ADJACENT_INDICIES {
                        let x = x + x_change;
                        let y = y + y_change;

                        // If this coord has already been used by the word, ignore this possibility
                        if positions.contains(&(x, y)) {
                            continue;
                        }

                        let letter = self
                            .letters
                            .get(x as usize)
                            .and_then(|row| row.get(y as usize));

                        if test_word_letters.first() == letter {
                            let word = test_word_letters[1..].to_vec();
                            let mut positions = positions.clone();
                            positions.push((x, y));
                            possible_matches.insert((positions, word));
                        }
                    }
                    possible_matches
                })
                .collect()
        }

        false
    }

    pub async fn is_valid_word(&self, word: &str) -> AnyResult<bool> {
        if word.len() < 3 {
            return Ok(false);
        }

        let Some(letters) = get_letters(word) else {
            return Ok(false);
        };

        if self.test_letters(letters) {
            let word = word.to_string();
            is_real_word(word).await
        } else {
            Ok(false)
        }
    }
}
