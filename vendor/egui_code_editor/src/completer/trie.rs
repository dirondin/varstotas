#![allow(dead_code)]
use std::{iter::Peekable, str::Chars};

const ROOT_CHAR: char = ' ';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trie {
    root: char,
    is_word: bool,
    leaves: Vec<Trie>,
}

impl PartialOrd for Trie {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Trie {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.root.cmp(&other.root)
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self {
            root: ROOT_CHAR,
            is_word: false,
            leaves: vec![],
        }
    }
}

impl Trie {
    pub fn new(root: char) -> Self {
        Trie {
            root,
            ..Default::default()
        }
    }
    pub fn clear(&mut self) {
        self.leaves.clear();
    }
    pub fn push(&mut self, word: &str) {
        self.push_chars(&mut word.chars());
    }

    pub fn push_chars(&mut self, word: &mut Chars) {
        if let Some(first) = word.next() {
            if let Some(leaf) = self.leaves.iter_mut().find(|l| l.root == first) {
                leaf.push_chars(word)
            } else {
                let mut new = Trie::new(first);
                new.push_chars(word);
                self.leaves.push(new);
            }
        } else {
            self.is_word = true;
            self.leaves.sort();
            self.leaves.reverse();
        }
    }

    pub fn from_words(words: &[&str]) -> Self {
        let mut trie = Trie::new(ROOT_CHAR);
        words.iter().for_each(|w| {
            trie.push_chars(&mut w.chars());
        });
        trie
    }

    pub fn words(&self) -> Vec<String> {
        let mut words = vec![];
        for child in self.leaves.iter() {
            child.words_recursive("", &mut words);
        }
        words.reverse();
        words
    }
    fn words_recursive(&self, prefix: &str, words: &mut Vec<String>) {
        let mut prefix = prefix.to_string();
        prefix.push(self.root);
        if self.is_word {
            words.push(prefix.clone());
        }
        for child in self.leaves.iter() {
            child.words_recursive(&prefix, words);
        }
    }

    pub fn find_completions(&self, prefix: &str) -> Vec<String> {
        self.find_by_prefix(prefix)
            .map(|t| t.words())
            .unwrap_or_default()
    }
    pub fn find_by_prefix(&self, prefix: &str) -> Option<&Trie> {
        let mut found = None;
        let mut start = " ".to_string();
        start.push_str(prefix);
        let mut part = start.chars().peekable();
        self.find_recursive(&mut part, &mut found);
        found
    }
    fn find_recursive<'a>(&'a self, part: &mut Peekable<Chars>, found: &mut Option<&'a Trie>) {
        if let Some(c) = part.next()
            && self.root == c
        {
            if part.peek().is_none() {
                *found = Some(self);
            }
            self.leaves
                .iter()
                .for_each(|l| l.find_recursive(&mut part.clone(), found))
        }
    }
}
