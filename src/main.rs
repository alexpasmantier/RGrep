use std::env;
use std::io;
use std::process;

use crate::engine::match_pattern;
use crate::parser::parse;

mod engine;
mod parser;

// Usage: echo <input_text> | your_grep.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    // parsing the regex
    let pattern = env::args().nth(2).unwrap();
    if let Ok(regex) = parse(&pattern) {
        // dbg!(&regex);
        // matching against the regex
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        input_line.retain(|c| c != '\n');
        let input_characters = input_line.chars();

        let input_chars: Vec<char> = input_characters.collect();
        if let Ok(match_len) = match_pattern(&input_chars, &regex) {
            if match_len > 0 {
                println!("MATCH, size {}", match_len);
                process::exit(0)
            }
        }
    }
    process::exit(1)
}
