use levenshtein::levenshtein;

pub enum Gender {
    Female,
    Male,
}

pub enum AdjectiveForms {
    Regular(String),
    Irregular(String, String, String, String, String, String, String)
}

pub enum Category {
    Noun(Gender),
    Verb,
    Adjective(AdjectiveForms),
}

impl Category {
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Noun(_) => 0b00000001,
            Self::Verb => 0b00000010,
            Self::Adjective(_) => 0b00000100,
        }
    }
}

pub struct Item {
    french: Option<String>,
    swedish: Option<String>,
    english: Option<String>,
    category: Category,
    category_int: u8,
}

impl Item {
    pub fn new(french: Option<String>, swedish: Option<String>, english: Option<String>, category: Category) -> Self {
        let category_int = category.to_u8();
        Self { french, swedish, english, category, category_int }
    }

    fn has_language(&self, language: &Language) -> bool {
        match language {
            Language::French => self.french.is_some(),
            Language::Swedish => self.swedish.is_some(),
            Language::English => self.english.is_some(),
        }
    }

    fn language_string(&self, language: &Language) -> &Option<String> {
        match language {
            Language::French => &self.french,
            Language::Swedish => &self.swedish,
            Language::English => &self.english,
        }
    }
}

pub enum Language {
    French,
    Swedish,
    English,
}

pub struct Query {
    string: String,
    language: Language,
    search_categories_int: u8,
}

pub struct Search {
    items: Vec<Item>,
}

impl Search {
    pub fn new(items: Vec<Item>) -> Self {
        Self { items }
    }

    pub fn search(&self, query: &Query, num_answers: usize) -> Vec<&Item> {
        let mut best_matches: Vec<&Item> = Vec::with_capacity(num_answers);
        let mut best_match_scores: Vec<usize> = vec![usize::MAX; num_answers];

        for item in &self.items {
            if let Some(string) = item.language_string(&query.language) {
                if item.category_int & query.search_categories_int != 0 {
                    let distance = levenshtein(&query.string, &string);
        
                    for i in 0..num_answers {
                        if distance < best_match_scores[i] {
                            best_match_scores.insert(i, distance);
                            best_match_scores.truncate(num_answers);
                            best_matches.insert(i, item);
                            best_matches.truncate(num_answers);
                            break;
                        }
                    }
                }
            }
        }

        best_matches
    }
}