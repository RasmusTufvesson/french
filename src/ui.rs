#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{self, egui::{self, Key, KeyboardShortcut, Modifiers}};
use levenshtein::levenshtein;
use crate::{explain::{explain, Part}, practice::{Practice, PracticeGroup, PracticeGroupCollection, Question, QuestionTemplate}, search::{Adjective, Category, Concreteness, Countability, Gender, Item, Language, Noun, NounCategory, Pronoun, ProperOrCommon, Query, Search, VerbForms}, sentence, utils};

#[derive(PartialEq)]
enum PracticeState {
    Wrong(String, String, String, usize, Item),
    Question(Question),
    AskContinue,
}

#[derive(PartialEq)]
enum Tab {
    Words,
    Sentences,
    Details(u32),
    Practice(PracticeState),
    Explain,
    PracticeSelect,
    PracticeView(usize),
    Example(Vec<(String, Item)>),
}

#[derive(PartialEq)]
enum PopupWindow {
    None,
    AddWord(String, String, Category, String, Option<u32>),
    AddSentence(String, String, String, Option<u32>),
    DeleteWord(u32),
    DeleteSentence(u32),
    NewGroup(String, Option<usize>),
    DeleteGroup(usize),
}

struct SearchCategories {
    noun: bool,
    verb: bool,
    adjective: bool,
    adverb: bool,
    article: bool,
    conjunction: bool,
    interjection: bool,
    preposition: bool,
    pronoun: bool,
    number: bool,
    other: bool,
}

impl SearchCategories {
    fn to_u16(&self) -> u16 {
        let mut int = 0;
        if self.noun {
            int += 0b1;
        }
        if self.verb {
            int += 0b10;
        }
        if self.adjective {
            int += 0b100;
        }
        if self.adverb {
            int += 0b1000;
        }
        if self.article {
            int += 0b10000;
        }
        if self.conjunction {
            int += 0b100000;
        }
        if self.interjection {
            int += 0b1000000;
        }
        if self.preposition {
            int += 0b10000000;
        }
        if self.pronoun {
            int += 0b100000000;
        }
        if self.number {
            int += 0b1000000000;
        }
        if self.other {
            int += 0b10000000000;
        }
        if int == 0 {
            int = u16::MAX;
        }
        int
    }

    fn new() -> Self {
        Self { noun: false, verb: false, adjective: false, adverb: false, article: false, conjunction: false, interjection: false, preposition: false, pronoun: false, number: false, other: false }
    }
}

pub struct App {
    search_words: Search,
    search_sentences: Search,
    query_string: String,
    tab: Tab,
    results_search: Vec<(String, Item)>,
    num_answers: usize,
    language: Language,
    popup: PopupWindow,
    search_words_file: String,
    search_sentences_file: String,
    categories: SearchCategories,
    min_num_answers: usize,
    result_explain: Vec<Part>,
    practice: Practice,
    practice_groups: PracticeGroupCollection,
    practice_groups_file: String,
    debug: bool,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, search_words: Search, search_sentences: Search, practice_groups: PracticeGroupCollection, search_words_file: String, search_sentences_file: String, practice_file: String) -> Self {
        let app = Self {
            search_words,
            search_sentences,
            query_string: "".to_string(),
            tab: Tab::Words,
            results_search: vec![],
            num_answers: 0,
            language: Language::French,
            popup: PopupWindow::None,
            search_words_file,
            search_sentences_file,
            categories: SearchCategories::new(),
            min_num_answers: 0,
            result_explain: vec![],
            practice: Practice::new(),
            practice_groups,
            practice_groups_file: practice_file,
            debug: false,
        };
        app
    }

    fn gen_results(&mut self) {
        self.num_answers = self.min_num_answers;
        match self.tab {
            Tab::Words => {
                let query = self.gen_query();
                if query.string.len() != 0 {
                    self.results_search = self.search_words.search(&query, self.num_answers);
                } else {
                    self.results_search = self.search_words.get_all(&query, self.num_answers);
                }
            }
            Tab::Sentences => {
                let query = self.gen_query();
                if query.string.len() != 0 {
                    self.results_search = self.search_sentences.search(&query, self.num_answers);
                } else {
                    self.results_search = self.search_sentences.get_all(&query, self.num_answers);
                }
            }
            Tab::Explain => {
                self.result_explain = explain(&self.query_string, &self.search_words);
            }
            _ => {}
        }
    }

    fn gen_query(&self) -> Query {
        Query::new(&self.query_string, &self.language, if self.tab != Tab::Sentences { self.categories.to_u16() } else { !0 }, self.query_string.ends_with('\''))
    }

    fn on_enter(&mut self) {
        match &mut self.tab {
            Tab::Practice(ref mut state) => {
                if let PracticeState::Question(question) = state {
                    if self.query_string == question.answer {
                        self.query_string.clear();
                        if self.practice.answer(true) {
                            *state = PracticeState::AskContinue;
                        } else {
                            *state = PracticeState::Question(self.practice.get_question(&self.search_words, &self.search_sentences));
                        }
                    } else {
                        let _ = self.practice.answer(false);
                        *state = PracticeState::Wrong(question.string.clone(), question.answer.clone(), self.query_string.clone(), levenshtein(&question.answer, &self.query_string), question.item.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Words").clicked() {
                    self.tab = Tab::Words;
                    self.results_search.clear();
                    self.query_string.clear();
                    self.popup = PopupWindow::None;
                    self.gen_results();
                }
                if ui.button("Sentences").clicked() {
                    self.tab = Tab::Sentences;
                    self.results_search.clear();
                    self.query_string.clear();
                    self.popup = PopupWindow::None;
                    self.gen_results();
                }
                if ui.button("Practice").clicked() {
                    self.tab = Tab::PracticeSelect;
                    self.query_string.clear();
                    self.popup = PopupWindow::None;
                }
                if ui.button("Explain").clicked() {
                    self.tab = Tab::Explain;
                    self.query_string.clear();
                    self.popup = PopupWindow::None;
                    self.result_explain.clear();
                }
                ui.with_layout(egui::Layout::right_to_left(eframe::emath::Align::Center), |ui| {
                    match self.tab {
                        Tab::Words => {
                            if ui.button("Add word").clicked() {
                                self.popup = PopupWindow::AddWord("".to_string(), "".to_string(), Category::Noun(Noun::default()), "".to_string(), None);
                            }
                            ui.separator();
                            egui::ComboBox::from_id_source("Language")
                                .selected_text(format!("{}", self.language))
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut self.language, Language::French, "French").clicked() |
                                    ui.selectable_value(&mut self.language, Language::Swedish, "Swedish").clicked() |
                                    ui.selectable_value(&mut self.language, Language::English, "English").clicked() {
                                        self.results_search.clear();
                                        self.query_string.clear();
                                        self.popup = PopupWindow::None;
                                        self.gen_results();
                                    }
                                }
                            );
                            ui.separator();
                        }
                        Tab::Sentences => {
                            if ui.button("Add sentence").clicked() {
                                self.popup = PopupWindow::AddSentence("".to_string(), "".to_string(), "".to_string(), None);
                            }
                            ui.separator();
                            egui::ComboBox::from_id_source("Language")
                                .selected_text(format!("{}", self.language))
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut self.language, Language::French, "French").clicked() |
                                    ui.selectable_value(&mut self.language, Language::Swedish, "Swedish").clicked() |
                                    ui.selectable_value(&mut self.language, Language::English, "English").clicked() {
                                        self.results_search.clear();
                                        self.query_string.clear();
                                        self.popup = PopupWindow::None;
                                        self.gen_results();
                                    }
                                }
                            );
                            ui.separator();
                        }
                        Tab::Details(_) | Tab::Practice(_) | Tab::Example(_) | Tab::PracticeView(_) => {

                        }
                        Tab::Explain => {
                            // egui::ComboBox::from_id_source("Language")
                            //     .selected_text(format!("{}", self.language))
                            //     .show_ui(ui, |ui| {
                            //         if ui.selectable_value(&mut self.language, Language::Swedish, "Swedish").clicked() |
                            //         ui.selectable_value(&mut self.language, Language::English, "English").clicked() {
                            //             self.result_explain.clear();
                            //             self.popup = PopupWindow::None;
                            //             self.gen_results();
                            //         }
                            //     }
                            // );
                            // ui.separator();
                        }
                        Tab::PracticeSelect => {
                            if ui.button("New group").clicked() {
                                self.popup = PopupWindow::NewGroup("".to_string(), None);
                            }
                        }
                    }
                });
            });
        });

        match self.tab {
            Tab::Words => {
                egui::SidePanel::left("side_panel").resizable(false).show(ctx, |ui| {
                    ui.heading("Categories");
        
                    if ui.checkbox(&mut self.categories.noun, "Nouns").changed() |
                    ui.checkbox(&mut self.categories.verb, "Verbs").changed() |
                    ui.checkbox(&mut self.categories.adjective, "Adjectives").changed() |
                    ui.checkbox(&mut self.categories.adverb, "Adverbs").changed() |
                    ui.checkbox(&mut self.categories.article, "Articles").changed() |
                    ui.checkbox(&mut self.categories.conjunction, "Conjunctions").changed() |
                    ui.checkbox(&mut self.categories.interjection, "Interjections").changed() |
                    ui.checkbox(&mut self.categories.preposition, "Prepositions").changed() |
                    ui.checkbox(&mut self.categories.pronoun, "Pronouns").changed() |
                    ui.checkbox(&mut self.categories.number, "Numbers").changed() |
                    ui.checkbox(&mut self.categories.other, "Other").changed() {
                        self.gen_results();
                    }
                });
            }
            _ => {}
        }

        if ctx.input_mut(|state| state.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::D))) {
            self.debug = !self.debug;
        }
        if self.debug {
            egui::SidePanel::right("debug_panel").resizable(false).show(ctx, |ui| {
                ui.heading("Debug");

                ui.label("Frame time:");
                ui.label(format!("{:.1} ms", ctx.input(|state| state.stable_dt) * 1000.0));    

                if ui.button("Fix uids").clicked() {
                    let indices = self.practice_groups.get_group_indices(&self.search_words, &self.search_sentences);
                    self.search_words.recreate_uids();
                    self.search_sentences.recreate_uids();
                    self.practice_groups.update_group_indices(&self.search_words, &self.search_sentences, indices);
                    self.search_words.save(&self.search_words_file);
                    self.search_sentences.save(&self.search_sentences_file);
                    self.practice_groups.save(&self.practice_groups_file);
                }
            });
        }

        let mut change_tab: Option<Tab> = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let width = ui.available_width();
            match &self.tab {
                Tab::Details(uid) => {
                    let item = self.search_words.get_item(*uid).unwrap();
                    egui::Grid::new("verb_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .min_col_width((width - 40.) / 2.)
                        .show(ui, |ui| {
                            let mut translation = false;
                            if let Some(string) = &item.swedish {
                                ui.label("Swedish");
                                ui.label(string);
                                ui.end_row();
                                translation = true;
                            }
                            if let Some(string) = &item.english {
                                ui.label("English");
                                ui.label(string);
                                ui.end_row();
                                translation = true;
                            }
                            if translation {
                                ui.end_row();
                            }
                            match &item.category {
                                Category::Verb(name, forms) => {
                                    ui.label("Type");
                                    match forms {
                                        VerbForms::Regular(..) => {
                                            ui.label("Regular verb");
                                        }
                                        VerbForms::Irregular(..) => {
                                            ui.label("Irregular verb");
                                        }
                                    }
                                    ui.end_row();
                                    ui.label("Name");
                                    ui.label(name);
                                    ui.end_row();
                                    let (VerbForms::Regular(je, tu, il, nous, vous, ils, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils) | VerbForms::Irregular(je, tu, il, nous, vous, ils, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils)) = &forms;
                                    ui.label("Je");
                                    ui.label(je);
                                    ui.end_row();
                                    ui.label("Tu");
                                    ui.label(tu);
                                    ui.end_row();
                                    ui.label("Il/elle/on");
                                    ui.label(il);
                                    ui.end_row();
                                    ui.label("Nous");
                                    ui.label(nous);
                                    ui.end_row();
                                    ui.label("Vous");
                                    ui.label(vous);
                                    ui.end_row();
                                    ui.label("Ils/elles");
                                    ui.label(ils);
                                    ui.end_row();

                                    ui.label("Passe composé");
                                    ui.label(pc);
                                    ui.end_row();

                                    ui.label("Je (imparfait)");
                                    ui.label(imp_je);
                                    ui.end_row();
                                    ui.label("Tu (imparfait)");
                                    ui.label(imp_tu);
                                    ui.end_row();
                                    ui.label("Il/elle/on (imparfait)");
                                    ui.label(imp_il);
                                    ui.end_row();
                                    ui.label("Nous (imparfait)");
                                    ui.label(imp_nous);
                                    ui.end_row();
                                    ui.label("Vous (imparfait)");
                                    ui.label(imp_vous);
                                    ui.end_row();
                                    ui.label("Ils/elles (imparfait)");
                                    ui.label(imp_ils);
                                    ui.end_row();
                                }
                                Category::Adjective(adjective) => {
                                    ui.label("Type");
                                    ui.label(format!("{} adjective", adjective.to_string()));
                                    ui.end_row();
                                    match adjective {
                                        Adjective::Demonstrative(male, male_vowel, female, plural) => {
                                            ui.label("Male");
                                            ui.label(male);
                                            ui.end_row();
                                            ui.label("Male and vowel");
                                            ui.label(male_vowel);
                                            ui.end_row();
                                            ui.label("Female");
                                            ui.label(female);
                                            ui.end_row();
                                            ui.label("Plural");
                                            ui.label(plural);
                                            ui.end_row();
                                        }
                                        Adjective::Descriptive(male, female, plural_male, plural_female) |
                                        Adjective::Indefinite(male, female, plural_male, plural_female) |
                                        Adjective::ExclamativeInterrogative(male, female, plural_male, plural_female) |
                                        Adjective::Past(male, female, plural_male, plural_female) |
                                        Adjective::Present(male, female, plural_male, plural_female) |
                                        Adjective::Relative(male, female, plural_male, plural_female) => {
                                            ui.label("Singular male");
                                            ui.label(male);
                                            ui.end_row();
                                            ui.label("Singular female");
                                            ui.label(female);
                                            ui.end_row();
                                            ui.label("Plural male");
                                            ui.label(plural_male);
                                            ui.end_row();
                                            ui.label("Plural female");
                                            ui.label(plural_female);
                                            ui.end_row();
                                        }
                                        Adjective::Negative(male, female) => {
                                            ui.label("Male");
                                            ui.label(male);
                                            ui.end_row();
                                            ui.label("Female");
                                            ui.label(female);
                                            ui.end_row();
                                        }
                                        Adjective::Possessive(male, female, plural) => {
                                            ui.label("Male");
                                            ui.label(male);
                                            ui.end_row();
                                            ui.label("Female");
                                            ui.label(female);
                                            ui.end_row();
                                            ui.label("Plural");
                                            ui.label(plural);
                                            ui.end_row();
                                        }
                                    }
                                }
                                Category::Adverb(adverb) => {
                                    ui.label("Type");
                                    ui.label("Adverb");
                                    ui.end_row();
                                    ui.label("French");
                                    ui.label(adverb);
                                    ui.end_row();
                                }
                                Category::Article(male, female, plural, elision) => {
                                    ui.label("Type");
                                    ui.label("Article");
                                    ui.end_row();
                                    ui.label("Male");
                                    ui.label(male);
                                    ui.end_row();
                                    ui.label("Female");
                                    ui.label(female);
                                    ui.end_row();
                                    ui.label("Plural");
                                    ui.label(plural);
                                    ui.end_row();
                                    if let Some(elision) = elision {
                                        ui.label("Elision").on_hover_text_at_pointer("Next word begins with vowel or h");
                                        ui.label(elision);
                                        ui.end_row();
                                    }
                                }
                                Category::Conjunction(conjunction) => {
                                    ui.label("Type");
                                    ui.label("Conjunction");
                                    ui.end_row();
                                    ui.label("French");
                                    ui.label(conjunction);
                                    ui.end_row();
                                }
                                Category::Interjection(interjection) => {
                                    ui.label("Type");
                                    ui.label("Interjection");
                                    ui.end_row();
                                    ui.label("French");
                                    ui.label(interjection);
                                    ui.end_row();
                                }
                                Category::Noun(noun) => {
                                    ui.label("Singular");
                                    ui.label(&noun.singular);
                                    ui.end_row();
                                    ui.label("Plural");
                                    ui.label(&noun.plural);
                                    ui.end_row();
                                    ui.label("Gender");
                                    ui.label(noun.gender.to_string());
                                    ui.end_row();
                                    ui.label("Countability");
                                    ui.label(noun.countable.to_string());
                                    ui.end_row();
                                    ui.label("Concreteness");
                                    ui.label(noun.concrete.to_string());
                                    ui.end_row();
                                    ui.label("Properness");
                                    ui.label(noun.proper.to_string());
                                    ui.end_row();
                                    ui.label("Category");
                                    ui.label(noun.category.to_string());
                                    ui.end_row();
                                }
                                Category::Number(cardinal, cardinal_female, ordinal, ordinal_female, multiplicative, approximate, fraction, fraction_other) => {
                                    ui.label("Type");
                                    ui.label("Number");
                                    ui.end_row();
                                    if let Some(female) = cardinal_female {
                                        ui.label("Cardinal male");
                                        ui.label(cardinal);
                                        ui.end_row();
                                        ui.label("Cardinal female");
                                        ui.label(female);
                                        ui.end_row();
                                    } else {
                                        ui.label("Cardinal");
                                        ui.label(cardinal);
                                        ui.end_row();
                                    }
                                    if let Some(female) = ordinal_female {
                                        ui.label("Ordinal male");
                                        ui.label(ordinal);
                                        ui.end_row();
                                        ui.label("Ordinal female");
                                        ui.label(female);
                                        ui.end_row();
                                    } else {
                                        ui.label("Ordinal");
                                        ui.label(ordinal);
                                        ui.end_row();
                                    }
                                    if let Some(multiplicative) = multiplicative {
                                        ui.label("Multiplicative");
                                        ui.label(multiplicative);
                                        ui.end_row();
                                    }
                                    if let Some(approximate) = approximate {
                                        ui.label("Approximate");
                                        ui.label(approximate);
                                        ui.end_row();
                                    }
                                    if let Some(fraction) = fraction {
                                        ui.end_row();
                                        ui.label("Fraction");
                                        ui.label(&format!("1/{}", item.swedish.clone().unwrap()));
                                        ui.end_row();
                                        ui.label("Fraction");
                                        ui.label(fraction);
                                        ui.end_row();
                                        if let Some(fraction_other) = fraction_other {
                                            ui.label("Fraction");
                                            ui.label(fraction_other);
                                            ui.end_row();
                                        }
                                    }
                                }
                                Category::Other(other) => {
                                    ui.label("Type");
                                    ui.label("Other");
                                    ui.end_row();
                                    ui.label("French");
                                    ui.label(other);
                                    ui.end_row();
                                }
                                Category::Preposition(preposition) => {
                                    ui.label("Type");
                                    ui.label("Preposition");
                                    ui.end_row();
                                    ui.label("French");
                                    ui.label(preposition);
                                    ui.end_row();
                                }
                                Category::Pronoun(pronoun) => {
                                    ui.label("Type");
                                    ui.label(&format!("{} pronoun", pronoun.to_string()));
                                    ui.end_row();
                                    match pronoun {
                                        Pronoun::Adverbial(string) |
                                        Pronoun::ImpersonalSubject(string) |
                                        Pronoun::IndefiniteDemonstrative(string) |
                                        Pronoun::IndefiniteRelative(string) |
                                        Pronoun::Interrogative(string) => {
                                            ui.label("French");
                                            ui.label(string);
                                            ui.end_row();
                                        }
                                        Pronoun::Negative(string) => {
                                            ui.label("French");
                                            ui.label(format!("ne ... {}", string));
                                            ui.end_row();
                                        }
                                        Pronoun::Demonstrative(s_m, s_f, p_m, p_f) |
                                        Pronoun::Possessive(s_m, s_f, p_m, p_f) => {
                                            ui.label("Singular male");
                                            ui.label(s_m);
                                            ui.end_row();
                                            ui.label("Singular female");
                                            ui.label(s_f);
                                            ui.end_row();
                                            ui.label("Plural male");
                                            ui.label(p_m);
                                            ui.end_row();
                                            ui.label("Plural female");
                                            ui.label(p_f);
                                            ui.end_row();
                                        }
                                        Pronoun::Indefinite(male, female) => {
                                            ui.label("Male");
                                            ui.label(male);
                                            ui.end_row();
                                            if let Some(female) = female {
                                                ui.label("Female");
                                                ui.label(female);
                                                ui.end_row();
                                            }
                                        }
                                        Pronoun::Personal(subject, reflexive, stressed, others) => {
                                            ui.label("Subject");
                                            ui.label(subject);
                                            ui.end_row();
                                            if let Some((direct_object, indirect_object)) = others {
                                                ui.label("Direct object");
                                                ui.label(direct_object);
                                                ui.end_row();
                                                ui.label("Indirect object");
                                                ui.label(indirect_object);
                                                ui.end_row();
                                            }
                                            ui.label("Reflexive");
                                            ui.label(reflexive);
                                            ui.end_row();
                                            ui.label("Stressed");
                                            ui.label(stressed);
                                            ui.end_row();
                                        }
                                        Pronoun::Relative(string, others) => {
                                            match others {
                                                None => {
                                                    ui.label("French");
                                                    ui.label(string);
                                                    ui.end_row();
                                                }
                                                Some((s_f, p_m, p_f)) => {
                                                    ui.label("Singular male");
                                                    ui.label(string);
                                                    ui.end_row();
                                                    ui.label("Singular female");
                                                    ui.label(s_f);
                                                    ui.end_row();
                                                    ui.label("Plural male");
                                                    ui.label(p_m);
                                                    ui.end_row();
                                                    ui.label("Plural female");
                                                    ui.label(p_f);
                                                    ui.end_row();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if self.debug {
                                ui.label("Uid");
                                ui.label(format!("{}", item.uid));
                            }
                        });
                }
                Tab::Words |
                Tab::Sentences |
                Tab::PracticeSelect => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        match self.tab {
                            Tab::Words => {
                                let num_results = ((ui.available_height() - 24.0) / 17.0).round() as usize;
                                if num_results != self.min_num_answers {
                                    self.min_num_answers = num_results;
                                    if self.min_num_answers > self.num_answers {
                                        self.gen_results();
                                    }
                                }
                                for (i, (string, item)) in self.results_search.iter().enumerate() {
                                    if i >= self.min_num_answers {
                                        break;
                                    }
                                    let response = ui.label(string);
                                    response.clone().on_hover_ui_at_pointer(|ui| {
                                        ui.label(format!("{}", item.tooltip()));
                                        if self.debug {
                                            ui.label(format!("Uid: {}", item.uid));
                                        }
                                    });
                                    response.context_menu(|ui| {
                                        if ui.button("Details").clicked() {
                                            ui.close_menu();
                                            self.tab = Tab::Details(item.uid);
                                            self.popup = PopupWindow::None;
                                        }
                                        if let Category::Verb(..) = item.category {
                                            if ui.button("Example").clicked() {
                                                ui.close_menu();
                                                self.tab = Tab::Example(sentence::generate(&self.search_words, None, Some(item.clone())));
                                                self.popup = PopupWindow::None;
                                            }
                                        }
                                        ui.menu_button("Add to practice group", |ui| {
                                            for group in &mut self.practice_groups.groups {
                                                if ui.button(&group.name).clicked() {
                                                    ui.close_menu();
                                                    group.questions.push(QuestionTemplate::Word(item.uid));
                                                    self.practice_groups.save(&self.practice_groups_file);
                                                    break;
                                                }
                                            }
                                        });
                                        if ui.button("Edit").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::AddWord(match item.swedish.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, match item.english.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, item.category.clone(), "".to_string(), Some(item.uid));
                                        }
                                        if ui.button("Delete").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::DeleteWord(item.uid);
                                        }
                                    });
                                }
                            }
                            Tab::Sentences => {
                                let num_results = ((ui.available_height() - 24.0) / 17.0).round() as usize;
                                if num_results != self.min_num_answers {
                                    self.min_num_answers = num_results;
                                    if self.min_num_answers > self.num_answers {
                                        self.gen_results();
                                    }
                                }
                                let mut update_results = false;
                                for (i, (string, item)) in self.results_search.iter().enumerate() {
                                    if i >= self.min_num_answers {
                                        break;
                                    }
                                    let response = ui.label(string);
                                    response.clone().on_hover_ui_at_pointer(|ui| {
                                        ui.label(format!("{}", item.tooltip()));
                                        if self.debug {
                                            ui.label(format!("Uid: {}", item.uid));
                                        }
                                    });
                                    response.context_menu(|ui| {
                                        if ui.button("Explain").clicked() {
                                            ui.close_menu();
                                            self.tab = Tab::Explain;
                                            self.query_string = string.clone();
                                            update_results = true;
                                        }
                                        ui.menu_button("Add to practice group", |ui| {
                                            for group in &mut self.practice_groups.groups {
                                                if ui.button(&group.name).clicked() {
                                                    ui.close_menu();
                                                    group.questions.push(QuestionTemplate::Sentence(item.uid));
                                                    self.practice_groups.save(&self.practice_groups_file);
                                                    break;
                                                }
                                            }
                                        });
                                        if ui.button("Edit").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::AddSentence(match &item.category {
                                                Category::Other(string) => string.clone(),
                                                _ => "".to_string(),
                                            }, match item.swedish.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, match item.english.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, Some(item.uid));
                                        }
                                        if ui.button("Delete").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::DeleteSentence(item.uid);
                                        }
                                    });
                                }
                                if update_results {
                                    self.gen_results();
                                }
                            }
                            Tab::PracticeSelect => {
                                for (i, group) in self.practice_groups.groups.iter().enumerate() {
                                    let response = ui.button(&group.name);
                                    response.clone().context_menu(|ui| {
                                        if ui.button("View").clicked() {
                                            ui.close_menu();
                                            self.tab = Tab::PracticeView(i);
                                        }
                                        if ui.button("Edit").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::NewGroup(group.name.clone(), Some(i));
                                        }
                                        if ui.button("Delete").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::DeleteGroup(i);
                                        }
                                    });
                                    if response.clicked() {
                                        self.practice.init(&self.practice_groups.groups[i]);
                                        self.tab = Tab::Practice(PracticeState::Question(self.practice.get_question(&self.search_words, &self.search_sentences)));
                                        self.query_string.clear();
                                        self.popup = PopupWindow::None;
                                    }
                                }
                            }
                            _ => {}
                        }
                    });
                }
                Tab::Practice(state) => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        match state {
                            PracticeState::Question(question) => {
                                ui.heading(&question.string);
                            }
                            PracticeState::Wrong(question, correct, answer, difference, _) => {
                                ui.heading(question);
                                ui.label(format!("The correct answer was '{}', not '{}'.", correct, answer));
                                if difference <= &2 {
                                    ui.label("You were close.");
                                } else if difference <= &3 {
                                    ui.label("That's almost close.");
                                } else {
                                    ui.label("You need to practice this more.");
                                }
                            }
                            PracticeState::AskContinue => {
                                ui.heading("You know all of the words, continue anyway?");
                            }
                        }
                    });
                }
                Tab::Example(words) => {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        for (word, item) in words {
                            let response = ui.label(word);
                            response.clone().on_hover_ui_at_pointer(|ui| {
                                ui.label(format!("{}", item.tooltip()));
                            });
                            response.context_menu(|ui| {
                                if ui.button("Details").clicked() {
                                    ui.close_menu();
                                    change_tab = Some(Tab::Details(item.uid));
                                    self.popup = PopupWindow::None;
                                }
                            });
                        }
                    });
                }
                Tab::PracticeView(index) => {
                    let mut to_details = None;
                    egui::Grid::new("practice_view_grid")
                        .num_columns(3)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .min_col_width((width - 80.) / 3.)
                        .show(ui, |ui| {
                            let group = &mut self.practice_groups.groups[*index];
                            ui.label(&group.name);
                            ui.end_row();
                            
                            let mut to_remove = vec![];
                            for (i, question) in group.questions.iter().enumerate() {
                                let (item, is_word) = match question {
                                    QuestionTemplate::Word(uid) => {
                                        (self.search_words.get_item(*uid), true)
                                    }
                                    QuestionTemplate::Sentence(uid) => {
                                        (self.search_sentences.get_item(*uid), false)
                                    }
                                };
                                if let Some(item) = item {
                                    let unknown = "Unknown".to_string();
                                    let french = item.language_string(&Language::French).unwrap_or(&unknown);
                                    let response = ui.label(french).on_hover_ui_at_pointer(|ui| { ui.label(item.tooltip()); });
                                    if is_word {
                                        response.context_menu(|ui| {
                                            if ui.button("Details").clicked() {
                                                ui.close_menu();
                                                to_details = Some(item.uid);
                                            }
                                            if ui.button("Remove").clicked() {
                                                ui.close_menu();
                                                to_remove.push(i);
                                            }
                                        });
                                    } else {
                                        response.context_menu(|ui| {
                                            if ui.button("Details").clicked() {
                                                ui.close_menu();
                                                to_details = Some(item.uid);
                                            }
                                        });
                                    }
                                    ui.label(item.language_string(&Language::Swedish).unwrap_or(&unknown));
                                    ui.label(item.language_string(&Language::English).unwrap_or(&unknown));
                                } else {
                                    ui.label("Deleted item");
                                }
                                ui.end_row();
                            }
                            for i in to_remove.iter().rev() {
                                group.questions.remove(*i);
                            }
                        });
                    if let Some(uid) = to_details {
                        self.tab = Tab::Details(uid);
                    }
                }
                Tab::Explain => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.label(&self.query_string);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            for part in &mut self.result_explain {
                                if part.matched.len() == 1 {
                                    let response = if part.sure {
                                        ui.label(&part.matched[part.chosen].0)
                                    } else {
                                        ui.label(&format!("{}?", part.matched[part.chosen].0))
                                    };
                                    response.clone().on_hover_ui_at_pointer(|ui| {
                                        ui.label(format!("({}) {}", &part.string, part.matched[part.chosen].1.tooltip()));
                                    });
                                    response.context_menu(|ui| {
                                        if ui.button("Details").clicked() {
                                            ui.close_menu();
                                            self.tab = Tab::Details(part.matched[part.chosen].1.uid);
                                            self.popup = PopupWindow::None;
                                        }
                                        if ui.button("Edit").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::AddWord(match part.matched[part.chosen].1.swedish.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, match part.matched[part.chosen].1.english.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, part.matched[part.chosen].1.category.clone(), "".to_string(), Some(part.matched[part.chosen].1.uid));
                                        }
                                        if ui.button("Delete").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::DeleteWord(part.matched[part.chosen].1.uid);
                                        }
                                    });
                                } else {
                                    let mut confirm = None;
                                    for (i, matched) in part.matched.iter().enumerate() {
                                        let response = if part.sure {
                                            ui.selectable_value(&mut part.chosen, i, &matched.0)
                                        } else {
                                            ui.selectable_value(&mut part.chosen, i, &format!("{}?", matched.0))
                                        };
                                        response.clone().on_hover_ui_at_pointer(|ui| {
                                            ui.label(format!("({}) {}", &part.string, matched.1.tooltip()));
                                        });
                                        response.context_menu(|ui| {
                                            if ui.button("Confirm").clicked() {
                                                ui.close_menu();
                                                confirm = Some(i);
                                            }
                                            if ui.button("Details").clicked() {
                                                ui.close_menu();
                                                self.tab = Tab::Details(part.matched[part.chosen].1.uid);
                                                self.popup = PopupWindow::None;
                                            }
                                            if ui.button("Edit").clicked() {
                                                ui.close_menu();
                                                self.popup = PopupWindow::AddWord(match part.matched[part.chosen].1.swedish.clone() {
                                                    None => "".to_string(),
                                                    Some(val) => val,
                                                }, match part.matched[part.chosen].1.english.clone() {
                                                    None => "".to_string(),
                                                    Some(val) => val,
                                                }, part.matched[part.chosen].1.category.clone(), "".to_string(), Some(part.matched[part.chosen].1.uid));
                                            }
                                            if ui.button("Delete").clicked() {
                                                ui.close_menu();
                                                self.popup = PopupWindow::DeleteWord(part.matched[part.chosen].1.uid);
                                            }
                                        });
                                    }
                                    if let Some(index) = confirm {
                                        let item = part.matched[index].clone();
                                        part.matched.clear();
                                        part.matched.push(item);
                                        part.chosen = 0;
                                    }
                                }
                            }
                        });
                        for language in [Language::English, Language::Swedish] {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                for part in &self.result_explain {
                                    let text = if let Some(text) = part.matched[part.chosen].1.language_string(&language) {
                                        text
                                    } else {
                                        &part.matched[part.chosen].0
                                    };
                                    let response = if part.sure {
                                        ui.label(text)
                                    } else {
                                        ui.label(&format!("{}?", text))
                                    };
                                    response.clone().on_hover_ui_at_pointer(|ui| {
                                        ui.label(format!("({}) {}", &part.matched[part.chosen].0, part.matched[part.chosen].1.tooltip()));
                                    });
                                    response.context_menu(|ui| {
                                        if ui.button("Details").clicked() {
                                            ui.close_menu();
                                            self.tab = Tab::Details(part.matched[part.chosen].1.uid);
                                            self.popup = PopupWindow::None;
                                        }
                                        if ui.button("Edit").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::AddWord(match part.matched[part.chosen].1.swedish.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, match part.matched[part.chosen].1.english.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, part.matched[part.chosen].1.category.clone(), "".to_string(), Some(part.matched[part.chosen].1.uid));
                                        }
                                        if ui.button("Delete").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::DeleteWord(part.matched[part.chosen].1.uid);
                                        }
                                    });
                                }
                            });
                        }
                    });
                }
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                let mut change_tab: Option<Tab> = None;
                match &self.tab {
                    Tab::Practice(PracticeState::Wrong(_, _, _, _, item)) => {
                        let response = ui.add_sized([width, 0.], egui::Button::new("Next question"));
                        if self.popup == PopupWindow::None {
                            response.request_focus();
                        }
                        if response.clicked() {
                            change_tab = Some(Tab::Practice(PracticeState::Question(self.practice.get_question(&self.search_words, &self.search_sentences))));
                            self.query_string.clear();
                        }
                        ui.add_space(ui.spacing().item_spacing.y);
                        if ui.add_sized([width, 0.], egui::Button::new("Details")).clicked() {
                            change_tab = Some(Tab::Details(item.uid));
                            self.popup = PopupWindow::None;
                        }
                    }
                    Tab::Practice(PracticeState::Question(_)) => {
                        let response = ui.add_sized([width, 0.], egui::TextEdit::singleline(&mut self.query_string));
                        if response.changed() {
                            self.gen_results();
                        }
                        if response.lost_focus()  && response.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.on_enter();
                        }
                        if self.popup == PopupWindow::None {
                            response.request_focus();
                        }
                        ui.add_space(ui.spacing().item_spacing.y);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                            let mut clicked = false;
                            if ui.button("é").clicked() {
                                self.query_string += "é";
                                clicked = true;
                            }
                            if ui.button("è").clicked() {
                                self.query_string += "è";
                                clicked = true;
                            }
                            if ui.button("ê").clicked() {
                                self.query_string += "ê";
                                clicked = true;
                            }
                            if ui.button("ç").clicked() {
                                self.query_string += "ç";
                                clicked = true;
                            }
                            if clicked {
                                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                                    let ccursor = egui::text::CCursor::new(self.query_string.chars().count());
                                    state.set_ccursor_range(Some(egui::text::CCursorRange::one(ccursor)));
                                    state.store(ui.ctx(), response.id);
                                }
                            }
                        });
                    }
                    Tab::Practice(PracticeState::AskContinue) => {
                        let response = ui.add_sized([width, 0.], egui::Button::new("Continue"));
                        if self.popup == PopupWindow::None {
                            response.request_focus();
                        }
                        if response.clicked() {
                            self.practice.continue_practice();
                            change_tab = Some(Tab::Practice(PracticeState::Question(self.practice.get_question(&self.search_words, &self.search_sentences))));
                            self.query_string.clear();
                        }
                    }
                    Tab::Details(uid) => {
                        let item = self.search_words.get_item(*uid).unwrap();
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                            if ui.button("Edit").clicked() {
                                self.popup = PopupWindow::AddWord(match item.swedish.clone() {
                                    None => "".to_string(),
                                    Some(val) => val,
                                }, match item.english.clone() {
                                    None => "".to_string(),
                                    Some(val) => val,
                                }, item.category.clone(), "".to_string(), Some(item.uid));
                            }
                            if ui.button("Delete").clicked() {
                                self.popup = PopupWindow::DeleteWord(item.uid);
                            }
                        });
                    }
                    Tab::PracticeSelect | Tab::PracticeView(_) | Tab::Example(_) => {}
                    _ => {
                        let response = ui.add_sized([width, 0.], egui::TextEdit::singleline(&mut self.query_string));
                        if response.changed() {
                            self.gen_results();
                        }
                        if self.popup == PopupWindow::None {
                            response.request_focus();
                        }
                    }
                }
                if let Some(tab) = change_tab {
                    self.tab = tab;
                }
            });
        });

        let mut close = false;
        let mut reload = false;
        match &mut self.popup {
            PopupWindow::None => {}
            PopupWindow::AddWord(swedish, english, ref mut category, any_verb, edit) => {
                egui::Window::new("Add word").collapsible(false).show(ctx, |ui| {
                    egui::ComboBox::from_label("Category")
                        .selected_text(format!("{}", category))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(category, Category::Noun(Noun::default()), "Noun");
                            ui.selectable_value(category, Category::Verb("".to_string(), VerbForms::Regular("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string())), "Verb");
                            ui.selectable_value(category, Category::Adjective(Adjective::Descriptive("".to_string(), "".to_string(), "".to_string(), "".to_string())), "Adjective");
                            ui.selectable_value(category, Category::Adverb("".to_string()), "Adverb");
                            ui.selectable_value(category, Category::Article("".to_string(), "".to_string(), "".to_string(), Some("".to_string())), "Article");
                            ui.selectable_value(category, Category::Conjunction("".to_string()), "Conjunction");
                            ui.selectable_value(category, Category::Interjection("".to_string()), "Interjection");
                            ui.selectable_value(category, Category::Preposition("".to_string()), "Preposition");
                            ui.selectable_value(category, Category::Pronoun(Pronoun::Personal("".to_string(), "".to_string(), "".to_string(), Some(("".to_string(), "".to_string())))), "Pronoun");
                            ui.selectable_value(category, Category::Number("".to_string(), None, "".to_string(), None, Some("".to_string()), Some("".to_string()), Some("".to_string()), None), "Number");
                            ui.selectable_value(category, Category::Other("".to_string()), "Other");
                        }
                    );
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(swedish));
                        ui.label("Swedish");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(english));
                        ui.label("English");
                    });
                    match category {
                        Category::Noun(noun) => {
                            ui.horizontal(|ui| {
                                if ui.add(egui::TextEdit::singleline(&mut noun.singular)).changed() {
                                    noun.plural = noun.singular.clone() + "s";
                                    noun.gender = if noun.singular.ends_with('e') { Gender::Female } else { Gender::Male };
                                }
                                ui.label("French Singular");
                            });
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut noun.plural));
                                ui.label("French Plural");
                            });
                            egui::ComboBox::from_label("Gender")
                                .selected_text(noun.gender.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut noun.gender, Gender::Male, "Male");
                                    ui.selectable_value(&mut noun.gender, Gender::Female, "Female");
                                }
                            );
                            egui::ComboBox::from_label("Countability")
                                .selected_text(noun.countable.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut noun.countable, Countability::Countable, "Countable");
                                    ui.selectable_value(&mut noun.countable, Countability::Uncountable, "Uncountable");
                                }
                            );
                            egui::ComboBox::from_label("Concreteness")
                                .selected_text(noun.concrete.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut noun.concrete, Concreteness::Concrete, "Concrete");
                                    ui.selectable_value(&mut noun.concrete, Concreteness::Abstract, "Abstract");
                                }
                            );
                            egui::ComboBox::from_label("Properness")
                                .selected_text(noun.proper.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut noun.proper, ProperOrCommon::Common, "Common");
                                    ui.selectable_value(&mut noun.proper, ProperOrCommon::Proper, "Proper");
                                }
                            );
                            egui::ComboBox::from_label("Noun Category")
                                .selected_text(noun.category.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut noun.category, NounCategory::Object, "Object");
                                    ui.selectable_value(&mut noun.category, NounCategory::Place, "Place");
                                    ui.selectable_value(&mut noun.category, NounCategory::Being, "Being");
                                    ui.selectable_value(&mut noun.category, NounCategory::Concept, "Concept");
                                }
                            );
                        }
                        Category::Verb(base, form) => {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(base));
                                ui.label("Name");
                            });
                            egui::ComboBox::from_label("Form")
                                .selected_text(format!("{}", form))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(form, VerbForms::Regular("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()), "Regular");
                                    ui.selectable_value(form, VerbForms::Irregular("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()), "Irregular");
                                }
                            );
                            match form {
                                VerbForms::Irregular(je, tu, il, nous, vous, ils, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(je));
                                        ui.label("Je");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(tu));
                                        ui.label("Tu");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(il));
                                        ui.label("Il/elle/on");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(nous));
                                        ui.label("Nous");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(vous));
                                        ui.label("Vous");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(ils));
                                        ui.label("Ils/elles");
                                    });

                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(pc));
                                        ui.label("Passe composé");
                                    });

                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(imp_je));
                                        ui.label("Je (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(imp_tu));
                                        ui.label("Tu (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(imp_il));
                                        ui.label("Il/elle/on (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(imp_nous));
                                        ui.label("Nous (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(imp_vous));
                                        ui.label("Vous (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(imp_ils));
                                        ui.label("Ils/elles (imparfait)");
                                    });
                                }
                                VerbForms::Regular(je, tu, il, nous, vous, ils, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils) => {
                                    let mut changed = "";
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(any_verb).changed() {
                                            changed = &any_verb;
                                        }
                                        ui.label("Any");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(je).changed() {
                                            changed = &je;
                                        }
                                        ui.label("Je");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(tu).changed() {
                                            changed = &tu;
                                        }
                                        ui.label("Tu");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(il).changed() {
                                            changed = &il;
                                        }
                                        ui.label("Il/elle/on");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(nous).changed() {
                                            changed = &nous;
                                        }
                                        ui.label("Nous");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(vous).changed() {
                                            changed = &vous;
                                        }
                                        ui.label("Vous");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(ils).changed() {
                                            changed = &ils;
                                        }
                                        ui.label("Ils/elles");
                                    });

                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(pc).changed() {
                                            changed = &pc;
                                        }
                                        ui.label("Passe composé");
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(imp_je).changed() {
                                            changed = &imp_je;
                                        }
                                        ui.label("Je (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(imp_tu).changed() {
                                            changed = &imp_tu;
                                        }
                                        ui.label("Tu (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(imp_il).changed() {
                                            changed = &imp_il;
                                        }
                                        ui.label("Il/elle/on (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(imp_nous).changed() {
                                            changed = &imp_nous;
                                        }
                                        ui.label("Nous (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(imp_vous).changed() {
                                            changed = &imp_vous;
                                        }
                                        ui.label("Vous (imparfait)");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(imp_ils).changed() {
                                            changed = &imp_ils;
                                        }
                                        ui.label("Ils/elles (imparfait)");
                                    });
                                    if changed != "" {
                                        (*je, *tu, *il, *nous, *vous, *ils, *pc, *imp_je, *imp_tu, *imp_il, *imp_nous, *imp_vous, *imp_ils) = VerbForms::gen_from_regular(changed);
                                    }
                                }
                            }
                        }
                        Category::Adjective(adjective) => {
                            egui::ComboBox::from_label("Type of adjective")
                                .selected_text(format!("{}", adjective))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(adjective, Adjective::Descriptive("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Descriptive");
                                    ui.selectable_value(adjective, Adjective::Indefinite("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Indefinite");
                                    ui.selectable_value(adjective, Adjective::ExclamativeInterrogative("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Exclamative and interrogative");
                                    ui.selectable_value(adjective, Adjective::Past("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Past participle");
                                    ui.selectable_value(adjective, Adjective::Present("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Present participle");
                                    ui.selectable_value(adjective, Adjective::Relative("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Relative");
                                    ui.selectable_value(adjective, Adjective::Negative("".to_string(), "".to_string()), "Negative");
                                    ui.selectable_value(adjective, Adjective::Possessive("".to_string(), "".to_string(), "".to_string()), "Possessive");
                                    ui.selectable_value(adjective, Adjective::Demonstrative("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Demonstrative");
                                }
                            );
                            match adjective {
                                Adjective::Descriptive(male, female, plural_male, plural_female) |
                                Adjective::Indefinite(male, female, plural_male, plural_female) |
                                Adjective::ExclamativeInterrogative(male, female, plural_male, plural_female) |
                                Adjective::Past(male, female, plural_male, plural_female) |
                                Adjective::Present(male, female, plural_male, plural_female) |
                                Adjective::Relative(male, female, plural_male, plural_female) => {
                                    ui.horizontal(|ui| {
                                        if ui.add(egui::TextEdit::singleline(male)).changed() {
                                            *plural_male = utils::get_adjective_plural(male);
                                        }
                                        ui.label("Singular male");
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.add(egui::TextEdit::singleline(female)).changed() {
                                            *plural_female = utils::get_adjective_plural(female);
                                        }
                                        ui.label("Singular female");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(plural_male));
                                        ui.label("Plural male");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(plural_female));
                                        ui.label("Plural female");
                                    });
                                }
                                Adjective::Demonstrative(male, male_vowel, female, plural) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(male));
                                        ui.label("Male");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(male_vowel));
                                        ui.label("Male and vowel");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(female));
                                        ui.label("Female");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(plural));
                                        ui.label("Plural");
                                    });
                                }
                                Adjective::Negative(male, female) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(male));
                                        ui.label("Male");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(female));
                                        ui.label("Female");
                                    });
                                }
                                Adjective::Possessive(male, female, plural) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(male));
                                        ui.label("Male");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(female));
                                        ui.label("Female");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(plural));
                                        ui.label("Plural");
                                    });
                                }
                            }
                        }
                        Category::Other(string) | Category::Adverb(string) | Category::Conjunction(string) | Category::Interjection(string) | Category::Preposition(string) => {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(string));
                                ui.label("French");
                            });
                        }
                        Category::Article(male, female, plural, vowel) => {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(male));
                                ui.label("Male");
                            });
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(female));
                                ui.label("Female");
                            });
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(plural));
                                ui.label("Plural");
                            });
                            let mut elision = if let Some(_) = vowel {
                                true
                            } else { false };
                            if ui.checkbox(&mut elision, "Elision").changed() {
                                if elision {
                                    *vowel = Some("".to_string());
                                } else {
                                    *vowel = None;
                                }
                            }
                            if let Some(string) = vowel {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(string));
                                    ui.label("Elision");
                                });
                            }
                        }
                        Category::Pronoun(pronoun) => {
                            egui::ComboBox::from_label("Type of pronoun")
                                .selected_text(format!("{}", pronoun))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(pronoun, Pronoun::Personal("".to_string(), "".to_string(), "".to_string(), Some(("".to_string(), "".to_string()))), "Personal");
                                    ui.selectable_value(pronoun, Pronoun::Adverbial("".to_string()), "Adverbial");
                                    ui.selectable_value(pronoun, Pronoun::ImpersonalSubject("".to_string()), "Impersonal subject");
                                    ui.selectable_value(pronoun, Pronoun::IndefiniteDemonstrative("".to_string()), "Indefinite demonstrative");
                                    ui.selectable_value(pronoun, Pronoun::IndefiniteRelative("".to_string()), "Indefinite relative");
                                    ui.selectable_value(pronoun, Pronoun::Interrogative("".to_string()), "Interrogative");
                                    ui.selectable_value(pronoun, Pronoun::Negative("".to_string()), "Negative");
                                    ui.selectable_value(pronoun, Pronoun::Demonstrative("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Demonstrative");
                                    ui.selectable_value(pronoun, Pronoun::Possessive("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Possesive");
                                    ui.selectable_value(pronoun, Pronoun::Indefinite("".to_string(), Some("".to_string())), "Indefinite");
                                    ui.selectable_value(pronoun, Pronoun::Relative("".to_string(), None), "Relative");
                                }
                            );
                            match pronoun {
                                Pronoun::Adverbial(string) | Pronoun::ImpersonalSubject(string) | Pronoun::IndefiniteDemonstrative(string) | Pronoun::IndefiniteRelative(string) | Pronoun::Interrogative(string) | Pronoun::Negative(string) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(string));
                                        ui.label("French");
                                    });
                                }
                                Pronoun::Demonstrative(s_m, s_f, p_m, p_f) | Pronoun::Possessive(s_m, s_f, p_m, p_f) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(s_m));
                                        ui.label("Singular male");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(s_f));
                                        ui.label("Singular female");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(p_m));
                                        ui.label("Plural male");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(p_f));
                                        ui.label("Plural female");
                                    });
                                }
                                Pronoun::Indefinite(male, female) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(male));
                                        ui.label("Male");
                                    });
                                    if let Some(female) = female {
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(female));
                                            ui.label("Female");
                                        });
                                    }
                                }
                                Pronoun::Personal(subject, reflexive, stressed, others) => {
                                    let mut do_and_io = if let Some(_) = others {
                                        true
                                    } else { false };
                                    if ui.checkbox(&mut do_and_io, "Direct object and indirect object").changed() {
                                        if do_and_io {
                                            *others = Some(("".to_string(), "".to_string()));
                                        } else {
                                            *others = None;
                                        }
                                    }
                                    if let Some((direct_object, indirect_object)) = others {
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(subject));
                                            ui.label("Subject");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(direct_object));
                                            ui.label("Direct object");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(indirect_object));
                                            ui.label("Indirect object");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(reflexive));
                                            ui.label("Reflexive");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(stressed));
                                            ui.label("Stressed");
                                        });
                                    } else {
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(subject));
                                            ui.label("Subject");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(reflexive));
                                            ui.label("Reflexive");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(stressed));
                                            ui.label("Stressed");
                                        });
                                    }
                                }
                                Pronoun::Relative(string, others) => {
                                    let mut bendable = if let Some(_) = others {
                                        true
                                    } else { false };
                                    if ui.checkbox(&mut bendable, "Bendable").changed() {
                                        if bendable {
                                            *others = Some(("".to_string(), "".to_string(), "".to_string()));
                                        } else {
                                            *others = None;
                                        }
                                    }
                                    if let Some((s_f, p_m, p_f)) = others {
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(string));
                                            ui.label("Singular male");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(s_f));
                                            ui.label("Singular female");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(p_m));
                                            ui.label("Plural male");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(p_f));
                                            ui.label("Plural female");
                                        });
                                    } else {
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(string));
                                            ui.label("French");
                                        });
                                    }
                                }
                            }
                        }
                        Category::Number(cardinal, cardinal_female, ordinal, ordinal_female, multiplicative, approximate, fraction, fraction_other) => {
                            let mut female = if let Some(_) = cardinal_female {
                                true
                            } else { false };
                            if ui.checkbox(&mut female, "Female variant").changed() {
                                if female {
                                    *cardinal_female = Some("".to_string());
                                    *ordinal_female = Some("".to_string());
                                } else {
                                    *cardinal_female = None;
                                    *ordinal_female = None;
                                }
                            }
                            
                            let response = ui.horizontal(|ui| {
                                let response = ui.add(egui::TextEdit::singleline(cardinal));
                                ui.label("Cardinal");
                                response
                            });

                            if response.inner.changed() {
                                let (ordinal_guess, approximate_guess) = utils::number_forms(cardinal);
                                if let Some(string) = fraction {
                                    *string = ordinal_guess.clone();
                                }
                                *ordinal = ordinal_guess;
                                if let Some(string) = approximate {
                                    *string = approximate_guess;
                                }
                            }

                            if let Some(string) = cardinal_female {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(string));
                                    ui.label("Cardinal female");
                                });
                            }

                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(ordinal));
                                ui.label("Ordinal");
                            });
                            if let Some(string) = ordinal_female {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(string));
                                    ui.label("Ordinal female");
                                });
                            }

                            let mut use_multiplicative = if let Some(_) = multiplicative {
                                true
                            } else { false };
                            if ui.checkbox(&mut use_multiplicative, "Multiplicative").changed() {
                                if use_multiplicative {
                                    *multiplicative = Some("".to_string());
                                } else {
                                    *multiplicative = None;
                                }
                            }
                            if let Some(string) = multiplicative {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(string));
                                    ui.label("Multiplicative");
                                });
                            }

                            let mut use_approximate = if let Some(_) = approximate {
                                true
                            } else { false };
                            if ui.checkbox(&mut use_approximate, "Approximate").changed() {
                                if use_approximate {
                                    *approximate = Some("".to_string());
                                } else {
                                    *approximate = None;
                                }
                            }
                            if let Some(string) = approximate {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(string));
                                    ui.label("Approximate");
                                });
                            }

                            let mut use_fraction = if let Some(_) = fraction {
                                true
                            } else { false };
                            if ui.checkbox(&mut use_fraction, "Fraction").changed() {
                                if use_fraction {
                                    *fraction = Some("".to_string());
                                } else {
                                    *fraction = None;
                                    *fraction_other = None;
                                }
                            }
                            if let Some(string) = fraction {
                                let mut two = if let Some(_) = fraction_other {
                                    true
                                } else { false };
                                if ui.checkbox(&mut two, "Two fractions").changed() {
                                    if two {
                                        *fraction_other = Some("".to_string());
                                    } else {
                                        *fraction_other = None;
                                    }
                                }
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(string));
                                    ui.label("Fraction");
                                });
                            }
                            if let Some(string) = fraction_other {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(string));
                                    ui.label("Fraction");
                                });
                            }
                        }
                    }
                    ui.horizontal(|ui| {
                        match edit {
                            None => {
                                if ui.button("Add").clicked() {
                                    close = true;
                                    let swedish_val = if swedish.len() > 0 { Some(swedish.clone()) } else { None };
                                    let english_val = if english.len() > 0 { Some(english.clone()) } else { None };
                                    let uid = self.search_words.next_uid();
                                    self.search_words.add_item(Item::new(swedish_val, english_val, category.clone(), uid));
                                    self.search_words.save(&self.search_words_file);
                                    reload = true;
                                }
                            }
                            Some(uid) => {
                                if ui.button("Apply").clicked() {
                                    close = true;
                                    let swedish_val = if swedish.len() > 0 { Some(swedish.clone()) } else { None };
                                    let english_val = if english.len() > 0 { Some(english.clone()) } else { None };
                                    let item = Item::new(swedish_val, english_val, category.clone(), *uid);
                                    self.search_words.edit_item(*uid, item).unwrap();
                                    self.search_words.save(&self.search_words_file);
                                    reload = true;
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            }
            PopupWindow::AddSentence(french, swedish, english, edit) => {
                egui::Window::new("Add sentence").collapsible(false).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.add(egui::TextEdit::singleline(french)).changed() {
                            *french = french.to_lowercase();
                        }
                        ui.label("French");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(swedish));
                        ui.label("Swedish");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(english));
                        ui.label("English");
                    });
                    ui.horizontal(|ui| {
                        match edit {
                            None => {
                                if ui.button("Add").clicked() {
                                    close = true;
                                    let swedish_val = if swedish.len() > 0 { Some(swedish.clone()) } else { None };
                                    let english_val = if english.len() > 0 { Some(english.clone()) } else { None };
                                    let uid = self.search_sentences.next_uid();
                                    self.search_sentences.add_item(Item::new(swedish_val, english_val, Category::Other(french.clone()), uid));
                                    self.search_sentences.save(&self.search_sentences_file);
                                    reload = true;
                                }
                            }
                            Some(uid) => {
                                if ui.button("Apply").clicked() {
                                    close = true;
                                    let swedish_val = if swedish.len() > 0 { Some(swedish.clone()) } else { None };
                                    let english_val = if english.len() > 0 { Some(english.clone()) } else { None };
                                    self.search_words.edit_item(*uid, Item::new(swedish_val, english_val, Category::Other(french.clone()), *uid)).unwrap();
                                    self.search_sentences.save(&self.search_sentences_file);
                                    reload = true;
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            }
            PopupWindow::DeleteWord(uid) => {
                egui::Window::new("Delete word").collapsible(false).show(ctx, |ui| {
                    ui.label("Are you sure?");
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            close = true;
                            self.search_words.remove_item(*uid).unwrap();
                            self.search_words.save(&self.search_words_file);
                            reload = true;
                            if let Tab::Details(_) = self.tab {
                                change_tab = Some(Tab::Words);
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            }
            PopupWindow::DeleteSentence(uid) => {
                egui::Window::new("Delete sentence").collapsible(false).show(ctx, |ui| {
                    ui.label("Are you sure?");
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            close = true;
                            self.search_sentences.remove_item(*uid).unwrap();
                            self.search_sentences.save(&self.search_sentences_file);
                            reload = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            }
            PopupWindow::NewGroup(name, index) => {
                egui::Window::new("New group").collapsible(false).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(name));
                        ui.label("Name");
                    });
                    ui.horizontal(|ui| {
                        match index {
                            None => {
                                if ui.button("Add").clicked() {
                                    close = true;
                                    self.practice_groups.add_group(PracticeGroup::new(name.clone()));
                                    self.practice_groups.save(&self.practice_groups_file);
                                }
                            }
                            Some(index) => {
                                if ui.button("Apply").clicked() {
                                    close = true;
                                    let group = self.practice_groups.groups.remove(*index);
                                    self.practice_groups.add_group(PracticeGroup::new_with_questions(name.clone(), group.questions));
                                    self.practice_groups.save(&self.practice_groups_file);
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            }
            PopupWindow::DeleteGroup(index) => {
                egui::Window::new("Delete group").collapsible(false).show(ctx, |ui| {
                    ui.label("Are you sure?");
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            close = true;
                            self.practice_groups.remove_group(*index);
                            self.practice_groups.save(&self.practice_groups_file);
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            }
        }
        if close {
            self.popup = PopupWindow::None;
        }
        if reload {
            self.gen_results();
        }
        if let Some(tab) = change_tab {
            self.tab = tab;
            self.results_search.clear();
            self.query_string.clear();
            self.gen_results();
        }
    }
}

pub fn run(search_words: Search, search_sentences: Search, practice_groups: PracticeGroupCollection, search_words_file: String, search_sentences_file: String, practice_file: String) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "French",
        native_options,
        Box::new(|cc| Box::new(App::new(cc, search_words, search_sentences, practice_groups, search_words_file, search_sentences_file, practice_file))),
    )
}