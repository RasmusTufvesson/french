use std::{fmt::Display, fs::File, io::{Write, Read}};
use levenshtein::levenshtein;
use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gender {
    Female,
    Male,
}

impl Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Female => "Female",
            Self::Male => "Male",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerbForms {
    Regular(String, String, String, String, String, String),
    Irregular(String, String, String, String, String, String),
}

impl Display for VerbForms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Regular(..) => "Regular",
            Self::Irregular(..) => "Irregular",
        })
    }
}

impl VerbForms {
    pub fn gen_from_regular(string: &str) -> Option<(String, String, String, String, String, String)> {
        let base = if string.ends_with("e") {
            &string[0..string.len()-1]
        } else if string.ends_with("es") || string.ends_with("ez") {
            &string[0..string.len()-2]
        } else if string.ends_with("ons") || string.ends_with("ent") {
            &string[0..string.len()-3]
        } else {
            return None;
        };
        Some((base.to_string()+"e", base.to_string()+"es", base.to_string()+"e", base.to_string()+"ons", base.to_string()+"ez", base.to_string()+"ent"))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Category {
    Noun(String, Gender),
    Verb(String, VerbForms),
    Adjective(String, String),
    All(String),
}

impl Category {
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Noun(..) => 0b00000001,
            Self::Verb(..) => 0b00000010,
            Self::Adjective(..) => 0b00000100,
            Self::All(_) => 0b11111111,
        }
    }

    fn display_detailed(&self, english: &Option<String>, swedish: &Option<String>) -> String {
        let swedish = match swedish {
            Some(val) => val,
            None => "unknown",
        };
        let english = match english {
            Some(val) => val,
            None => "unknown",
        };
        match self {
            Self::All(string) => format!("{} ({}, {})", string, swedish, english),
            Self::Noun(string, gender) => {
                match gender {
                    Gender::Male => format!("{} ({}, {}), masculine noun", string, swedish, english),
                    Gender::Female => format!("{} ({}, {}), feminine noun", string, swedish, english),
                }
            }
            Self::Verb(name, forms) => {
                match forms {
                    VerbForms::Irregular(..) => format!("{} ({}, {}), irregular verb", name, swedish, english),
                    VerbForms::Regular(..) => format!("{} ({}, {}), regular verb", name, swedish, english),
                }
            }
            Self::Adjective(female, male) => format!("{}/{} ({}, {}), adjective", male, female, swedish, english)
        }
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::All(_) => "Other",
            Self::Noun(..) => "Noun",
            Self::Verb(..) => "Verb",
            Self::Adjective(..) => "Adjective",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub swedish: Option<String>,
    pub english: Option<String>,
    pub category: Category,
    category_int: u8,
}

impl Item {
    pub fn new(swedish: Option<String>, english: Option<String>, category: Category) -> Self {
        let category_int = category.to_u8();
        Self { swedish, english, category, category_int }
    }

    fn language_strings(&self, language: &Language) -> Option<Vec<String>> {
        match language {
            Language::French => {
                match &self.category {
                    Category::All(string) => Some(vec![string.to_owned()]),
                    Category::Adjective(female, male) => Some(vec![female.to_owned(), male.to_owned()]),
                    Category::Noun(string, _) => Some(vec![string.to_owned()]),
                    Category::Verb(base, form) => {
                        match form {
                            VerbForms::Regular(je, tu, _, nous, vous, ils) => Some(vec![je.to_owned(), tu.to_owned(), nous.to_owned(), vous.to_owned(), ils.to_owned(), base.to_owned()]),
                            VerbForms::Irregular(je, tu, il, nous, vous, ils) => Some(vec![je.to_owned(), tu.to_owned(), il.to_owned(), nous.to_owned(), vous.to_owned(), ils.to_owned(), base.to_owned()]),
                        }
                    }
                }
            }
            Language::Swedish => {
                match &self.swedish {
                    None => None,
                    Some(string) => Some(vec![string.to_owned()]),
                }
            }
            Language::English => {
                match &self.english {
                    None => None,
                    Some(string) => Some(vec![string.to_owned()]),
                }
            }
        }
    }

    pub fn tooltip(&self) -> String {
        self.category.display_detailed(&self.english, &self.swedish)
    }
}

#[derive(Debug, PartialEq)]
pub enum Language {
    French,
    Swedish,
    English,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::French => "French",
            Self::Swedish => "Swedish",
            Self::English => "English",
        })
    }
}

pub struct Query<'a> {
    string: &'a String,
    language: &'a Language,
    search_categories_int: u8,
}

impl<'a> Query<'a> {
    pub fn new(string: &'a String, language: &'a Language, search_categories_int: u8) -> Self {
        Self { string, language, search_categories_int }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Search {
    items: Vec<Item>,
}

impl Search {
    pub fn search(&self, query: &Query, num_answers: usize) -> Vec<(String, Item)> {
        let mut best_matches: Vec<(String, Item)> = Vec::with_capacity(num_answers);
        let mut best_match_scores: Vec<usize> = vec![usize::MAX; num_answers];

        for item in &self.items {
            if let Some(strings) = item.language_strings(&query.language) {
                if item.category_int & query.search_categories_int != 0 {
                    for string in strings {
                        let list_item = (string, item.clone());
                        if !best_matches.contains(&list_item) {
                            let distance = levenshtein(&query.string, &list_item.0);
                
                            for i in 0..num_answers {
                                if distance < best_match_scores[i] {
                                    best_match_scores.insert(i, distance);
                                    best_match_scores.truncate(num_answers);
                                    best_matches.insert(i, list_item);
                                    best_matches.truncate(num_answers);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        best_matches
    }

    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn save(&self, file: &str) {
        let serialized_data = serialize(self).unwrap();
        let mut file = File::create(file).unwrap();
        file.write_all(&serialized_data).unwrap();
    }

    pub fn load_or_new(file: &str) -> Self {
        match File::open(file) {
            Ok(mut file) => {
                let mut serialized_data = Vec::new();
                file.read_to_end(&mut serialized_data).unwrap();
                let data: Self = deserialize(&serialized_data).unwrap();
                data
            }
            Err(_) => {
                Self {
                    items: vec![],
                }
            }
        }
    }
}