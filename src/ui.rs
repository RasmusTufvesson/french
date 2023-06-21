#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{self, egui};
use crate::search::{Search, Item, Query, Language, Category, Gender, VerbForms};

enum Tab {
    Words,
    Sentences,
    Verbs,
}

enum PopupWindow {
    None,
    AddWord(String, String, Category, String),
    AddSentence(String, String, String),
}

struct SearchCategories {
    noun: bool,
    verb: bool,
    adjective: bool,
}

impl SearchCategories {
    fn to_u8(&self) -> u8 {
        let mut int = 0;
        if self.noun {
            int += 0b00000001;
        }
        if self.verb {
            int += 0b00000010;
        }
        if self.adjective {
            int += 0b00000100;
        }
        if int == 0 {
            int = 0b11111111;
        }
        int
    }

    fn new() -> Self {
        Self { noun: false, verb: false, adjective: false }
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
                let results = self.search_words.search(&Query::new(&self.query_string, &self.language, 0b00000010), 1);
                if results.len() != 0 {
                    let result = &results[0].1;
                    match &result.category {
                        Category::Verb(name, form) => {
                            self.results_verbs = (result.swedish.clone(), result.english.clone(), name.clone(), form.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn gen_query(&self) -> Query {
        Query::new(&self.query_string, &self.language, self.categories.to_u8())
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
                ui.with_layout(egui::Layout::right_to_left(eframe::emath::Align::Center), |ui| {
                    match self.tab {
                        Tab::Words => {
                            if ui.button("Add word").clicked() {
                                self.popup = PopupWindow::AddWord("".to_string(), "".to_string(), Category::All("".to_string()), "".to_string());
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
                                self.popup = PopupWindow::AddSentence("".to_string(), "".to_string(), "".to_string());
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
                    ui.checkbox(&mut self.categories.adjective, "Adjectives").changed() {
                        self.gen_results();
                    }
                });
            }
            _ => {}
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                match self.tab {
                    Tab::Words => {
                        let num_results = ((ui.available_height() - 26.0) / 17.0).round() as usize;
                        if num_results != self.min_num_answers {
                            self.min_num_answers = num_results;
                            if self.min_num_answers > self.num_answers {
                                self.gen_results();
                            }
                        }
                        for (i, (string, item)) in self.results_words.iter().enumerate() {
                            if i >= self.min_num_answers {
                                break;
                            }
                            ui.label(string).on_hover_ui_at_pointer(|ui| {
                                ui.label(format!("{}", item.tooltip()));
                            });
                        }
                    }
                    Tab::Sentences => {
                        let num_results = ((ui.available_height() - 26.0) / 17.0).round() as usize;
                        if num_results != self.min_num_answers {
                            self.min_num_answers = num_results;
                            if self.min_num_answers > self.num_answers {
                                self.gen_results();
                            }
                        }
                        for (i, (string, item)) in self.results_sentences.iter().enumerate() {
                            if i >= self.min_num_answers {
                                break;
                            }
                            ui.label(string).on_hover_ui_at_pointer(|ui| {
                                ui.label(format!("{}", item.tooltip()));
                            });
                        }
                    }
                    Tab::Verbs => {
                        egui::Grid::new("verb_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
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
                    }
                }
            });
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                let response = ui.add_sized([ui.available_width(), 0.], egui::TextEdit::singleline(&mut self.query_string));
                if response.changed() {
                    self.gen_results();
                }
            });
        });

        let mut close = false;
        let mut reload = false;
        match &mut self.popup {
            PopupWindow::None => {}
            PopupWindow::AddWord(swedish, english, ref mut category, any_verb) => {
                egui::Window::new("Add word").collapsible(false).show(ctx, |ui| {
                    egui::ComboBox::from_label("Category")
                        .selected_text(format!("{}", category))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(category, Category::Noun("".to_string(), Gender::Male), "Noun");
                            ui.selectable_value(category, Category::Verb("".to_string(), VerbForms::Regular("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string())), "Verb");
                            ui.selectable_value(category, Category::Adjective("".to_string(), "".to_string()), "Adjective");
                            ui.selectable_value(category, Category::All("".to_string()), "Other");
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
                        Category::Noun(string, gender) => {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(string));
                                ui.label("French");
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
                                        if let Some(forms) = VerbForms::gen_from_regular(changed) {
                                            (*je, *tu, *il, *nous, *vous, *ils) = forms;
                                        }
                                    }
                                }
                            }
                        }
                        Category::Adjective(female, male) => {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(male));
                                ui.label("Male");
                            });
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(female));
                                ui.label("Female");
                            });
                        }
                        Category::All(string) => {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(string));
                                ui.label("French");
                            });
                        }
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Add").clicked() {
                            close = true;
                            let swedish_val = if swedish.len() > 0 { Some(swedish.clone()) } else { None };
                            let english_val = if english.len() > 0 { Some(english.clone()) } else { None };
                            self.search_words.add_item(Item::new(swedish_val, english_val, category.clone()));
                            self.search_words.save(&self.search_words_file);
                            reload = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            }
            PopupWindow::AddSentence(french, swedish, english) => {
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
                        if ui.button("Add").clicked() {
                            close = true;
                            let swedish_val = if swedish.len() > 0 { Some(swedish.clone()) } else { None };
                            let english_val = if english.len() > 0 { Some(english.clone()) } else { None };
                            self.search_sentences.add_item(Item::new(swedish_val, english_val, Category::All(french.clone())));
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