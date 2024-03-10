use crate::search::{Item, Language, Query, Search};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref WORD_REGEX: Regex = Regex::new(r"[a-zA-Zéèàùâêôîûëïüÿç]+").unwrap();
}

pub struct Part {
    pub string: String,
    pub matched: String,
    pub item: Item,
}

pub fn explain(string: &str, search: &Search) -> Vec<Part> {
    let mut parts = vec![];
    for cap in WORD_REGEX.captures_iter(string) {
        let cap_string = cap.get(0).unwrap().as_str().to_string();
        let result = search.search(&Query::new(&cap_string, &Language::French, 0b1111111111111111), 1);
        parts.push(Part { string: cap_string, matched: result[0].0.clone(), item: result[0].1.clone() });
    }
    parts
}