use crate::search::{Category, Language, Search, VerbForms};
use rand::{thread_rng, Rng};

#[derive(PartialEq)]
pub struct Question {
    pub string: String,
    pub answer: String,
    pub language: Language,
}

impl Question {
    fn translate(french: String, swedish: String, to_language: Language) -> Self {
        match to_language {
            Language::French => Self { string: format!("What is '{}' in french?", swedish), answer: french, language: to_language },
            Language::Swedish => Self { string: format!("What is '{}' in swedish?", french), answer: swedish, language: to_language },
            Language::English => unreachable!(),
        }
    }

    fn translate_adjective(french: String, swedish: String, to_language: Language) -> Self {
        match to_language {
            Language::French => Self { string: format!("What is '{}' in french (masculine)?", swedish), answer: french, language: to_language },
            Language::Swedish => Self { string: format!("What is '{}' in swedish?", french), answer: swedish, language: to_language },
            Language::English => unreachable!(),
        }
    }

    fn translate_verb(french: String, swedish: String, form: &str, to_language: Language) -> Self {
        match to_language {
            Language::French => Self { string: format!("What is the {} form of '{}' in french?", form, swedish), answer: french, language: to_language },
            Language::Swedish => Self { string: format!("What is '{}' ({}) in swedish?", french, form), answer: swedish, language: to_language },
            Language::English => unreachable!(),
        }
    }
}

pub fn get_practice_question(search: &Search) -> Question {
    let item = search.random_item();
    let to_language = thread_rng().gen::<Language>();
    match item.category {
        Category::All(s) => {
            Question::translate(s, item.swedish.unwrap(), to_language)
        }
        Category::Adjective(_, s, _) => {
            Question::translate_adjective(s, item.swedish.unwrap(), to_language)
        }
        Category::Noun(s, _) => {
            Question::translate(s, item.swedish.unwrap(), to_language)
        }
        Category::Verb(_, forms) => {
            let (VerbForms::Regular(je, tu, il, nous, vous, ils) | VerbForms::Irregular(je, tu, il, nous, vous, ils)) = forms;
            match thread_rng().gen_range(0..=5) {
                0 => Question::translate_verb(je, item.swedish.unwrap(), "je", to_language),
                1 => Question::translate_verb(tu, item.swedish.unwrap(), "tu", to_language),
                2 => Question::translate_verb(il, item.swedish.unwrap(), "il/elle/on", to_language),
                3 => Question::translate_verb(nous, item.swedish.unwrap(), "nous", to_language),
                4 => Question::translate_verb(vous, item.swedish.unwrap(), "vous", to_language),
                _ => Question::translate_verb(ils, item.swedish.unwrap(), "ils/elles", to_language),
            }
        }
    }
}