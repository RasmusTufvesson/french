use std::{fmt::Display, fs::File, io::{Write, Read}};
use levenshtein::levenshtein;
use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};
use rand::{distributions::{Distribution, Standard}, rngs::ThreadRng, seq::SliceRandom, Rng};

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
    //        je,     tu,     il,     nous,   vous,   ils,    passe composé, imp je, imp tu, imp il, imp nous, imp vous, imp ils
    Regular(  String, String, String, String, String, String, String,        String, String, String, String,   String,   String),
    Irregular(String, String, String, String, String, String, String,        String, String, String, String,   String,   String),
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub enum VerbFormsOld {
//     Regular(  String, String, String, String, String, String),
//     Irregular(String, String, String, String, String, String),
// }

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
    pub fn gen_from_regular(string: &str) -> (String, String, String, String, String, String, String, String, String, String, String, String, String) {
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
                (base.to_string()+"e", base.to_string()+"es", base.to_string()+"e", base.to_string()+"ons", base.to_string()+"ez", base.to_string()+"ent", base.to_string()+"é", base.to_string()+"ais", base.to_string()+"ais", base.to_string()+"ait", base.to_string()+"ions", base.to_string()+"iez", base.to_string()+"aient")
            }
            RegularVerbType::Ir => {
                (base.to_string()+"is", base.to_string()+"is", base.to_string()+"it", base.to_string()+"issons", base.to_string()+"issez", base.to_string()+"issent", base.to_string()+"i", base.to_string()+"issais", base.to_string()+"issais", base.to_string()+"issait", base.to_string()+"issions", base.to_string()+"issiez", base.to_string()+"issaient")
            }
            RegularVerbType::Re => {
                (base.to_string()+"s", base.to_string()+"s", base.to_string(), base.to_string()+"ons", base.to_string()+"ez", base.to_string()+"ent", base.to_string()+"u", base.to_string()+"ais", base.to_string()+"ais", base.to_string()+"ait", base.to_string()+"ions", base.to_string()+"iez", base.to_string()+"aient")
            }
        }
    }

    // fn from_old(old: VerbFormsOld) -> Self {
    //     match old {
    //         VerbFormsOld::Regular(je, tu, il, nous, vous, ils) => {
    //             let base = &je[..je.len()-1];
    //             Self::Regular(je.clone(), tu, il, nous, vous, ils, base.to_string()+"é", base.to_string()+"ais", base.to_string()+"ais", base.to_string()+"ait", base.to_string()+"ions", base.to_string()+"iez", base.to_string()+"aient")
    //         }
    //         VerbFormsOld::Irregular(je, tu, il, nous, vous, ils) => {
    //             Self::Irregular(je, tu, il, nous, vous, ils, "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string())
    //         }
    //     }
    // }
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
pub enum Adjective {
    Demonstrative(String, String),
    ExclamativeInterrogative(String, String, String, String),
    Indefinite(String, String, String, String),
    Negative(String, String),
    Possessive(String, String, String),
    Relative(String, String, String, String),
    Past(String, String, String, String),
    Present(String, String, String, String),
}

impl Display for Adjective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Demonstrative(..) => "Demonstrative",
            Self::ExclamativeInterrogative(..) => "Exclamative and interrogative",
            Self::Indefinite(..) => "Indefinite",
            Self::Negative(..) => "Negative",
            Self::Past(..) => "Past participle",
            Self::Possessive(..) => "Possessive",
            Self::Present(..) => "Present participle",
            Self::Relative(..) => "Relative",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Category {
    Noun(String, Gender, String),
    Verb(String, VerbForms),
    Adjective(Adjective),
    Article(String, String, String, Option<String>),
    Conjunction(String),
    Pronoun(Pronoun),
    Preposition(String),
    Adverb(String),
    Interjection(String),
    Number(String, Option<String>, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>),
    Other(String),
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub enum CategoryOld {
//     Noun(String, Gender, String),
//     Verb(String, VerbFormsOld),
//     Adjective(Adjective),
//     Article(String, String, String, Option<String>),
//     Conjunction(String),
//     Pronoun(Pronoun),
//     Preposition(String),
//     Adverb(String),
//     Interjection(String),
//     Number(String, Option<String>, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>),
//     Other(String),
// }

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
            Self::Number(..) => 0b1000000000,
            Self::Other(_) => 0b10000000000,
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
            Self::Adjective(adjective) => {
                match adjective {
                    Adjective::Demonstrative(singular, plural) => format!("{}/{} ({}, {}), demonstrative adjective", singular, plural, swedish, english),
                    Adjective::ExclamativeInterrogative(s_m, s_f, p_m, p_f) => format!("{}/{}/{}/{} ({}, {}), exclamative and interrogative adjective", s_m, s_f, p_m, p_f, swedish, english),
                    Adjective::Indefinite(s_m, s_f, p_m, p_f) => format!("{}/{}/{}/{} ({}, {}), indefinite adjective", s_m, s_f, p_m, p_f, swedish, english),
                    Adjective::Negative(male, female) => format!("ne ... {}/ne ... {} ({}, {}), negative adjective", male, female, swedish, english),
                    Adjective::Past(s_m, s_f, p_m, p_f) => format!("{}/{}/{}/{} ({}, {}), past participle adjective", s_m, s_f, p_m, p_f, swedish, english),
                    Adjective::Possessive(male, female, plural) => format!("{}/{}/{} ({}, {}), possessive adjective", male, female, plural, swedish, english),
                    Adjective::Present(s_m, s_f, p_m, p_f) => format!("{}/{}/{}/{} ({}, {}), present participle adjective", s_m, s_f, p_m, p_f, swedish, english),
                    Adjective::Relative(s_m, s_f, p_m, p_f) => format!("{}/{}/{}/{} ({}, {}), relative adjective", s_m, s_f, p_m, p_f, swedish, english),
                }
            }
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
                    Pronoun::Possessive(s_m, s_f, p_m, p_f) => format!("{}/{}/{}/{} ({}, {}), possessive pronoun", s_m, s_f, p_m, p_f, swedish, english),
                    Pronoun::Relative(string, others) => {
                        match others {
                            None => format!("{} ({}, {}), relative pronoun", string, swedish, english),
                            Some((s_f, p_m, p_f)) => format!("{}/{}/{}/{} ({}, {}), relative pronoun", string, s_f, p_m, p_f, swedish, english),
                        }
                    }
                }
            }
            Self::Number(cardinal, cardinal_female, ordinal, ordinal_female, multiplicative, approximate, fraction, fraction_other) => {
                let mut string = cardinal.to_owned();
                if let Some(cardinal_female) = cardinal_female {
                    string += "/";
                    string += cardinal_female;
                }
                string += "/";
                string += ordinal;
                if let Some(ordinal_female) = ordinal_female {
                    string += "/";
                    string += ordinal_female;
                }
                if let Some(multiplicative) = multiplicative {
                    string += "/";
                    string += multiplicative;
                }
                if let Some(approximate) = approximate {
                    string += "/";
                    string += approximate;
                }
                if let Some(fraction) = fraction {
                    string += "/";
                    string += fraction;
                }
                if let Some(fraction_other) = fraction_other {
                    string += "/";
                    string += fraction_other;
                }
                format!("{} ({}), number", string, swedish)
            }
        }
    }

    // fn from_old(old: CategoryOld) -> Self {
    //     match old {
    //         CategoryOld::Adjective(a) => Self::Adjective(a),
    //         CategoryOld::Article(a, b, c, d) => Self::Article(a, b, c, d),
    //         CategoryOld::Adverb(a) => Self::Adverb(a),
    //         CategoryOld::Conjunction(a) => Self::Conjunction(a),
    //         CategoryOld::Interjection(a) => Self::Interjection(a),
    //         CategoryOld::Noun(a, b, c) => Self::Noun(a, b, c),
    //         CategoryOld::Other(a) => Self::Other(a),
    //         CategoryOld::Preposition(a) => Self::Preposition(a),
    //         CategoryOld::Pronoun(a) => Self::Pronoun(a),
    //         CategoryOld::Verb(a, b) => Self::Verb(a, VerbForms::from_old(b)),
    //         CategoryOld::Number(a, b, c, d, e, f, g, h) => Self::Number(a, b, c, d, e, f, g, h),
    //     }
    // }
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
            Self::Pronoun(..) => "Pronoun",
            Self::Number(..) => "Number",
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

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub struct ItemOld {
//     pub swedish: Option<String>,
//     pub english: Option<String>,
//     pub category: CategoryOld,
//     category_int: u16,
// }

impl Item {
    pub fn new(swedish: Option<String>, english: Option<String>, category: Category) -> Self {
        let category_int = category.to_u16();
        Self { swedish, english, category, category_int }
    }

    fn language_strings(&self, language: &Language) -> Option<Vec<&String>> {
        match language {
            Language::French => {
                match &self.category {
                    Category::Other(string) => Some(vec![string]),
                    Category::Adjective(adjective) => {
                        match adjective {
                            Adjective::Demonstrative(a, b) |
                            Adjective::Negative(a, b) => Some(vec![a,b]),
                            Adjective::ExclamativeInterrogative(a, b, c, d) |
                            Adjective::Indefinite(a, b, c, d) |
                            Adjective::Past(a, b, c, d) |
                            Adjective::Present(a, b, c, d) |
                            Adjective::Relative(a, b, c, d) => Some(vec![a, b, c, d]),
                            Adjective::Possessive(a, b, c) => Some(vec![a, b, c]),
                        }
                    }
                    Category::Noun(string, _, plural) => Some(vec![string, plural]),
                    Category::Verb(base, form) => {
                        match form {
                            VerbForms::Regular(je, tu, _, nous, vous, ils, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils) => Some(vec![je, tu, nous, vous, ils, base, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils]),
                            VerbForms::Irregular(je, tu, il, nous, vous, ils, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils) => Some(vec![je, tu, il, nous, vous, ils, base, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils]),
                        }
                    }
                    Category::Adverb(string) => Some(vec![string]),
                    Category::Article(male, female, plural, vowel) => {
                        match vowel {
                            Some(vowel) => Some(vec![male, female, plural, vowel]),
                            None => Some(vec![male, female, plural]),
                        }
                    }
                    Category::Conjunction(string) => Some(vec![string]),
                    Category::Interjection(string) => Some(vec![string]),
                    Category::Preposition(string) => Some(vec![string]),
                    Category::Pronoun(pronoun) => {
                        match pronoun {
                            Pronoun::Adverbial(string) |
                            Pronoun::ImpersonalSubject(string) |
                            Pronoun::IndefiniteDemonstrative(string) |
                            Pronoun::IndefiniteRelative(string) |
                            Pronoun::Interrogative(string) |
                            Pronoun::Negative(string) => Some(vec![string]),
                            Pronoun::Demonstrative(s_m, s_f, p_m, p_f) => Some(vec![s_m, s_f, p_m, p_f]),
                            Pronoun::Indefinite(male, female) => {
                                match female {
                                    Some(female) => Some(vec![male, female]),
                                    None => Some(vec![male]),
                                }
                            }
                            Pronoun::Personal(subject, reflexive, stressed, others) => {
                                match others {
                                    Some((direct_object, indirect_object)) => Some(vec![subject, direct_object, indirect_object, reflexive, stressed]),
                                    None => Some(vec![subject, reflexive, stressed])
                                }
                            }
                            Pronoun::Possessive(s_m, s_f, p_m, p_f) => Some(vec![s_m, s_f, p_m, p_f]),
                            Pronoun::Relative(string, others) => {
                                match others {
                                    Some((s_f, p_m, p_f)) => Some(vec![string, s_f, p_m, p_f]),
                                    None => Some(vec![string]),
                                }
                            }
                        }
                    }
                    Category::Number(cardinal, cardinal_female, ordinal, ordinal_female, multiplicative, approximate, fraction, fraction_female) => {
                        let mut strings = vec![cardinal, ordinal];
                        if let Some(cardinal_female) = cardinal_female {
                            strings.push(cardinal_female);
                        }
                        if let Some(ordinal_female) = ordinal_female {
                            strings.push(ordinal_female);
                        }
                        if let Some(multiplicative) = multiplicative {
                            strings.push(multiplicative);
                        }
                        if let Some(approximate) = approximate {
                            strings.push(approximate);
                        }
                        if let Some(fraction) = fraction {
                            strings.push(fraction);
                        }
                        if let Some(fraction_female) = fraction_female {
                            strings.push(fraction_female);
                        }
                        Some(strings)
                    }
                }
            }
            Language::Swedish => {
                match &self.swedish {
                    None => None,
                    Some(string) => Some(vec![string]),
                }
            }
            Language::English => {
                match &self.english {
                    None => None,
                    Some(string) => Some(vec![string]),
                }
            }
        }
    }

    pub fn language_string(&self, language: &Language) -> Option<&String> {
        match language {
            Language::French => {
                match &self.category {
                    Category::Adjective(adjective) => {
                        match adjective {
                            Adjective::Demonstrative(a, _) |
                            Adjective::Negative(a, _) |
                            Adjective::ExclamativeInterrogative(a, ..) |
                            Adjective::Indefinite(a, ..) |
                            Adjective::Past(a, ..) |
                            Adjective::Present(a, ..) |
                            Adjective::Relative(a, ..) |
                            Adjective::Possessive(a, ..) => Some(a),
                        }
                    }
                    Category::Number(string, ..) |
                    Category::Other(string) |
                    Category::Noun(string, _, _) |
                    Category::Verb(string, _) |
                    Category::Adverb(string) |
                    Category::Conjunction(string) |
                    Category::Interjection(string) |
                    Category::Preposition(string) |
                    Category::Article(string, ..) => Some(string),
                    Category::Pronoun(pronoun) => {
                        match pronoun {
                            Pronoun::Adverbial(string) |
                            Pronoun::ImpersonalSubject(string) |
                            Pronoun::IndefiniteDemonstrative(string) |
                            Pronoun::IndefiniteRelative(string) |
                            Pronoun::Interrogative(string) |
                            Pronoun::Negative(string) |
                            Pronoun::Demonstrative(string, ..) |
                            Pronoun::Indefinite(string, _) |
                            Pronoun::Personal(string, ..) |
                            Pronoun::Possessive(string, ..) |
                            Pronoun::Relative(string, _) => Some(string),
                        }
                    }
                }
            }
            Language::Swedish => {
                match &self.swedish {
                    None => None,
                    Some(string) => Some(string),
                }
            }
            Language::English => {
                match &self.english {
                    None => None,
                    Some(string) => Some(string),
                }
            }
        }
    }

    pub fn tooltip(&self) -> String {
        self.category.display_detailed(&self.english, &self.swedish)
    }

    // fn from_old(old: ItemOld) -> Self {
    //     Self { swedish: old.swedish, english: old.english, category: Category::from_old(old.category), category_int: old.category_int }
    // }
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
    pub string: &'a String,
    pub language: &'a Language,
    pub search_categories_int: u16,
    pub match_length: bool,
}

impl<'a> Query<'a> {
    pub fn new(string: &'a String, language: &'a Language, search_categories_int: u16, match_length: bool) -> Self {
        Self { string, language, search_categories_int, match_length }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Search {
    items: Vec<Item>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct SearchOld {
//     items: Vec<ItemOld>,
// }

impl Search {
    pub fn get_all(&self, query: &Query, num_answers: usize) -> Vec<(String, Item)> {
        let mut matches = vec![];

        'outer: for item in &self.items {
            if let Some(string) = item.language_string(&query.language) {
                if item.category_int & query.search_categories_int != 0 {
                    matches.push((string, item.clone()));
                    if matches.len() == num_answers {
                        break 'outer;
                    }
                }
            }
        }

        matches.iter().map(|(s, x)| (s.to_owned().to_owned(), x.to_owned())).collect()
    }

    pub fn all_items(&self, query: &Query) -> Vec<&Item> {
        let mut matches = vec![];

        for item in &self.items {
            if item.category_int & query.search_categories_int != 0 {
                matches.push(item);
            }
        }

        matches
    }

    pub fn search(&self, query: &Query, num_answers: usize) -> Vec<(String, Item)> {
        let mut best_matches: Vec<(&String, Item)> = Vec::with_capacity(num_answers);
        let mut best_match_scores: Vec<usize> = vec![usize::MAX; num_answers];
        let length = query.string.len();

        for item in &self.items {
            if let Some(strings) = item.language_strings(&query.language) {
                if item.category_int & query.search_categories_int != 0 {
                    for string in strings {
                        let list_item = (string, item.clone());
                        if ((query.match_length && length == string.len()) || !query.match_length) && !best_matches.contains(&list_item) {
                            let distance = levenshtein(&query.string, &list_item.0);
                
                            for i in 0..num_answers {
                                if distance < best_match_scores[i] && distance < string.len() {
                                    best_match_scores.insert(i, distance);
                                    best_match_scores.truncate(num_answers);
                                    best_matches.insert(i, list_item.to_owned());
                                    best_matches.truncate(num_answers);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        best_matches.iter().map(|(s, x)| (s.to_owned().to_owned(), x.to_owned())).collect()
    }

    pub fn search_best_answers(&self, query: &Query) -> (Vec<(String, Item)>, usize) {
        let mut best_matches: Vec<(&String, Item)> = vec![];
        let mut best_match_score: usize = usize::MAX;
        let length = query.string.len();

        for item in &self.items {
            if let Some(strings) = item.language_strings(&query.language) {
                if item.category_int & query.search_categories_int != 0 {
                    for string in strings {
                        let list_item = (string, item.clone());
                        if ((query.match_length && length == string.len()) || !query.match_length) && !best_matches.contains(&list_item) {
                            let distance = levenshtein(&query.string, &list_item.0);
                            if distance < best_match_score && distance < string.len() {
                                best_match_score = distance;
                                best_matches.clear();
                                best_matches.push(list_item.to_owned());
                            } else if distance == best_match_score {
                                best_matches.push(list_item.to_owned());
                            }
                        }
                    }
                }
            }
        }

        (best_matches.iter().map(|(s, x)| (s.to_owned().to_owned(), x.to_owned())).collect(), best_match_score)
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

    pub fn get_item(&self, index: usize) -> Item {
        self.items[index].clone()
    }

    pub fn random_item(&self, query: &Query, rng: &mut ThreadRng) -> Item {
        (**self.all_items(query).choose(rng).unwrap()).clone()
    }

    // pub fn from_old(old: SearchOld) -> Self {
    //     Self { items: old.items.iter().map(|x| Item::from_old(x.to_owned())).collect() }
    // }
}

// impl SearchOld {
//     pub fn load_or_new(file: &str) -> Self {
//         match File::open(file) {
//             Ok(mut file) => {
//                 let mut serialized_data = Vec::new();
//                 file.read_to_end(&mut serialized_data).unwrap();
//                 let data: Self = deserialize(&serialized_data).unwrap();
//                 data
//             }
//             Err(_) => {
//                 Self {
//                     items: vec![],
//                 }
//             }
//         }
//     }
// }