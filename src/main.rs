mod search;
mod ui;
mod practice;

const WORDS_FILE: &str = "words.bin";
const SENTENCES_FILE: &str = "sentences.bin";

fn main() {
    let engine_words = search::Search::load_or_new(WORDS_FILE);
    let engine_sentences = search::Search::load_or_new(SENTENCES_FILE);
    ui::run(engine_words, engine_sentences, WORDS_FILE.to_string(), SENTENCES_FILE.to_string()).unwrap();
}
