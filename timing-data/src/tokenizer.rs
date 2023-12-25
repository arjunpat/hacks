use crate::utils::CompilerError;
use crate::utils::Scanner;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TokenInfo<LiteralType> {
    lexeme: String,       // the actual text of the lexeme
    literal: LiteralType, // the value of the lexeme
    byte_idx: u32,        // the byte index of the lexeme
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Date {
    year: u32,
    month: u32,
    day: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Time {
    hour: u32,
    minute: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Token {
    // Single-character tokens.
    Asterisk(TokenInfo<String>),
    NewLine(TokenInfo<String>),
    Hyphen(TokenInfo<String>),
    // Eof(TokenInfo<String>),

    // Literals.
    Identifier(TokenInfo<String>),
    PeriodName(TokenInfo<String>),
    String(TokenInfo<String>),
    Date(TokenInfo<Date>),
    Time(TokenInfo<Time>),

    // Keywords.
    Repeat(TokenInfo<String>),
    Calendar(TokenInfo<String>),
    Periods(TokenInfo<String>),
    NonPeriods(TokenInfo<String>),
    Schedule(TokenInfo<String>),
}

fn make_string_token(lexeme: String, byte_idx: u32) -> Result<Token> {
    let literal = lexeme[1..lexeme.len() - 1].to_string();
    let re: Regex = Regex::new(r#"^[a-zA-Z0-9 \-'().]+$"#).unwrap();

    if !re.is_match(&literal) {
        return Err(anyhow!(CompilerError::new(
            byte_idx,
            byte_idx + lexeme.len() as u32 - 1,
            "string contains invalid chars",
            "expected a-z, A-Z, 0-9, -, ', (, ), and space"
        )));
    }

    Ok(Token::String(TokenInfo {
        lexeme,
        literal,
        byte_idx,
    }))
}

fn make_newline_token(byte_idx: u32) -> Token {
    Token::NewLine(TokenInfo {
        lexeme: "\n".to_string(),
        literal: "\n".to_string(),
        byte_idx,
    })
}

fn make_date_token(lexeme: String, byte_idx: u32) -> Result<Token> {
    let parts: Vec<_> = lexeme.split('/').collect();

    let parse_err = anyhow!(CompilerError::new(
        byte_idx,
        byte_idx + lexeme.len() as u32 - 1,
        "invalid date",
        "expected format: MM/DD/YYYY"
    ));

    if parts.len() != 3 {
        return Err(parse_err);
    }

    let parts: Vec<_> = parts.into_iter().map(|s| s.parse::<u32>()).collect();

    if parts.iter().any(|r| r.is_err()) {
        return Err(parse_err);
    }

    let mut parts = parts.into_iter().map(|r| r.unwrap());

    let date = Date {
        month: parts.next().unwrap(),
        day: parts.next().unwrap(),
        year: parts.next().unwrap(),
    };

    if date.month > 12 || date.day > 31 {
        return Err(parse_err);
    }

    Ok(Token::Date(TokenInfo {
        lexeme,
        literal: date,
        byte_idx,
    }))
}

fn make_time_token(lexeme: String, byte_idx: u32) -> Result<Token> {
    let parse_error = anyhow!(CompilerError::new(
        byte_idx,
        byte_idx + lexeme.len() as u32 - 1,
        "invalid time",
        "i thought this was a time but did not get HH:MM"
    ));

    let parts: Vec<_> = lexeme.split(':').collect();

    if parts.len() != 2 {
        return Err(parse_error);
    }

    let parts: Vec<_> = parts.into_iter().map(|s| s.parse::<u32>()).collect();

    if parts.iter().any(|r| r.is_err()) {
        return Err(parse_error);
    }

    let mut parts = parts.into_iter().map(|r| r.unwrap());

    let literal = Time {
        hour: parts.next().unwrap(),
        minute: parts.next().unwrap(),
    };

    if literal.hour > 23 || literal.minute > 59 {
        return Err(parse_error);
    }

    Ok(Token::Time(TokenInfo {
        lexeme,
        literal,
        byte_idx,
    }))
}

fn read_periods(scanner: &mut Scanner, tokens: &mut Vec<Token>) {
    let mut lexeme = String::new();

    while let Some(period_char) = scanner.peek_char() {
        if period_char == '*' {
            break;
        }

        scanner.read_char();
        if period_char == '\n' {
            if lexeme.len() > 0 {
                tokens.push(Token::PeriodName(TokenInfo {
                    lexeme: lexeme.clone(),
                    literal: lexeme.clone(),
                    byte_idx: scanner.get_pos() - lexeme.len() as u32,
                }));
                lexeme.clear();
            }
            tokens.push(make_newline_token(scanner.get_pos()));
        } else {
            lexeme.push(period_char);
        }
    }
}

fn tokenizer(scanner: &mut Scanner) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();

    while let Some(c) = scanner.read_char() {
        match c {
            '*' => {
                tokens.push(Token::Asterisk(TokenInfo {
                    lexeme: "*".to_string(),
                    literal: "*".to_string(),
                    byte_idx: scanner.get_pos(),
                }));
            }
            '\n' => {
                tokens.push(make_newline_token(scanner.get_pos()));
            }
            '"' => {
                // beginning of a string
                let byte_pos = scanner.get_pos();
                let mut lexeme = String::new();
                lexeme.push(c);

                while let Some(string_char) = scanner.read_char() {
                    lexeme.push(string_char);
                    if string_char == '"' {
                        break;
                    } else if string_char == '\n' {
                        return Err(anyhow!(CompilerError::new(
                            byte_pos,
                            scanner.get_pos() - 1,
                            "unterminated string",
                            "strings must be enclosed in double quotes"
                        )));
                    }
                }
                tokens.push(make_string_token(lexeme, byte_pos)?);
            }
            '#' => {
                // comment
                while let Some(comment_char) = scanner.peek_char() {
                    if comment_char == '\n' {
                        break;
                    }
                    scanner.read_char();
                }
            }
            '-' => {
                // hyphen
                tokens.push(Token::Hyphen(TokenInfo {
                    lexeme: "-".to_string(),
                    literal: "-".to_string(),
                    byte_idx: scanner.get_pos(),
                }));
            }
            '0'..='9' => {
                // date or time
                let mut lexeme = String::new();
                lexeme.push(c);
                let byte_idx = scanner.get_pos();
                let mut is_time = false;

                while let Some(date_char) = scanner.peek_char() {
                    let date_uni = date_char as u8;
                    let valid = (date_uni >= b'0' && date_uni <= b'9')
                        || date_uni == b':'
                        || date_uni == b'/';
                    if !valid {
                        break;
                    } else {
                        is_time = is_time || date_char == ':';
                        lexeme.push(date_char);
                        scanner.read_char();
                    }
                }

                if is_time {
                    tokens.push(make_time_token(lexeme, byte_idx)?);

                    // assert space after time
                    if let Some(space_char) = scanner.read_char() {
                        if space_char != ' ' {
                            return Err(anyhow!(CompilerError::new(
                                scanner.get_pos(),
                                scanner.get_pos(),
                                "expected space after time",
                                "schedule items are of form {time} {period name}\\n"
                            )));
                        }
                    } else {
                        return Err(anyhow!(CompilerError::new(
                            scanner.get_pos(),
                            scanner.get_pos(),
                            "expected space after time",
                            "schedule items are of form {time} {period name}\\n"
                        )));
                    }

                    // parse PeriodName after time
                    let byte_idx = scanner.get_pos();
                    let mut pn_lexeme = String::new();
                    while let Some(pn_char) = scanner.peek_char() {
                        if pn_char == '\n' {
                            break;
                        } else {
                            pn_lexeme.push(pn_char);
                            scanner.read_char();
                        }
                    }

                    tokens.push(Token::PeriodName(TokenInfo {
                        lexeme: pn_lexeme.clone(),
                        literal: pn_lexeme.clone(),
                        byte_idx: byte_idx + 1,
                    }));
                } else {
                    tokens.push(make_date_token(lexeme, byte_idx)?);
                }
            }
            'a'..='z' => {
                // identifier
                let byte_idx = scanner.get_pos();
                let mut lexeme = String::new();
                lexeme.push(c);

                while let Some(identifier_char) = scanner.peek_char() {
                    if identifier_char == ' ' || identifier_char == '\n' {
                        break;
                    } else {
                        lexeme.push(identifier_char);
                        scanner.read_char();
                    }
                }

                match lexeme.as_str() {
                    "repeat" => {
                        tokens.push(Token::Repeat(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            byte_idx,
                        }));
                    }
                    "calendar" => {
                        tokens.push(Token::Calendar(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            byte_idx,
                        }));
                    }
                    "periods" => {
                        match &tokens[tokens.len() - 1] {
                            Token::Asterisk(_) => {}
                            _ => {
                                return Err(anyhow!(CompilerError::new(
                                    scanner.get_pos() - lexeme.len() as u32 + 1,
                                    scanner.get_pos(),
                                    "periods must be preceded by an asterisk",
                                    "directives are of form * {directive}"
                                )));
                            }
                        }
                        tokens.push(Token::Periods(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            byte_idx,
                        }));

                        // read until next directive (*)
                        read_periods(scanner, &mut tokens);
                    }
                    "non-periods" => {
                        match &tokens[tokens.len() - 1] {
                            Token::Asterisk(_) => {}
                            _ => {
                                return Err(anyhow!(CompilerError::new(
                                    scanner.get_pos() - lexeme.len() as u32 + 1,
                                    scanner.get_pos(),
                                    "non-periods must be preceded by an asterisk",
                                    "directives are of form * {directive}"
                                )));
                            }
                        }

                        tokens.push(Token::NonPeriods(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            byte_idx,
                        }));

                        // read until next directive (*)
                        read_periods(scanner, &mut tokens);
                    }
                    "schedule" => {
                        match &tokens[tokens.len() - 1] {
                            Token::Asterisk(_) => {}
                            _ => {
                                return Err(anyhow!(CompilerError::new(
                                    scanner.get_pos() - lexeme.len() as u32 + 1,
                                    scanner.get_pos(),
                                    "schedule must be preceded by an asterisk",
                                    "directives are of form * {directive}"
                                )));
                            }
                        }

                        tokens.push(Token::Schedule(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            byte_idx,
                        }));
                    }
                    _ => {
                        let regex = Regex::new(r#"^[a-z]+([a-z0-9]+[-])*[a-z0-9]+$"#).unwrap();
                        if !regex.is_match(&lexeme) {
                            return Err(anyhow!(CompilerError::new(
                                byte_idx,
                                byte_idx + lexeme.len() as u32 - 1,
                                "invalid identifier",
                                "expected a-z, 0-9, -, must start with a-z, and end with a-z or 0-9"
                            )));
                        }

                        tokens.push(Token::Identifier(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            byte_idx,
                        }));
                    }
                }
            }
            ' ' => {}
            _ => {
                return Err(anyhow!(CompilerError::new(
                    scanner.get_pos(),
                    scanner.get_pos(),
                    &format!("unexpected character: {} (unicode: {})", c, c as u32),
                    "expected a-z, A-Z, 0-9, *, #, \", -, :, /, \\n, and space"
                )));
            }
        }
    }

    Ok(tokens)
}

pub fn make_tokens(scanner: &mut Scanner) -> Result<Vec<Token>> {
    let res = tokenizer(scanner);

    if let Err(e) = res {
        let downcast: CompilerError = e.downcast()?;
        let mut err_string = String::new();
        downcast.fmt_for_terminal(&mut err_string, scanner)?;
        return Err(anyhow!(err_string));
    }

    res
}
