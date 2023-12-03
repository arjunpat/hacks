use anyhow::{anyhow, Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TokenInfo<LiteralType> {
    lexeme: String,       // the actual text of the lexeme
    literal: LiteralType, // the value of the lexeme
    line: u32,
    col: u32, // points to the beginning of lexeme
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Date {
    year: u32,
    month: u32,
    day: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Time {
    hour: u32,
    minute: u32,
}

impl FromStr for Date {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split('/');
        let month = parts
            .next()
            .ok_or(anyhow!("Invalid date"))?
            .parse::<u32>()?;
        let day = parts
            .next()
            .ok_or(anyhow!("Invalid date"))?
            .parse::<u32>()?;
        let year = parts
            .next()
            .ok_or(anyhow!("Invalid date"))?
            .parse::<u32>()?;

        Ok(Date { year, month, day })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Token {
    // Single-character tokens.
    Asterisk(TokenInfo<String>),
    NewLine(TokenInfo<String>),
    // Eof(TokenInfo<String>),

    // Literals.
    Identifier(TokenInfo<String>),
    String(TokenInfo<String>),
    Date(TokenInfo<Date>),
    DateRange(TokenInfo<(Date, Date)>),
    Time(TokenInfo<Time>),

    // Keywords.
    Repeat(TokenInfo<String>),
    Calendar(TokenInfo<String>),
    Periods(TokenInfo<String>),
    NonPeriods(TokenInfo<String>),
    Schedule(TokenInfo<String>),
}

pub struct Scanner {
    col: u32,
    reset_cols: bool,
    line: u32,
    reader: BufReader<File>,
    eof: bool,
}

impl Scanner {
    pub fn new(filename: String) -> Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        Ok(Scanner {
            line: 1,
            col: 1,
            reader,
            eof: false,
            reset_cols: false,
        })
    }

    pub fn read_char(&mut self) -> Result<char> {
        if self.eof {
            return Err(anyhow!("No more to read"));
        }

        let mut one_char: [u8; 1] = [0; 1];
        let bytes_read = self.reader.read(&mut one_char)?;

        if bytes_read == 0 {
            self.eof = true;
            return Err(anyhow!("No more to read"));
        }
        let c = one_char[0] as char;

        if self.reset_cols {
            self.col = 1;
            self.line += 1;
            self.reset_cols = false;
        } else {
            self.col += 1;
        }

        if c == '\n' {
            self.reset_cols = true;
        }
        // if '\r', may want to skip

        Ok(c)
    }

    pub fn get_line(&self) -> u32 {
        self.line
    }

    pub fn get_col(&self) -> u32 {
        self.col
    }

    // peek at the next character without consuming it
    pub fn peek_char(&mut self) -> Result<char> {
        if self.eof {
            return Err(anyhow!("No more to read"));
        }

        let mut one_char: [u8; 1] = [0; 1];
        let bytes_read = self.reader.read(&mut one_char)?;

        if bytes_read == 0 {
            self.eof = true;
            return Err(anyhow!("No more to read"));
        }

        self.reader.seek_relative(-1)?;

        Ok(one_char[0] as char)
    }
}

fn make_string_token(lexeme: String, line: u32, col: u32) -> Result<Token> {
    let literal = lexeme[1..lexeme.len() - 1].to_string();
    let re: Regex = Regex::new(r#"^[a-zA-Z0-9 \-']+$"#).unwrap();

    if !re.is_match(&literal) {
        return Err(anyhow!("Invalid string on line {} near {}", line, lexeme));
    }

    Ok(Token::String(TokenInfo {
        lexeme,
        literal,
        line,
        col,
    }))
}

fn make_newline_token(line: u32, col: u32) -> Token {
    Token::NewLine(TokenInfo {
        lexeme: "\n".to_string(),
        literal: "\n".to_string(),
        line,
        col,
    })
}

fn make_time_token(lexeme: String, line: u32, col: u32) -> Result<Token> {
    let mut parts = lexeme.split(':');
    let hour = parts.next().unwrap().parse::<u32>()?;
    let minute = parts.next().unwrap().parse::<u32>()?;

    Ok(Token::Time(TokenInfo {
        lexeme,
        literal: Time { hour, minute },
        line,
        col,
    }))
}

fn read_periods(scanner: &mut Scanner, tokens: &mut Vec<Token>) -> Result<()> {
    let mut lexeme = String::new();

    while let Ok(period_char) = scanner.peek_char() {
        if period_char == '*' {
            break;
        }

        scanner.read_char()?;
        if period_char == '\n' {
            if lexeme.len() > 0 {
                tokens.push(Token::Identifier(TokenInfo {
                    lexeme: lexeme.clone(),
                    literal: lexeme.clone(),
                    line: scanner.get_line(),
                    col: 1,
                }));
                lexeme.clear();
            }
            tokens.push(make_newline_token(scanner.get_line(), scanner.get_col()));
        }
    }

    Ok(())
}

fn read_schedule(scanner: &mut Scanner, tokens: &mut Vec<Token>) -> Result<()> {
    let mut lexeme = String::new();

    while let Ok(period_char) = scanner.peek_char() {
        if period_char == '*' {
            break;
        }

        scanner.read_char()?;
        match period_char {
            '\n' => {
                tokens.push(make_newline_token(scanner.get_line(), scanner.get_col()));
            }
            '0'..='9' => {
                // reading time
                let mut time_lexeme = String::new();
                time_lexeme.push(period_char);
                let col = scanner.get_col();

                while let Ok(time_char) = scanner.peek_char() {
                    if time_char == ' ' || time_char == '\n' {
                        break;
                    } else {
                        time_lexeme.push(time_char);
                        scanner.read_char()?;
                    }
                }

                tokens.push(make_time_token(time_lexeme, scanner.get_line(), col)?);
            }
            _ => {
                // reading identifier
                let col = scanner.get_col();
                lexeme.push(period_char);

                while let Ok(identifier_char) = scanner.peek_char() {
                    if identifier_char == ' ' || identifier_char == '\n' {
                        break;
                    } else {
                        lexeme.push(identifier_char);
                        scanner.read_char()?;
                    }
                }

                tokens.push(Token::Identifier(TokenInfo {
                    lexeme: lexeme.clone(),
                    literal: lexeme.clone(),
                    line: scanner.get_line(),
                    col,
                }));
            }
        }
    }

    Ok(())
}

fn tokenizer(filename: String) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();

    let mut scanner = Scanner::new(filename)?;
    // let file = File::open(filename)?;
    // let mut buffer = BufReader::new(file);
    // let mut line_number: usize = 0;

    while let Ok(c) = scanner.read_char() {
        match c {
            '*' => {
                tokens.push(Token::Asterisk(TokenInfo {
                    lexeme: "*".to_string(),
                    literal: "*".to_string(),
                    line: scanner.get_line(),
                    col: scanner.get_col(),
                }));
            }
            '\n' => {
                tokens.push(make_newline_token(scanner.get_line(), scanner.get_col()));
            }
            '"' => {
                // beginning of a string
                let col = scanner.get_col();
                let mut lexeme = String::new();
                lexeme.push(c);

                while let Ok(string_char) = scanner.read_char() {
                    lexeme.push(string_char);
                    if string_char == '"' {
                        break;
                    } else if string_char == '\n' {
                        return Err(anyhow!(
                            "Unterminated string on line {} near {}",
                            scanner.get_line(),
                            lexeme
                        ));
                    }
                }
                tokens.push(make_string_token(lexeme, scanner.get_line(), col)?);
            }
            '#' => {
                // comment
                while let Ok(comment_char) = scanner.peek_char() {
                    if comment_char == '\n' {
                        break;
                    }
                    scanner.read_char()?;
                }
            }
            '0'..='9' => {
                // date (or range)
                let mut lexeme = String::new();
                lexeme.push(c);
                let col = scanner.get_col();
                let mut is_range = false;

                while let Ok(date_char) = scanner.peek_char() {
                    if date_char == ' ' || date_char == '\n' {
                        break;
                    } else {
                        is_range = is_range || date_char == '-';
                        lexeme.push(date_char);
                        scanner.read_char()?;
                    }
                }

                if is_range {
                    let mut parts = lexeme.split('-');
                    let start = parts.next().unwrap().parse::<Date>()?;
                    let end = parts.next().unwrap().parse::<Date>()?;

                    tokens.push(Token::DateRange(TokenInfo {
                        lexeme,
                        literal: (start, end),
                        line: scanner.get_line(),
                        col,
                    }));
                } else {
                    let date = lexeme.parse::<Date>()?;

                    tokens.push(Token::Date(TokenInfo {
                        lexeme,
                        literal: date,
                        line: scanner.get_line(),
                        col,
                    }));
                }
            }
            'a'..='z' => {
                // identifier
                let col = scanner.get_col();
                let mut lexeme = String::new();
                lexeme.push(c);

                while let Ok(identifier_char) = scanner.peek_char() {
                    if identifier_char == ' ' || identifier_char == '\n' {
                        break;
                    } else {
                        lexeme.push(identifier_char);
                        scanner.read_char()?;
                    }
                }

                match lexeme.as_str() {
                    "repeat" => {
                        tokens.push(Token::Repeat(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            line: scanner.get_line(),
                            col,
                        }));
                    }
                    "calendar" => {
                        tokens.push(Token::Calendar(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            line: scanner.get_line(),
                            col,
                        }));
                    }
                    "periods" => {
                        match &tokens[tokens.len() - 1] {
                            Token::Asterisk(_) => {}
                            _ => {
                                return Err(anyhow!(
                                    "Line {}: periods must be preceded by an asterisk",
                                    scanner.get_line()
                                ));
                            }
                        }
                        tokens.push(Token::Periods(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            line: scanner.get_line(),
                            col,
                        }));

                        // read until next directive (*)
                        read_periods(&mut scanner, &mut tokens)?;
                    }
                    "non-periods" => {
                        match &tokens[tokens.len() - 1] {
                            Token::Asterisk(_) => {}
                            _ => {
                                return Err(anyhow!(
                                    "Line {}: non-periods must be preceded by an asterisk",
                                    scanner.get_line()
                                ));
                            }
                        }

                        tokens.push(Token::NonPeriods(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            line: scanner.get_line(),
                            col,
                        }));

                        // read until next directive (*)
                        read_periods(&mut scanner, &mut tokens)?;
                    }
                    "schedule" => {
                        match &tokens[tokens.len() - 1] {
                            Token::Asterisk(_) => {}
                            _ => {
                                return Err(anyhow!(
                                    "Line {}: schedule must be preceded by an asterisk",
                                    scanner.get_line()
                                ));
                            }
                        }

                        tokens.push(Token::Schedule(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            line: scanner.get_line(),
                            col,
                        }));

                        // read until next directive (*)
                        read_schedule(&mut scanner, &mut tokens)?;
                    }
                    _ => {
                        tokens.push(Token::Identifier(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            line: scanner.get_line(),
                            col,
                        }));
                    }
                }
            }
            ' ' => {}
            _ => {
                return Err(anyhow!(
                    "Line {}: unexpected character: {} ({})",
                    scanner.get_line(),
                    c,
                    c as u32
                ));
            }
        }
    }

    Ok(tokens)
}

fn main() {
    let result = tokenizer("data/mvhs/school_new.txt".to_string()).unwrap();
    // let result = tokenizer("testdata/school_new.txt".to_string()).unwrap();
    println!("{:#?}", result);
    // println!("{}", result);
}

#[test]
fn test_tokenizer() {
    let computed = tokenizer("testdata/schedule.txt".to_string()).unwrap();
    let actual = File::open("testdata/schedule_tokens.json").unwrap();
    let actual: Vec<Token> = serde_json::from_reader(actual).unwrap();

    assert_eq!(computed, actual);
}
