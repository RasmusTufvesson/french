use std::{fmt::Display, fs::File, io::{Read, Write}};
use crate::search::{Adjective, Category, Item, Language, Pronoun, Search, VerbForms};
use bincode::{deserialize, serialize};
use rand::{thread_rng, Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

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

    fn translate_plural(french: String, swedish: String, to_language: Language, item: Item) -> Self {
        match to_language {
            Language::French => Self { string: format!("What is '{}' in french plural?", swedish), answer: french, language: to_language, item },
            Language::Swedish => Self { string: format!("What is '{}' (plural) in swedish?", french), answer: swedish, language: to_language, item },
            Language::English => unreachable!(),
        }
    }
}

fn generate_practice_question(item: Item) -> Question {
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
                _ => Question::translate_plural(plural.clone(), item.swedish.clone().unwrap(), to_language, item),
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum QuestionTemplate {
    Word(usize),
    Sentence(usize),
}

#[derive(Serialize, Deserialize)]
pub struct PracticeGroup {
    pub name: String,
    pub questions: Vec<QuestionTemplate>,
}

impl PracticeGroup {
    pub fn new(name: String) -> Self {
        Self { name, questions: vec![] }
    }

    pub fn new_with_questions(name: String, questions: Vec<QuestionTemplate>) -> Self {
        Self { name, questions }
    }
}

#[derive(Debug)]
pub struct Practice {
    templates: Vec<QuestionTemplate>,
    questions: Vec<usize>,
    question: usize,
    question_index: usize,
    to_repeat: Vec<usize>,
    answers: Vec<bool>,
    continuing: bool,
}

impl Practice {
    pub fn get_question(&mut self, words: &Search, sentences: &Search) -> Question {
        let mut rng = thread_rng();
        self.question += 1;
        if self.question == self.questions.len() {
            self.question = 0;
            self.questions.shuffle(&mut rng);
        }
        if self.to_repeat.len() != 0 && rng.gen_bool(0.3) {
            self.question -= 1;
            let to_repeat_index = rng.gen_range(0..self.to_repeat.len());
            self.question_index = self.to_repeat.swap_remove(to_repeat_index);
        } else {
            self.question_index = self.questions[self.question];
        }
        self.gen_question(words, sentences)
    }

    fn gen_question(&self, words: &Search, sentences: &Search) -> Question {
        match self.templates[self.question_index] {
            QuestionTemplate::Sentence(index) => {
                generate_practice_question(sentences.get_item(index))
            }
            QuestionTemplate::Word(index) => {
                generate_practice_question(words.get_item(index))
            }
        }
    }

    pub fn answer(&mut self, answer: bool) -> bool {
        self.answers[self.question_index] = answer;
        if !answer {
            self.to_repeat.push(self.question_index);
            false
        } else if !self.continuing && self.answers.iter().all(|x| *x) {
            true
        } else {
            false
        }
    }

    pub fn continue_practice(&mut self) {
        self.continuing = true;
    }

    pub fn new() -> Self {
        Self { templates: vec![], questions: vec![], question: 0, question_index: 0, to_repeat: vec![], answers: vec![], continuing: false }
    }

    pub fn init(&mut self, group: &PracticeGroup) {
        self.templates = group.questions.clone();
        self.questions = (0..group.questions.len()).collect();
        self.questions.shuffle(&mut thread_rng());
        self.question = 0;
        self.question_index = 0;
        self.to_repeat = vec![];
        self.answers = vec![false; group.questions.len()];
        self.continuing = false;
    }
}

#[derive(Serialize, Deserialize)]
pub struct PracticeGroupCollection {
    pub groups: Vec<PracticeGroup>,
}

impl PracticeGroupCollection {
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
                    groups: vec![],
                }
            }
        }
    }

    pub fn add_group(&mut self, group: PracticeGroup) {
        self.groups.push(group);
    }

    pub fn remove_group(&mut self, index: usize) {
        let _ = self.groups.remove(index);
    }
}