#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{self, egui::{self, Layout}};
use levenshtein::levenshtein;
use crate::{explain::{explain, Part}, practice::{get_practice_question, Question}, search::{Adjective, Category, Gender, Item, Language, Pronoun, Query, Search, VerbForms}, utils};

#[derive(PartialEq)]
enum PracticeState {
    Wrong(String, String, String, usize),
    Question(Question),
}

#[derive(PartialEq)]
enum Tab {
    Words,
    Sentences,
    Verbs,
    Practice(PracticeState),
    Explain,
}

#[derive(PartialEq)]
enum PopupWindow {
    None,
    AddWord(String, String, Category, String, Option<usize>),
    AddSentence(String, String, String, Option<usize>),
    DeleteWord(usize),
    DeleteSentence(usize),
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
    results_words: Vec<(String, Item)>,
    results_sentences: Vec<(String, Item)>,
    num_answers: usize,
    language: Language,
    popup: PopupWindow,
    search_words_file: String,
    search_sentences_file: String,
    categories: SearchCategories,
    min_num_answers: usize,
    results_verbs: (Option<String>, Option<String>, String, VerbForms),
    result_explain: Vec<Part>
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, search_words: Search, search_sentences: Search, search_words_file: String, search_sentences_file: String) -> Self {
        let app = Self {
            search_words,
            search_sentences,
            query_string: "".to_string(),
            tab: Tab::Words,
            results_words: vec![],
            results_sentences: vec![],
            num_answers: 0,
            language: Language::French,
            popup: PopupWindow::None,
            search_words_file,
            search_sentences_file,
            categories: SearchCategories::new(),
            min_num_answers: 0,
            results_verbs: (None, None, "".to_string(), VerbForms::Regular("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string())),
            result_explain: vec![],
        };
        app
    }

    fn gen_results(&mut self) {
        self.num_answers = self.min_num_answers;
        match self.tab {
            Tab::Words => {
                self.results_words = self.search_words.search(&self.gen_query(), self.num_answers);
            }
            Tab::Sentences => {
                self.results_sentences = self.search_sentences.search(&self.gen_query(), self.num_answers);
            }
            Tab::Verbs => {
                let results = self.search_words.search(&Query::new(&self.query_string, &self.language, 0b10), 1);
                if results.len() != 0 {
                    let result = &results[0].1;
                    match &result.category {
                        Category::Verb(name, form) => {
                            self.results_verbs = (result.swedish.clone(), result.english.clone(), name.clone(), form.clone());
                        }
                        _ => {}
                    }
                }
            },
            Tab::Explain => {
                self.result_explain = explain(&self.query_string, &self.search_words);
            }
            _ => {}
        }
    }

    fn gen_query(&self) -> Query {
        Query::new(&self.query_string, &self.language, self.categories.to_u16())
    }

    fn on_enter(&mut self) {
        match &mut self.tab {
            Tab::Practice(ref mut state) => {
                if let PracticeState::Question(question) = state {
                    if self.query_string == question.answer {
                        *state = PracticeState::Question(get_practice_question(&self.search_words));
                        self.query_string.clear();
                    } else {
                        *state = PracticeState::Wrong(question.string.clone(), question.answer.clone(), self.query_string.clone(), levenshtein(&question.answer, &self.query_string));
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
                    self.results_words.clear();
                    self.query_string.clear();
                    self.popup = PopupWindow::None;
                    self.gen_results();
                }
                if ui.button("Sentences").clicked() {
                    self.tab = Tab::Sentences;
                    self.results_sentences.clear();
                    self.query_string.clear();
                    self.popup = PopupWindow::None;
                    self.gen_results();
                }
                if ui.button("Verbs").clicked() {
                    self.tab = Tab::Verbs;
                    self.results_verbs = (None, None, "".to_string(), VerbForms::Regular("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()));
                    self.query_string.clear();
                    self.popup = PopupWindow::None;
                    self.gen_results();
                }
                if ui.button("Practice").clicked() {
                    self.tab = Tab::Practice(PracticeState::Question(get_practice_question(&self.search_words)));
                    self.query_string.clear();
                    self.popup = PopupWindow::None;
                }
                if ui.button("Explain").clicked() {
                    self.tab = Tab::Explain;
                    self.query_string.clear();
                    self.popup = PopupWindow::None;
                    self.result_explain.clear();
                    if self.language == Language::French {
                        self.language = Language::Swedish;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(eframe::emath::Align::Center), |ui| {
                    match self.tab {
                        Tab::Words => {
                            if ui.button("Add word").clicked() {
                                self.popup = PopupWindow::AddWord("".to_string(), "".to_string(), Category::Other("".to_string()), "".to_string(), None);
                            }
                            ui.separator();
                            egui::ComboBox::from_id_source("Language")
                                .selected_text(format!("{}", self.language))
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut self.language, Language::French, "French").clicked() |
                                    ui.selectable_value(&mut self.language, Language::Swedish, "Swedish").clicked() |
                                    ui.selectable_value(&mut self.language, Language::English, "English").clicked() {
                                        self.results_sentences.clear();
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
                                        self.results_sentences.clear();
                                        self.query_string.clear();
                                        self.popup = PopupWindow::None;
                                        self.gen_results();
                                    }
                                }
                            );
                            ui.separator();
                        }
                        Tab::Verbs => {
                            egui::ComboBox::from_id_source("Language")
                                .selected_text(format!("{}", self.language))
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut self.language, Language::French, "French").clicked() |
                                    ui.selectable_value(&mut self.language, Language::Swedish, "Swedish").clicked() |
                                    ui.selectable_value(&mut self.language, Language::English, "English").clicked() {
                                        self.results_sentences.clear();
                                        self.query_string.clear();
                                        self.popup = PopupWindow::None;
                                        self.gen_results();
                                    }
                                }
                            );
                            ui.separator();
                        }
                        Tab::Practice(_) => {

                        }
                        Tab::Explain => {
                            egui::ComboBox::from_id_source("Language")
                                .selected_text(format!("{}", self.language))
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut self.language, Language::Swedish, "Swedish").clicked() |
                                    ui.selectable_value(&mut self.language, Language::English, "English").clicked() {
                                        self.result_explain.clear();
                                        self.popup = PopupWindow::None;
                                        self.gen_results();
                                    }
                                }
                            );
                            ui.separator();
                        }
                    }
                });
            });
        });

        match self.tab {
            Tab::Words => {
                egui::SidePanel::left("side_panel").show(ctx, |ui| {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.tab {
                Tab::Verbs => {
                    ui.with_layout(Layout::top_down_justified(eframe::emath::Align::Center), |ui| {
                        egui::Grid::new("verb_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .min_col_width(ui.available_width()/2.0)
                            .show(ui, |ui| {
                                let mut translation = false;
                                if let Some(string) = &self.results_verbs.0 {
                                    ui.label("Swedish");
                                    ui.label(string);
                                    ui.end_row();
                                    translation = true;
                                }
                                if let Some(string) = &self.results_verbs.1 {
                                    ui.label("English");
                                    ui.label(string);
                                    ui.end_row();
                                    translation = true;
                                }
                                if translation {
                                    ui.end_row();
                                }
                                ui.label("Name");
                                ui.label(&self.results_verbs.2);
                                ui.end_row();
                                let (VerbForms::Regular(je, tu, il, nous, vous, ils) | VerbForms::Irregular(je, tu, il, nous, vous, ils)) = &self.results_verbs.3;
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
                            });
                    });
                }
                Tab::Words |
                Tab::Sentences => {
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
                                let mut update_results = false;
                                for (i, (string, item)) in self.results_words.iter().enumerate() {
                                    if i >= self.min_num_answers {
                                        break;
                                    }
                                    let response = ui.label(string);
                                    response.clone().on_hover_ui_at_pointer(|ui| {
                                        ui.label(format!("{}", item.tooltip()));
                                    });
                                    response.context_menu(|ui| {
                                        if let Category::Verb(name, _) = &item.category {
                                            if ui.button("Open in verb view").clicked() {
                                                ui.close_menu();
                                                self.tab = Tab::Verbs;
                                                self.query_string = name.clone();
                                                update_results = true;
                                            }
                                        }
                                        if ui.button("Edit").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::AddWord(match item.swedish.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, match item.english.clone() {
                                                None => "".to_string(),
                                                Some(val) => val,
                                            }, item.category.clone(), "".to_string(), Some(self.search_words.get_item_index(item)));
                                        }
                                        if ui.button("Delete").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::DeleteWord(self.search_words.get_item_index(item));
                                        }
                                    });
                                }
                                if update_results {
                                    self.gen_results();
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
                                for (i, (string, item)) in self.results_sentences.iter().enumerate() {
                                    if i >= self.min_num_answers {
                                        break;
                                    }
                                    let response = ui.label(string);
                                    response.clone().on_hover_ui_at_pointer(|ui| {
                                        ui.label(format!("{}", item.tooltip()));
                                    });
                                    response.context_menu(|ui| {
                                        if ui.button("Explain").clicked() {
                                            ui.close_menu();
                                            self.tab = Tab::Explain;
                                            self.query_string = string.clone();
                                            if self.language == Language::French {
                                                self.language = Language::Swedish;
                                            }
                                            update_results = true;
                                        }
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
                                            }, Some(self.search_sentences.get_item_index(item)));
                                        }
                                        if ui.button("Delete").clicked() {
                                            ui.close_menu();
                                            self.popup = PopupWindow::DeleteSentence(self.search_sentences.get_item_index(item));
                                        }
                                    });
                                }
                                if update_results {
                                    self.gen_results();
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
                            PracticeState::Wrong(question, correct, answer, difference) => {
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
                        }
                    });
                }
                Tab::Explain => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.label(&self.query_string);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            for part in &self.result_explain {
                                ui.label(&part.matched).on_hover_ui_at_pointer(|ui| {
                                    ui.label(format!("({}) {}", &part.string, part.item.tooltip()));
                                });
                            }
                        });
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            if self.language == Language::Swedish {
                                for part in &self.result_explain {
                                    ui.label(&part.item.swedish.clone().unwrap()).on_hover_ui_at_pointer(|ui| {
                                        ui.label(format!("({}) {}", &part.matched, part.item.tooltip()));
                                    });
                                }
                            } else {
                                for part in &self.result_explain {
                                    ui.label(&part.item.english.clone().unwrap()).on_hover_ui_at_pointer(|ui| {
                                        ui.label(format!("({}) {}", &part.matched, part.item.tooltip()));
                                    });
                                }
                            }
                        });
                    });
                }
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                match self.tab {
                    Tab::Practice(PracticeState::Wrong(_, _, _, _)) => {
                        let response = ui.add_sized([ui.available_width(), 0.], egui::Button::new("Next question"));
                        if self.popup == PopupWindow::None {
                            response.request_focus();
                        }
                        if response.clicked() {
                            self.tab = Tab::Practice(PracticeState::Question(get_practice_question(&self.search_words)));
                            self.query_string.clear();
                        }
                    }
                    Tab::Practice(PracticeState::Question(_)) => {
                        let response = ui.add_sized([ui.available_width(), 0.], egui::TextEdit::singleline(&mut self.query_string));
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
                    _ => {
                        let response = ui.add_sized([ui.available_width(), 0.], egui::TextEdit::singleline(&mut self.query_string));
                        if response.changed() {
                            self.gen_results();
                        }
                        if self.popup == PopupWindow::None {
                            response.request_focus();
                        }
                    }
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
                            ui.selectable_value(category, Category::Noun("".to_string(), Gender::Male, "".to_string()), "Noun");
                            ui.selectable_value(category, Category::Verb("".to_string(), VerbForms::Regular("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string())), "Verb");
                            ui.selectable_value(category, Category::Adjective(Adjective::Indefinite("".to_string(), "".to_string(), "".to_string(), "".to_string())), "Adjective");
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
                        Category::Noun(string, gender, plural) => {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(string));
                                ui.label("French Singular");
                            });
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(plural));
                                ui.label("French Plural");
                            });
                            egui::ComboBox::from_label("Gender")
                                .selected_text(format!("{}", gender))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(gender, Gender::Male, "Male");
                                    ui.selectable_value(gender, Gender::Female, "Female");
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
                                    ui.selectable_value(form, VerbForms::Regular("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()), "Regular");
                                    ui.selectable_value(form, VerbForms::Irregular("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()), "Irregular");
                                }
                            );
                            match form {
                                VerbForms::Irregular(je, tu, il, nous, vous, ils) => {
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
                                }
                                VerbForms::Regular(je, tu, il, nous, vous, ils) => {
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
                                    if changed != "" {
                                        (*je, *tu, *il, *nous, *vous, *ils) = VerbForms::gen_from_regular(changed);
                                    }
                                }
                            }
                        }
                        Category::Adjective(adjective) => {
                            egui::ComboBox::from_label("Type of adjective")
                                .selected_text(format!("{}", adjective))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(adjective, Adjective::Indefinite("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Indefinite");
                                    ui.selectable_value(adjective, Adjective::ExclamativeInterrogative("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Exclamative and interrogative");
                                    ui.selectable_value(adjective, Adjective::Past("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Past participle");
                                    ui.selectable_value(adjective, Adjective::Present("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Present participle");
                                    ui.selectable_value(adjective, Adjective::Relative("".to_string(), "".to_string(), "".to_string(), "".to_string()), "Relative");
                                    ui.selectable_value(adjective, Adjective::Negative("".to_string(), "".to_string()), "Negative");
                                    ui.selectable_value(adjective, Adjective::Possessive("".to_string(), "".to_string(), "".to_string()), "Possessive");
                                    ui.selectable_value(adjective, Adjective::Demonstrative("".to_string(), "".to_string()), "Demonstrative");
                                }
                            );
                            match adjective {
                                Adjective::Indefinite(male, female, plural_male, plural_female) |
                                Adjective::ExclamativeInterrogative(male, female, plural_male, plural_female) |
                                Adjective::Past(male, female, plural_male, plural_female) |
                                Adjective::Present(male, female, plural_male, plural_female) |
                                Adjective::Relative(male, female, plural_male, plural_female) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(male));
                                        ui.label("Singular male");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(female));
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
                                Adjective::Demonstrative(singular, plural) => {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(singular));
                                        ui.label("Singular");
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
                                    self.search_words.add_item(Item::new(swedish_val, english_val, category.clone()));
                                    self.search_words.save(&self.search_words_file);
                                    reload = true;
                                }
                            }
                            Some(index) => {
                                if ui.button("Apply").clicked() {
                                    close = true;
                                    let swedish_val = if swedish.len() > 0 { Some(swedish.clone()) } else { None };
                                    let english_val = if english.len() > 0 { Some(english.clone()) } else { None };
                                    self.search_words.remove_item(*index);
                                    self.search_words.add_item(Item::new(swedish_val, english_val, category.clone()));
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
                        ui.add(egui::TextEdit::singleline(french));
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
                                    self.search_sentences.add_item(Item::new(swedish_val, english_val, Category::Other(french.clone())));
                                    self.search_sentences.save(&self.search_sentences_file);
                                    reload = true;
                                }
                            }
                            Some(index) => {
                                if ui.button("Apply").clicked() {
                                    close = true;
                                    let swedish_val = if swedish.len() > 0 { Some(swedish.clone()) } else { None };
                                    let english_val = if english.len() > 0 { Some(english.clone()) } else { None };
                                    self.search_sentences.remove_item(*index);
                                    self.search_sentences.add_item(Item::new(swedish_val, english_val, Category::Other(french.clone())));
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
            PopupWindow::DeleteWord(index) => {
                egui::Window::new("Delete word").collapsible(false).show(ctx, |ui| {
                    ui.label("Are you sure?");
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            close = true;
                            self.search_words.remove_item(*index);
                            self.search_words.save(&self.search_words_file);
                            reload = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            }
            PopupWindow::DeleteSentence(index) => {
                egui::Window::new("Delete sentence").collapsible(false).show(ctx, |ui| {
                    ui.label("Are you sure?");
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            close = true;
                            self.search_sentences.remove_item(*index);
                            self.search_sentences.save(&self.search_sentences_file);
                            reload = true;
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
    }
}

pub fn run(search_words: Search, search_sentences: Search, search_words_file: String, search_sentences_file: String) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "French",
        native_options,
        Box::new(|cc| Box::new(App::new(cc, search_words, search_sentences, search_words_file, search_sentences_file))),
    )
}