use std::collections::HashMap;
use std::collections::hash_map;

pub struct WordMap {
    words: Vec<String>,
    index: HashMap<String, usize>,
}

impl WordMap {
    pub fn new() -> Self {
        WordMap {
            words: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn number(&mut self, word: &str) -> usize {
        match self.index.entry(word.to_owned()) {
            hash_map::Entry::Occupied(e) => *e.get(),
            hash_map::Entry::Vacant(e) => {
                let v = e.insert(self.words.len());
                self.words.push(word.to_owned());
                *v
            }
        }
    }

    pub fn word(&self, n: usize) -> Option<&str> {
        self.words.get(n).map(AsRef::as_ref)
    }
}
