pub fn get_plural(string: &str) -> String {
    if string.ends_with("al") || string.ends_with("au") {
        string[0..string.len()-2].to_string() + "aux"
    } else if string.ends_with("ail") {
        string[0..string.len()-3].to_string() + "aux"
    } else if string.ends_with("eu") || string.ends_with("ou") {
        string.to_string() + "x"
    } else if !string.ends_with("s") && !string.ends_with("x") && !string.ends_with("z") {
        string.to_string() + "s"
    } else {
        string.to_string()
    }
}

pub fn number_forms(cardinal: &str) -> (String, String) {
    let ordinal = if cardinal.ends_with("e") {
        cardinal[0..cardinal.len()-1].to_string() + "ième"
    } else {
        cardinal.to_string() + "ième"
    };
    let approximate = if cardinal.ends_with("e") {
        cardinal[0..cardinal.len()-1].to_string() + "aine"
    } else if cardinal.ends_with("x") {
        cardinal[0..cardinal.len()-1].to_string() + "zaine"
    } else {
        cardinal.to_string() + "aine"
    };
    (ordinal, approximate)
}