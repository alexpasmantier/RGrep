use anyhow::{anyhow, Result};

use crate::parser::RE;

pub fn match_pattern(input_characters: &Vec<char>, pattern: &[RE]) -> Result<usize> {
    if let RE::StartAnchor = pattern[0] {
        return _match_pattern(input_characters, &pattern[1..], 0, Vec::new());
    }
    let mut counter: usize = 0;
    loop {
        if let Ok(match_len) = _match_pattern(&input_characters[counter..], &pattern, 0, Vec::new())
        {
            if match_len > 0 {
                return Ok(match_len);
            }
        }
        // this advances the input string at each iteration
        if input_characters.get(counter + 1) == None {
            return Ok(0);
        };
        counter += 1;
    }
}

#[allow(unused_assignments)]
pub fn _match_pattern(
    input_characters: &[char],
    pattern: &[RE],
    current_match_len: usize,
    mut back_references: Vec<String>,
) -> Result<usize> {
    // println!(
    //     "input: {:?}\t\t\tpattern: {}\t\t\tmatch len: {}\t\tbackrefs: {:?}",
    //     &input_characters,
    //     &pattern.get(0).unwrap_or(&RE::Character { character: '0' }),
    //     &current_match_len,
    //     &back_references
    // );
    // the whole pattern was consumed, we have a match
    if pattern.len() == 0 {
        return Ok(current_match_len);
    }
    match input_characters.get(0) {
        Some(c) => match &pattern[0] {
            RE::BackReference { target } => {
                if let Some(matched_string) = back_references.get((*target - 1) as usize) {
                    let matched_characters_pattern: Vec<RE> = matched_string
                        .chars()
                        .map(|ch| RE::Character { character: ch })
                        .collect();
                    return _match_pattern(
                        input_characters,
                        &[&matched_characters_pattern, &pattern[1..]].concat(),
                        current_match_len,
                        back_references,
                    );
                } else {
                    return Err(anyhow!("Back reference does not exist"));
                }
            }
            RE::Alternation { members } => {
                for member in members {
                    if let Ok(match_len) =
                        _match_pattern(input_characters, member, 0, back_references.clone())
                    {
                        if match_len > 0 {
                            back_references.push(input_characters[..match_len].iter().collect());
                            return _match_pattern(
                                &input_characters[match_len..],
                                &pattern[1..],
                                current_match_len + match_len,
                                back_references,
                            );
                        }
                    }
                }
                return Ok(0);
            }
            RE::OneOrMore { target: Some(atom) } => {
                // if there's a match, continue matching with the same atom zero or more times
                let mut counter = 1;
                if let Ok(true) = match_single_character(c, atom) {
                    loop {
                        if let Some(c) = &input_characters.get(counter) {
                            match match_single_character(c, atom) {
                                Ok(false) => break,
                                _ => counter += 1,
                            }
                        } else {
                            break;
                        }
                    }
                    return _match_pattern(
                        &input_characters[counter..],
                        &pattern[1..],
                        current_match_len + counter,
                        back_references,
                    );
                }
                return Ok(0);
            }
            RE::ZeroOrMore { target: Some(atom) } => {
                if let Ok(match_len) =
                    _match_pattern(input_characters, &pattern[1..], 0, back_references.clone())
                {
                    if match_len > 0 {
                        return Ok(current_match_len + match_len);
                    } else if let Ok(true) = match_single_character(c, atom) {
                        return _match_pattern(
                            input_characters,
                            pattern,
                            current_match_len + 1,
                            back_references,
                        );
                    } else {
                        return Ok(0);
                    }
                } else {
                    return Ok(0);
                }
            }
            RE::ZeroOrOne { target: Some(atom) } => {
                if let Ok(match_len) =
                    _match_pattern(input_characters, &pattern[1..], 0, back_references.clone())
                {
                    if match_len > 0 {
                        return Ok(current_match_len + match_len);
                    } else if let Ok(true) = match_single_character(c, atom) {
                        return _match_pattern(
                            &input_characters[1..],
                            &pattern[1..],
                            current_match_len + 1,
                            back_references,
                        );
                    } else {
                        return Ok(0);
                    }
                } else {
                    return Ok(0);
                }
            }
            re => {
                if let Ok(true) = match_single_character(c, re) {
                    return _match_pattern(
                        &input_characters[1..],
                        &pattern[1..],
                        current_match_len + 1,
                        back_references,
                    );
                } else {
                    return Ok(0);
                }
            }
        },
        None => {
            if let RE::EndAnchor = &pattern[0] {
                return Ok(current_match_len);
            } else {
                return Ok(0);
            }
        }
    }
}

fn match_single_character(c: &char, re_element: &RE) -> Result<bool> {
    match re_element {
        RE::Wildcard => {
            return Ok(true);
        }
        RE::Character { character } => {
            return Ok(_match_character(c, *character));
        }
        RE::DigitClass => {
            return Ok(_match_digit_class(c));
        }
        RE::AlphanumericClass => {
            return Ok(_match_alphanumeric_class(c));
        }
        RE::CClass { members } => {
            return _match_character_class(c, &members);
        }
        RE::NCClass { members } => {
            return Ok(!_match_character_class(c, &members)?);
        }
        _ => return Ok(false),
    }
}

fn _match_character(input_character: &char, re_char: char) -> bool {
    input_character == &re_char
}

fn _match_digit_class(input_character: &char) -> bool {
    input_character.is_ascii_digit()
}

fn _match_alphanumeric_class(input_character: &char) -> bool {
    input_character.is_ascii_alphanumeric()
}

fn _match_character_class(input_character: &char, character_class: &[RE]) -> Result<bool> {
    for c in character_class {
        if _match_pattern(&[*input_character], &[c.clone()], 0, Vec::new())? == 1 {
            return Ok(true);
        }
    }
    return Ok(false);
}
