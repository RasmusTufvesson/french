use rand::seq::SliceRandom;
use crate::search::{Category, Item, Language, Pronoun, Query, Search, VerbForms};

pub fn generate(words: &Search, subject: Option<Item>, verb: Option<Item>) -> Vec<(String, Item)> {
    let mut rng = rand::thread_rng();
    let mut sentence = vec![];
    let subject = subject.unwrap_or_else(|| words.random_item(&Query::new(&"".to_string(), &Language::French, 0b100000000, false), &mut rng));
    let verb = verb.unwrap_or_else(|| words.random_item(&Query::new(&"".to_string(), &Language::French, 0b10, false), &mut rng));
    
    if let Category::Pronoun(Pronoun::Personal(pronoun, _, _, _)) = &subject.category {
        sentence.push((pronoun.clone(), subject.clone()));
        if let Category::Verb(_, forms) = &verb.category {
            let (VerbForms::Regular(je, tu, il, nous, vous, ils, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils) | VerbForms::Irregular(je, tu, il, nous, vous, ils, pc, imp_je, imp_tu, imp_il, imp_nous, imp_vous, imp_ils)) = &forms;
            sentence.push(((**match pronoun.as_str() {
                "je" => vec![je, imp_je],
                "tu" => vec![tu, imp_tu],
                "il" | "elle" | "on" => vec![il, imp_il],
                "nous" => vec![nous, imp_nous],
                "vous" => vec![vous, imp_vous],
                "ils" | "elles" => vec![ils, imp_ils],
                _ => unreachable!(),
            }.choose(&mut rng).unwrap()).clone(), verb));
        }
    }
    
    if sentence.len() != 0 {
        let first = sentence[0].0.remove(0).to_uppercase().next().unwrap();
        sentence[0].0.insert(0, first);
        let index = sentence.len()-1;
        sentence[index].0 += ".";
    }

    sentence
}