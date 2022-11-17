use lazy_static::lazy_static;
use std::{collections::HashSet, io, sync::RwLock};
use ui::WordEntryResult;

// Dictionary API >> https://dictionaryapi.dev/

lazy_static! {
    pub static ref WORDS: RwLock<HashSet<String>> = RwLock::new(HashSet::new());
}

mod ui;

fn main() -> Result<(), io::Error> {
    ui::ui(|word| {
        if word.contains('s') {
            WordEntryResult::InvalidWord
        } else {
            WORDS.write().unwrap().insert(word);
            WordEntryResult::Valid
        }
    })
}
