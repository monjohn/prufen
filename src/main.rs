
extern crate ansi_term;
#[macro_use]
extern crate serde_derive;
extern crate csv;
extern crate rand;
extern crate regex;
// for command line args
extern crate getopts;
// use getopts::Options;
// use getopts::Matches;
// use std::env;
use ansi_term::Colour::{Red, Green};
use std::error::Error;
use std::io;
use std::process;
use std::io::BufReader;
use std::collections::HashMap;
use std::fs::File;
use rand::{thread_rng, Rng};
use regex::Regex;

#[allow(dead_code)]
fn only_nouns(word_pairs: &Vec<WordPair>) -> Vec<WordPair> {
    let re = Regex::new(r"^(der |die |das )").unwrap();
    let pairs = word_pairs.iter()
        .filter(|&pair| re.is_match(&pair.german[..]));
    pairs.map(|noun| noun.clone()).collect()
}

fn only_verbs(word_pairs: &Vec<WordPair>) -> Vec<WordPair> {
    let re = Regex::new(r"^(der |die |das )").unwrap();
    let pairs = word_pairs.iter()
        .filter(|&pair| !re.is_match(&pair.german[..]));
    pairs.map(|noun| noun.clone()).collect()
}

#[derive(Debug,Deserialize,Clone)]
struct WordPair {
    root: String,
    german: String,
    english: String,
}

fn read_file() -> Result<Vec<WordPair>, Box<Error>> {
    let dictionary = "/Users/monte/Desktop/word-families.csv";
    let mut word_vec = Vec::new();

    let f = File::open(dictionary).unwrap();
    let reader = BufReader::new(&f);

    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: WordPair = result?;
        word_vec.push(record);
    }
    Ok(word_vec)
}

fn sorted_by_root(wordpairs: Vec<WordPair>) -> HashMap<String, Vec<WordPair>> {
    wordpairs.iter()
        .fold(HashMap::new(), |mut acc, pair| {
            acc.entry(pair.root.clone())
                .or_insert(Vec::new())
                .push(pair.clone());
            acc
        })
}

fn pick_one<T>(coll: &Vec<T>) -> T
    where T: std::clone::Clone
{
    let random_pick: usize = thread_rng().gen_range(0, coll.len());
    coll[random_pick].clone()
}

fn select_nouns_or_verbs(words: &Vec<WordPair>) -> Vec<WordPair> {
    let random_pick = thread_rng().gen_weighted_bool(6);
    match random_pick {
        true => only_verbs(words),
        false => only_nouns(words),
    }
}

fn select_four(words: &Vec<WordPair>) -> Vec<WordPair> {
    let mut rng = thread_rng();
    let mut copied = words.clone();
    let shuffled = copied.as_mut_slice();
    rng.shuffle(shuffled);
    let num: usize = if shuffled.len() > 3 {
        4
    } else {
        shuffled.len()
    };
    shuffled[..num].to_vec()
}

fn print_options(words: &Vec<WordPair>) -> () {
    for (i, ref word_pair) in words.iter().enumerate() {
        println!("{} - {}", i + 1, word_pair.english);
    }

}

fn guess(options: &Vec<WordPair>, answer: &String) -> bool {
    let mut guess = String::new();
    let mut guess_num: i32;

    loop {
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        if guess.trim() == "q" {
            process::exit(1)
        }

        guess_num = match guess.trim().parse::<i32>() {
            Ok(i) => i,
            Err(_) => -1,
        };

        if guess_num < 1 || guess_num > options.len() as i32 {
            println!("Your guess is invalid");
        } else {
            break;
        }
    }

    let ref german = options[(guess_num - 1) as usize].german;

    german == answer
}

fn main() {
    let words = read_file().unwrap();
    let sorted_by_root = sorted_by_root(words);

    loop {
        let root_keys = sorted_by_root.keys();

        let keys = root_keys.map(|noun| noun.clone()).collect::<Vec<String>>();
        let current_root = pick_one(&keys);
        let selected_wordpairs = sorted_by_root.get(&current_root.to_string())
            .expect("Where did the words go?");

        let nouns_or_verbs = select_nouns_or_verbs(&selected_wordpairs);
        let options = select_four(&nouns_or_verbs);
        if options.len() == 0 {
            continue;
        }
        let word_pair = pick_one(&options);


        println!("\n{} – Which definition is correct?\n", word_pair.german);
        print_options(&options);

        let mut attempts: usize = 0;
        while attempts < options.len() as usize {
            attempts += 1;
            match guess(&options, &word_pair.german) {
                true => {
                    println!("{}", Green.paint("You are correct!"));
                    break;
                }
                false => {
                    println!("{}", Red.paint("That is incorrect!"));

                }
            }
        }

        println!("{} – {}", word_pair.german, word_pair.english);
        println!("–––––––––––––––––––––––––––––––––––––––––––––");

        let mut temp = String::new();
        io::stdin()
            .read_line(&mut temp)
            .expect("Failed to read line");
    }
}
