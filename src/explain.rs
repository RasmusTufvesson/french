use crate::search::{Item, Language, Query, Search};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref WORD_REGEX: Regex = Regex::new(r"[a-zA-Zéèàùâêôîûëïüÿç]+'?").unwrap();
}

pub struct Part {
    pub string: String,
    pub matched: Vec<(String, Item)>,
    pub chosen: usize,
    pub sure: bool
}

pub fn explain(string: &str, search: &Search) -> Vec<Part> {
    let mut parts = vec![];
    for cap in WORD_REGEX.captures_iter(string) {
        let mut cap_string = cap.get(0).unwrap().as_str().to_string();
        if cap_string.ends_with('\'') {
            cap_string.replace_range((cap_string.len()-1)..cap_string.len(), "e");
        }
        let result = search.search_best_answers(&Query::new(&cap_string, &Language::French, 0b1111111111111111, cap_string.ends_with('\'')));
        parts.push(Part { string: cap_string, matched: result.0.clone(), sure: result.1 == 0, chosen: 0 });
    }
    parts
}