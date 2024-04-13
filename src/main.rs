use regex::Regex;
use std::{collections::HashMap, fs, io::Read};

fn main() {
    let entropy = precompute_entropy();
    let mut valid_words = {
        let mut valid: String = String::from("");
        let _ = fs::File::open("./wordle.list")
            .unwrap()
            .read_to_string(&mut valid);
        let words: Vec<String> = valid.lines().map(String::from).collect();
        words
    };
    //MainWindow::new().unwrap().run().unwrap();
    let mut correct = [
        "\\w".to_string(),
        "\\w".to_string(),
        "\\w".to_string(),
        "\\w".to_string(),
        "\\w".to_string(),
    ];
    let mut incorrect = [
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    ];
    let mut nullified = "[]".to_string();
    let mut found = false;

    while !found {
        let info = get_word_info();
        for idx in 0..info.len() {
            match info.get(idx) {
                None => panic!("Wot"),
                Some(wl) => match wl.state {
                    LetterState::CorrectPlace => {
                        let brw = &mut correct[idx];
                        let mut buf: [u8; 4] = [0; 4];
                        *brw = wl.c.encode_utf8(&mut buf).to_string();
                    }
                    LetterState::Nullifed => {
                        let brw = &mut nullified;
                        if incorrect.join("").contains(wl.c)
                            || correct.iter().any(|e| e == wl.c.to_string().as_str())
                        {
                            continue;
                        }
                        brw.insert(1, wl.c);
                    }
                    LetterState::IncorrectPlace => {
                        let brw = &mut incorrect[idx];
                        brw.insert(0, wl.c);
                    }
                },
            }
        }
        let correct_regex = Regex::new(&correct.join("")).unwrap();
        let nullified_regex = Regex::new(&nullified).unwrap();
        valid_words = valid_words
            .into_iter()
            .filter(|e| {
                correct_regex.find(e).is_some()
                    && nullified_regex.find(e).is_none()
                    && e.chars().enumerate().all(|(i, val)| {
                        if incorrect[i] != "" {
                            return !incorrect[i].chars().any(|c| c == val);
                        }
                        true
                    })
                    && incorrect.join("").as_str().chars().all(|c| e.contains(c))
            })
            .collect();
        if valid_words.len() >= 2 {
            let mut display_clone = valid_words.clone();
            display_clone
                .sort_unstable_by(|a, b| entropy.get(a).partial_cmp(&entropy.get(b)).unwrap());
            println!("Possible Words: {:?}", display_clone);
            drop(display_clone);
        } else {
            println!("FOUND! {}", valid_words[0]);
            found = !found;
        }
    }
}

fn get_word_info() -> Vec<WordleLetter> {
    let mut current_word: String = "".to_string();
    let mut state: String = "".to_string();
    while current_word.trim_end().len() != 5 {
        current_word = String::from("");
        println!("Current Guess:");
        let _ = std::io::stdin().read_line(&mut current_word);
    }
    while state.trim_end().len() != 5
        || state
            .chars()
            .all(|c| matches!(c.to_uppercase().last().unwrap(), 'N' | 'Y' | 'G'))
    {
        state = String::from("");
        println!("Give letter state\n\tN: Grey (No match)\n\tG: Green (Correct Placement)\n\tY: Yellow (Correct letter incorrect placement)");
        let _ = std::io::stdin().read_line(&mut state);
    }
    let mut letter_state: Vec<WordleLetter> = Vec::new();
    state = state.trim_end().to_string();
    current_word = current_word.trim_end().to_string();
    for (l, s) in current_word.chars().zip(state.chars()) {
        letter_state.push(WordleLetter {
            c: l,
            state: match s.to_uppercase().last().unwrap() {
                'N' => LetterState::Nullifed,
                'Y' => LetterState::IncorrectPlace,
                'G' => LetterState::CorrectPlace,
                _ => panic!("Incorrect Letter"),
            },
        })
    }

    letter_state
}

fn precompute_entropy() -> std::collections::HashMap<String, f32> {
    let mut letter_count: HashMap<char, u32> = HashMap::new();
    letter_count.reserve(26);
    for charpoint in 'a'..='z' {
        letter_count.insert(charpoint, 0);
    }
    fs::File::open("./wordle.list")
        .unwrap()
        .bytes()
        .for_each(|c| {
            let c = c.unwrap();
            if c != b'\n' {
                letter_count.insert(
                    c as char,
                    letter_count.get(&(c as char)).expect(
                        ("Got none on ".to_owned() + std::str::from_utf8(&[c]).unwrap()).as_str(),
                    ) + 1,
                );
            }
        });

    let mut total: f32 = 0f32;
    letter_count.values().for_each(|n| total += *n as f32);
    let mut relative_frequency: HashMap<char, f32> = HashMap::new();
    for (k, v) in letter_count.into_iter() {
        relative_frequency.insert(k, v as f32 / total);
    }

    let words = {
        let mut lines: String = String::from("");
        let _ = fs::File::open("./wordle.list")
            .unwrap()
            .read_to_string(&mut lines);
        let lines: Vec<String> = lines.lines().map(String::from).collect();
        lines
    };

    let mut word_information: HashMap<String, f32> = HashMap::new();

    for word in words.into_iter() {
        let mut entropy = 0f32;
        word.chars()
            .collect::<std::collections::HashSet<char>>()
            .iter()
            .for_each(|c| entropy += relative_frequency.get(c).unwrap());
        word_information.insert(word, -entropy.log2());
    }
    word_information
}

struct WordleLetter {
    pub c: char,
    pub state: LetterState,
}

enum LetterState {
    CorrectPlace,
    Nullifed,
    IncorrectPlace,
}
