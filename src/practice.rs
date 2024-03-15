use std::fmt::Display;

use crate::search::{Adjective, Category, Item, Language, Pronoun, Search, VerbForms};
use rand::{thread_rng, Rng};

enum Form {
    Male,
    Female,
    Plural,
}

impl Display for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Male => "male",
            Self::Female => "female",
            Self::Plural => "plural",
        })
    }
}

#[derive(PartialEq)]
pub struct Question {
    pub string: String,
    pub answer: String,
    pub language: Language,
    pub item: Item,
}

impl Question {
    fn translate(french: String, swedish: String, to_language: Language, item: Item) -> Self {
        match to_language {
            Language::French => Self { string: format!("What is '{}' in french?", swedish), answer: french, language: to_language, item },
            Language::Swedish => Self { string: format!("What is '{}' in swedish?", french), answer: swedish, language: to_language, item },
            Language::English => unreachable!(),
        }
    }

    fn translate_adjective(french: String, swedish: String, to_language: Language, item: Item) -> Self {
        match to_language {
            Language::French => Self { string: format!("What is '{}' in french (masculine)?", swedish), answer: french, language: to_language, item },
            Language::Swedish => Self { string: format!("What is '{}' in swedish?", french), answer: swedish, language: to_language, item },
            Language::English => unreachable!(),
        }
    }

    fn translate_verb(french: String, swedish: String, form: &str, to_language: Language, item: Item) -> Self {
        match to_language {
            Language::French => Self { string: format!("What is the {} form of '{}' in french?", form, swedish), answer: french, language: to_language, item },
            Language::Swedish => Self { string: format!("What is '{}' ({}) in swedish?", french, form), answer: swedish, language: to_language, item },
            Language::English => unreachable!(),
        }
    }

    fn translate_form(french: String, swedish: String, form: Form, to_language: Language, item: Item) -> Self {
        match to_language {
            Language::French => Self { string: format!("What is the {} form of '{}' in french?", form, swedish), answer: french, language: to_language, item },
            Language::Swedish => Self { string: format!("What is '{}' ({}) in swedish?", french, form), answer: swedish, language: to_language, item },
            Language::English => unreachable!(),
        }
    }
    fn translate_number(french: String, num: String, to_language: Language, item: Item) -> Self {
        match to_language {
            Language::French => Self { string: format!("What is '{}' in french?", num), answer: french, language: to_language, item },
            Language::Swedish => Self { string: format!("What number is '{}'?", french), answer: num, language: to_language, item },
            Language::English => unreachable!(),
        }
    }
}

pub fn get_practice_question(search: &Search) -> Question {
    let item = search.random_item();
    let to_language = thread_rng().gen::<Language>();
    match item.category {
        Category::Other(ref s) |
        Category::Adverb(ref s) |
        Category::Conjunction(ref s) |
        Category::Interjection(ref s) |
        Category::Preposition(ref s) => {
            Question::translate(s.clone(), item.swedish.clone().unwrap(), to_language, item)
        }
        Category::Adjective(ref adjective) => {
            match adjective {
                Adjective::Indefinite(s, _, _, _) |
                Adjective::ExclamativeInterrogative(s, _, _, _) |
                Adjective::Past(s, _, _, _) |
                Adjective::Present(s, _, _, _) |
                Adjective::Relative(s, _, _, _) |
                Adjective::Demonstrative(s, _) |
                Adjective::Negative(s, _) |
                Adjective::Possessive(s, _, _) => Question::translate_adjective(s.clone(), item.swedish.clone().unwrap(), to_language, item),
            }
        }
        Category::Noun(ref s, _, ref plural) => {
            match thread_rng().gen_range(0..=1) {
                0 => Question::translate(s.clone(), item.swedish.clone().unwrap(), to_language, item),
                _ => Question::translate(plural.clone(), item.swedish.clone().unwrap(), to_language, item),
            }
            
        }
        Category::Verb(_, ref forms) => {
            let (VerbForms::Regular(je, tu, il, nous, vous, ils) | VerbForms::Irregular(je, tu, il, nous, vous, ils)) = forms.clone();
            match thread_rng().gen_range(0..=5) {
                0 => Question::translate_verb(je, item.swedish.clone().unwrap(), "je", to_language, item),
                1 => Question::translate_verb(tu, item.swedish.clone().unwrap(), "tu", to_language, item),
                2 => Question::translate_verb(il, item.swedish.clone().unwrap(), "il/elle/on", to_language, item),
                3 => Question::translate_verb(nous, item.swedish.clone().unwrap(), "nous", to_language, item),
                4 => Question::translate_verb(vous, item.swedish.clone().unwrap(), "vous", to_language, item),
                _ => Question::translate_verb(ils, item.swedish.clone().unwrap(), "ils/elles", to_language, item),
            }
        }
        Category::Article(ref m, ref f, ref p, _) => {
            match thread_rng().gen_range(0..=2) {
                0 => Question::translate_form(m.clone(), item.swedish.clone().unwrap(), Form::Male, to_language, item),
                1 => Question::translate_form(f.clone(), item.swedish.clone().unwrap(), Form::Female, to_language, item),
                _ => Question::translate_form(p.clone(), item.swedish.clone().unwrap(), Form::Plural, to_language, item),
            }
        }
        Category::Number(ref c, _, ref o, _, _, _, _, _) => {
            match thread_rng().gen_range(0..=1) {
                0 => Question::translate_number(c.clone(), item.swedish.clone().unwrap(), to_language, item),
                _ => Question::translate_number(o.clone(), item.swedish.clone().unwrap(), to_language, item),
            }
        }
        Category::Pronoun(ref p) => {
            match p {
                Pronoun::Adverbial(s) |
                Pronoun::ImpersonalSubject(s) |
                Pronoun::IndefiniteDemonstrative(s) |
                Pronoun::IndefiniteRelative(s) |
                Pronoun::Interrogative(s) |
                Pronoun::Negative(s) |
                Pronoun::Personal(s, _, _, _) |
                Pronoun::Possessive(s, _, _, _) |
                Pronoun::Demonstrative(s, _, _, _) |
                Pronoun::Relative(s, _) |
                Pronoun::Indefinite(s, _) => Question::translate_adjective(s.clone(), item.swedish.clone().unwrap(), to_language, item),
            }
        }
    }
}