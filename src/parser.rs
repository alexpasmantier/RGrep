use anyhow::{anyhow, Result};
use core::fmt;

#[derive(Clone, Debug)]
pub enum RE {
    Character { character: char },
    CClass { members: Box<Vec<RE>> },
    NCClass { members: Box<Vec<RE>> },
    DigitClass,
    AlphanumericClass,
    Wildcard,
    StartAnchor,
    EndAnchor,
    OneOrMore { target: Option<Box<RE>> },
    ZeroOrMore { target: Option<Box<RE>> },
    ZeroOrOne { target: Option<Box<RE>> },
    // also used as capture groups
    Alternation { members: Vec<Vec<RE>> },
    BackReference { target: u32 },
}

impl fmt::Display for RE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            RE::Character { character } => write!(f, "{}", character),
            // I'll implement that later
            RE::CClass { members: _ } => write!(f, "cclass"),
            RE::NCClass { members: _ } => write!(f, "ncclass"),
            RE::DigitClass => write!(f, r"\d"),
            RE::AlphanumericClass => write!(f, r"\w"),
            RE::Wildcard => write!(f, "."),
            RE::StartAnchor => write!(f, "^"),
            RE::EndAnchor => write!(f, "$"),
            RE::OneOrMore { target } => write!(f, "{}+", target.as_ref().unwrap()),
            RE::ZeroOrMore { target } => write!(f, "{}*", target.as_ref().unwrap()),
            RE::ZeroOrOne { target } => write!(f, "{}?", target.as_ref().unwrap()),
            RE::Alternation { members: _ } => write!(f, "alternation"),
            RE::BackReference { target } => write!(f, r"\{}", target),
        }
    }
}

#[allow(unused_variables)]
pub fn parse(regex_string: &str) -> Result<Vec<RE>> {
    let characters: Vec<char> = regex_string.chars().collect();
    let mut regex: Vec<RE> = Vec::new();
    // check for an anchor
    let mut counter: usize = 0;
    if characters[0] == '^' {
        regex.push(RE::StartAnchor);
        counter += 1;
    }
    let (_, regex) = _parse(&characters[counter..], &mut regex)?;
    let regex = &_post_process(regex.to_vec())?;
    Ok(regex.to_owned())
}

fn _parse<'a, 'b>(
    characters: &'a [char],
    regex: &'b mut Vec<RE>,
) -> Result<(&'a [char], &'b Vec<RE>)> {
    match characters.get(0) {
        // alternation (basic implementation, without recursion)
        Some('(') => {
            let mut parts = characters[1..].splitn(2, |c| *c == ')');
            let chars_in_alternation = parts.next().expect("empty group");
            let alternation_parts = chars_in_alternation.split(|c| *c == '|');
            let alternation_members: Vec<Vec<RE>> = alternation_parts
                .map(|member| {
                    let mut member_regex: Vec<RE> = Vec::new();
                    _parse(member, &mut member_regex)
                        .expect("unable to parse alternation member")
                        .1
                        .to_owned()
                })
                .collect();
            regex.push(RE::Alternation {
                members: alternation_members,
            });
            if let Some(rest) = parts.next() {
                return _parse(rest, regex);
            } else {
                return _parse(&[], regex);
            }
        }
        // special character classes
        Some('\\') => {
            match characters.get(1) {
                // digit class
                Some('d') => {
                    regex.push(RE::DigitClass);
                    _parse(&characters[2..], regex)
                }
                Some('w') => {
                    regex.push(RE::AlphanumericClass);
                    _parse(&characters[2..], regex)
                }
                Some(c) => {
                    if c.is_ascii_digit() {
                        regex.push(RE::BackReference {
                            target: c.to_digit(10).unwrap(),
                        });
                        _parse(&characters[2..], regex)
                    } else {
                        return Err(anyhow!("Unsupported special character class {c}"));
                    }
                }
                None => {
                    return Err(anyhow!("Missing special character class specifier"));
                }
            }
        }
        // regular characters classes
        Some('[') => {
            let mut counter: usize = 1;
            let mut negated = false;
            if characters.get(1) == Some(&'^') {
                negated = true;
                counter += 1;
            }
            let mut parts = characters[counter..].splitn(2, |c| *c == ']');
            let cls_characters = parts.next().expect("Invalid pattern");
            let mut cls_members: Vec<RE> = Vec::new();
            let (_, cls_members) = _parse(&cls_characters, &mut cls_members)?;
            if negated {
                regex.push(RE::NCClass {
                    members: Box::new(cls_members.to_vec()),
                });
            } else {
                regex.push(RE::CClass {
                    members: Box::new(cls_members.to_vec()),
                });
            }
            if let Some(rest) = parts.next() {
                return _parse(rest, regex);
            } else {
                return _parse(&[], regex);
            }
        }
        Some('$') => {
            regex.push(RE::EndAnchor);
            if characters.get(1) != None {
                return Err(anyhow!(
                    "Invalid pattern, `$` should only appear as the last character"
                ));
            } else {
                return Ok((&[], regex));
            }
        }
        Some('+') => {
            regex.push(RE::OneOrMore { target: None });
            return _parse(&characters[1..], regex);
        }
        Some('*') => {
            regex.push(RE::ZeroOrMore { target: None });
            return _parse(&characters[1..], regex);
        }
        Some('?') => {
            regex.push(RE::ZeroOrOne { target: None });
            return _parse(&characters[1..], regex);
        }
        Some('.') => {
            regex.push(RE::Wildcard);
            return _parse(&characters[1..], regex);
        }
        // regular character
        Some(c) => {
            regex.push(RE::Character { character: *c });
            return _parse(&characters[1..], regex);
        }
        None => return Ok((&[], regex)),
    }
}

fn _post_process(regex: Vec<RE>) -> Result<Vec<RE>> {
    let mut new_regex: Vec<RE> = Vec::new();
    for i in 0..regex.len() {
        if let RE::Alternation { members } = &regex[i] {
            let mut new_members: Vec<Vec<RE>> = Vec::new();
            for member in members {
                let new_member = _post_process(member.to_owned())?;
                new_members.push(new_member);
            }
            new_regex.push(RE::Alternation {
                members: new_members,
            });
        } else if let Some(variant @ RE::OneOrMore { target: _ })
        | Some(variant @ RE::ZeroOrMore { target: _ })
        | Some(variant @ RE::ZeroOrOne { target: _ }) = &regex.get(i + 1)
        {
            let new_target = Some(Box::new(regex.get(i).unwrap().clone()));
            match *variant {
                RE::OneOrMore { target: _ } => {
                    new_regex.push(RE::OneOrMore { target: new_target });
                }
                RE::ZeroOrMore { target: _ } => {
                    new_regex.push(RE::ZeroOrMore { target: new_target });
                }
                RE::ZeroOrOne { target: _ } => {
                    new_regex.push(RE::ZeroOrOne { target: new_target });
                }
                _ => {}
            }
        } else if let RE::OneOrMore { target: _ }
        | RE::ZeroOrMore { target: _ }
        | RE::ZeroOrOne { target: _ } = &regex[i]
        {
            // do nothing
        } else if let RE::CClass { members } = &regex[i] {
            if members.iter().any(|v| {
                matches!(
                    v,
                    RE::OneOrMore { target: _ }
                        | RE::ZeroOrMore { target: _ }
                        | RE::ZeroOrOne { target: _ }
                )
            }) {
                return Err(anyhow!("character class cannot contain `+`"));
            }
            new_regex.push(RE::CClass {
                members: members.clone(),
            });
        } else if let RE::NCClass { members } = &regex[i] {
            if members.iter().any(|v| {
                matches!(
                    v,
                    RE::OneOrMore { target: _ }
                        | RE::ZeroOrMore { target: _ }
                        | RE::ZeroOrOne { target: _ }
                )
            }) {
                return Err(anyhow!("character class cannot contain `+`"));
            }
            new_regex.push(RE::NCClass {
                members: members.clone(),
            });
        } else {
            new_regex.push(regex[i].clone());
        }
    }
    Ok(new_regex)
}
