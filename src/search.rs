use std::{fmt::Display, fs::File, io::{Write, Read}};
use levenshtein::levenshtein;
use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};
use rand::{seq::SliceRandom, thread_rng, distributions::{Distribution, Standard}, Rng};

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

enum RegularVerbType {
    Er,
    Re,
    Ir,
}

impl VerbForms {
    pub fn gen_from_regular(string: &str) -> (String, String, String, String, String, String) {
        let (base, regular_verb_type) = if string.ends_with("issons") || string.ends_with("issent") {
            (&string[0..string.len()-6], RegularVerbType::Ir)
        } else if string.ends_with("issez") {
            (&string[0..string.len()-5], RegularVerbType::Ir)
        } else if string.ends_with("ons") || string.ends_with("ent") {
            (&string[0..string.len()-3], RegularVerbType::Er)
        } else if string.ends_with("es") || string.ends_with("ez") || string.ends_with("er") {
            (&string[0..string.len()-2], RegularVerbType::Er)
        } else if string.ends_with("is") || string.ends_with("it") || string.ends_with("ir") {
            (&string[0..string.len()-2], RegularVerbType::Ir)
        } else if string.ends_with("re") {
            (&string[0..string.len()-2], RegularVerbType::Re)
        } else if string.ends_with("e") {
            (&string[0..string.len()-1], RegularVerbType::Er)
        } else if string.ends_with("s") {
            (&string[0..string.len()-1], RegularVerbType::Ir)
        } else {
            (string, RegularVerbType::Re)
        };
        match regular_verb_type {
            RegularVerbType::Er => {
                (base.to_string()+"e", base.to_string()+"es", base.to_string()+"e", base.to_string()+"ons", base.to_string()+"ez", base.to_string()+"ent")
            }
            RegularVerbType::Ir => {
                (base.to_string()+"is", base.to_string()+"is", base.to_string()+"it", base.to_string()+"issons", base.to_string()+"issez", base.to_string()+"issent")
            }
            RegularVerbType::Re => {
                (base.to_string()+"s", base.to_string()+"s", base.to_string(), base.to_string()+"ons", base.to_string()+"ez", base.to_string()+"ent")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pronoun {
    Personal(String, String, String, Option<(String, String)>),
    Adverbial(String),
    Demonstrative(String, String, String, String),
    ImpersonalSubject(String),
    IndefiniteDemonstrative(String),
    Indefinite(String, Option<String>),
    Interrogative(String),
    Negative(String),
    Possessive(String, String, String, String),
    Relative(String, Option<(String, String, String)>),
    IndefiniteRelative(String),
}

impl Display for Pronoun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Adverbial(_) => "Adverbial",
            Self::Demonstrative(..) => "Demonstrative",
            Self::ImpersonalSubject(_) => "Impersonal subject",
            Self::Indefinite(..) => "Indefinite",
            Self::IndefiniteDemonstrative(_) => "Indefinite demonstrative",
            Self::IndefiniteRelative(_) => "Indefinite relative",
            Self::Interrogative(_) => "Interrogative",
            Self::Negative(_) => "Negative",
            Self::Personal(..) => "Personal",
            Self::Possessive(..) => "Possessive",
            Self::Relative(..) => "Relative",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Category {
    Noun(String, Gender, String),
    Verb(String, VerbForms),
    Adjective(String, String, String),
    Article(String, String, String, Option<String>),
    Conjunction(String),
    Pronoun(Pronoun),
    Preposition(String),
    Adverb(String),
    Interjection(String),
    Other(String),
}

fn get_plural(string: &str) -> String {
    if string.ends_with("al") || string.ends_with("au") {
        string[0..string.len()-2].to_string() + "aux"
    } else if string.ends_with("ail") {
        string[0..string.len()-3].to_string() + "aux"
    } else if string.ends_with("eu") || string.ends_with("ou") {
        string.to_string() + "x"
    } else if !string.ends_with("s") && !string.ends_with("x") && !string.ends_with("z") {
        string.to_string() + "s"
    } else {
        string.to_string()
    }
}

impl Category {
    pub fn to_u16(&self) -> u16 {
        match self {
            Self::Noun(..) => 0b1,
            Self::Verb(..) => 0b10,
            Self::Adjective(..) => 0b100,
            Self::Adverb(_) => 0b1000,
            Self::Article(..) => 0b10000,
            Self::Conjunction(_) => 0b100000,
            Self::Interjection(_) => 0b1000000,
            Self::Preposition(_) => 0b10000000,
            Self::Pronoun(..) => 0b100000000,
            Self::Other(_) => 0b11111111,
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
            Self::Other(string) => format!("{} ({}, {})", string, swedish, english),
            Self::Noun(string, gender, plural) => {
                match gender {
                    Gender::Male => format!("{}/{} ({}, {}), masculine noun", string, plural, swedish, english),
                    Gender::Female => format!("{}/{} ({}, {}), feminine noun", string, plural, swedish, english),
                }
            }
            Self::Verb(name, forms) => {
                match forms {
                    VerbForms::Irregular(..) => format!("{} ({}, {}), irregular verb", name, swedish, english),
                    VerbForms::Regular(..) => format!("{} ({}, {}), regular verb", name, swedish, english),
                }
            }
            Self::Adjective(female, male, plural) => format!("{}/{}/{} ({}, {}), adjective", male, female, plural, swedish, english),
            Self::Adverb(string) => format!("{} ({}, {}), adverb", string, swedish, english),
            Self::Article(male, female, plural, vowel) => {
                match vowel {
                    Some(v) => format!("{}/{}/{}/{} ({}, {}), article", male, female, plural, v, swedish, english),
                    None => format!("{}/{}/{} ({}, {}), article", male, female, plural, swedish, english),
                }
            }
            Self::Conjunction(string) => format!("{} ({}, {}), conjunction", string, swedish, english),
            Self::Interjection(string) => format!("{} ({}, {}), interjection", string, swedish, english),
            Self::Preposition(string) => format!("{} ({}, {}), preposition", string, swedish, english),
            Self::Pronoun(pronoun_type) => {
                match pronoun_type {
                    Pronoun::Personal(subject, reflexive, stressed, others) => {
                        match others {
                            Some((direct_object, indirect_object)) => format!("{}/{}/{}/{}/{} ({}, {}), personal pronoun", subject, direct_object, indirect_object, reflexive, stressed, swedish, english),
                            None => format!("{}/{}/{} ({}, {}), personal pronoun", subject, reflexive, stressed, swedish, english),
                        }
                    }
                    Pronoun::Adverbial(string) => format!("{} ({}, {}), adverbial pronoun", string, swedish, english),
                    Pronoun::Demonstrative(s_m, s_f, p_m, p_f) => format!("{}/{}/{}/{} ({}, {}), demonstrative pronoun", s_m, s_f, p_m, p_f, swedish, english),
                    Pronoun::ImpersonalSubject(string) => format!("{} ({}, {}), impersonal subject", string, swedish, english),
                    Pronoun::Indefinite(male, female) => {
                        match female {
                            Some(female) => format!("{}/{} ({}, {}), indefinite pronoun", male, female, swedish, english),
                            None => format!("{} ({}, {}), indefinite pronoun", male, swedish, english),
                        }
                    }
                    Pronoun::IndefiniteDemonstrative(string) => format!("{} ({}, {}), indefinite demonstrative pronoun", string, swedish, english),
                    Pronoun::IndefiniteRelative(string) => format!("{} ({}, {}), indefinite relative pronoun", string, swedish, english),
                    Pronoun::Interrogative(string) => format!("{} ({}, {}), interrogative pronoun", string, swedish, english),
                    Pronoun::Negative(string) => format!("ne ... {} ({}, {}), negative pronoun", string, swedish, english),
                    Pronoun::Possessive(s_m, s_f, p_m, p_f) => format!("{}/{}/{}/{} ({}, {}), possesive pronoun", s_m, s_f, p_m, p_f, swedish, english),
                    Pronoun::Relative(string, others) => {
                        match others {
                            None => format!("{} ({}, {}), relative pronoun", string, swedish, english),
                            Some((s_f, p_m, p_f)) => format!("{}/{}/{}/{} ({}, {}), relative pronoun", string, s_f, p_m, p_f, swedish, english),
                        }
                    }
                }
            }
        }
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Other(_) => "Other",
            Self::Noun(..) => "Noun",
            Self::Verb(..) => "Verb",
            Self::Adjective(..) => "Adjective",
            Self::Adverb(_) => "Adverb",
            Self::Article(..) => "Article",
            Self::Conjunction(_) => "Conjunction",
            Self::Interjection(_) => "Interjection",
            Self::Preposition(_) => "Preposition",
            Self::Pronoun(..) => "Pronoun"
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub swedish: Option<String>,
    pub english: Option<String>,
    pub category: Category,
    category_int: u16,
}

impl Item {
    pub fn new(swedish: Option<String>, english: Option<String>, category: Category) -> Self {
        let category_int = category.to_u16();
        Self { swedish, english, category, category_int }
    }

    fn language_strings(&self, language: &Language) -> Option<Vec<String>> {
        match language {
            Language::French => {
                match &self.category {
                    Category::Other(string) => Some(vec![string.to_owned()]),
                    Category::Adjective(female, male, plural) => Some(vec![female.to_owned(), male.to_owned(), plural.to_owned()]),
                    Category::Noun(string, _, plural) => Some(vec![string.to_owned(), plural.to_owned()]),
                    Category::Verb(base, form) => {
                        match form {
                            VerbForms::Regular(je, tu, _, nous, vous, ils) => Some(vec![je.to_owned(), tu.to_owned(), nous.to_owned(), vous.to_owned(), ils.to_owned(), base.to_owned()]),
                            VerbForms::Irregular(je, tu, il, nous, vous, ils) => Some(vec![je.to_owned(), tu.to_owned(), il.to_owned(), nous.to_owned(), vous.to_owned(), ils.to_owned(), base.to_owned()]),
                        }
                    }
                    Category::Adverb(string) => Some(vec![string.to_owned()]),
                    Category::Article(male, female, plural, vowel) => {
                        match vowel {
                            Some(vowel) => Some(vec![male.to_owned(), female.to_owned(), plural.to_owned(), vowel.to_owned()]),
                            None => Some(vec![male.to_owned(), female.to_owned(), plural.to_owned()]),
                        }
                    }
                    Category::Conjunction(string) => Some(vec![string.to_owned()]),
                    Category::Interjection(string) => Some(vec![string.to_owned()]),
                    Category::Preposition(string) => Some(vec![string.to_owned()]),
                    Category::Pronoun(pronoun) => {
                        match pronoun {
                            Pronoun::Adverbial(string) => Some(vec![string.to_owned()]),
                            Pronoun::ImpersonalSubject(string) => Some(vec![string.to_owned()]),
                            Pronoun::IndefiniteDemonstrative(string) => Some(vec![string.to_owned()]),
                            Pronoun::IndefiniteRelative(string) => Some(vec![string.to_owned()]),
                            Pronoun::Interrogative(string) => Some(vec![string.to_owned()]),
                            Pronoun::Negative(string) => Some(vec![string.to_owned()]),
                            Pronoun::Demonstrative(s_m, s_f, p_m, p_f) => Some(vec![s_m.to_owned(), s_f.to_owned(), p_m.to_owned(), p_f.to_owned()]),
                            Pronoun::Indefinite(male, female) => {
                                match female {
                                    Some(female) => Some(vec![male.to_owned(), female.to_owned()]),
                                    None => Some(vec![male.to_owned()]),
                                }
                            }
                            Pronoun::Personal(subject, reflexive, stressed, others) => {
                                match others {
                                    Some((direct_object, indirect_object)) => Some(vec![subject.to_owned(), direct_object.to_owned(), indirect_object.to_owned(), reflexive.to_owned(), stressed.to_owned()]),
                                    None => Some(vec![subject.to_owned(), reflexive.to_owned(), stressed.to_owned()])
                                }
                            }
                            Pronoun::Possessive(s_m, s_f, p_m, p_f) => Some(vec![s_m.to_owned(), s_f.to_owned(), p_m.to_owned(), p_f.to_owned()]),
                            Pronoun::Relative(string, others) => {
                                match others {
                                    Some((s_f, p_m, p_f)) => Some(vec![string.to_owned(), s_f.to_owned(), p_m.to_owned(), p_f.to_owned()]),
                                    None => Some(vec![string.to_owned()]),
                                }
                            }
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

impl Language {
    pub fn to_str(&self) -> &str {
        match self {
            Self::French => "french",
            Self::Swedish => "swedish",
            Self::English => "english",
        }
    }
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

impl Distribution<Language> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Language {
        match rng.gen_range(0..=1) {
            0 => Language::French,
            _ => Language::Swedish,
        }
    }
}

pub struct Query<'a> {
    string: &'a String,
    language: &'a Language,
    search_categories_int: u16,
}

impl<'a> Query<'a> {
    pub fn new(string: &'a String, language: &'a Language, search_categories_int: u16) -> Self {
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

    pub fn get_item_index(&self, item: &Item) -> usize {
        self.items.iter().position(|x| x == item).unwrap()
    }

    pub fn remove_item(&mut self, item: usize) {
        self.items.remove(item);
    }

    pub fn random_item(&self) -> Item {
        self.items.choose(&mut thread_rng()).unwrap().clone()
    }
}