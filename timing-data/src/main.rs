use anyhow::{anyhow, Error, Result};
use core::fmt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TokenInfo<LiteralType> {
    lexeme: String,       // the actual text of the lexeme
    literal: LiteralType, // the value of the lexeme
    byte_idx: usize,      // the byte index of the lexeme
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

#[derive(Debug)]
struct LexerError {
    from: u32,
    to: u32,
    msg: String,
}

impl std::error::Error for LexerError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LexError: {}", self.msg)
    }
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
    PeriodName(TokenInfo<String>),
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
    newline_pos: Vec<usize>,
    pos: usize,
    reader: BufReader<File>,
    eof: bool,
}

impl Scanner {
    pub fn new(filename: &str) -> Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        Ok(Scanner {
            newline_pos: Vec::new(),
            pos: 0,
            reader,
            eof: false,
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

        if one_char[0] == '\n' as u8 {
            self.newline_pos.push(self.pos);
        }

        self.pos += 1;

        Ok(one_char[0] as char)
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

    pub fn get_pos(&self) -> usize {
        self.pos
    }

    pub fn get_line_col(&self, pos: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for newline_pos in &self.newline_pos {
            if pos <= *newline_pos {
                break;
            }
            line += 1;
        }

        if line > 1 {
            col = pos - self.newline_pos[line - 2];
        } else {
            col = pos;
        }

        (line, col)
    }

    /// Returns the surrounding lines byte positions. Usedful for printing errors.
    /// Return values:
    /// return.0 = the byte pos of the start of the line before pos; if none exists, returns 0
    /// return.1 = the byte pos of the start of the line after pos; if none exists,
    /// returns the byte position of the end of the file
    pub fn get_surrounding_lines(&self, pos: usize) -> (usize, usize) {
        let (line, _) = self.get_line_col(pos);
        let before;
        let after;

        if line == 1 {
            before = 0;
        } else {
            before = self.newline_pos[line - 2];
        }

        if line == self.newline_pos.len() + 1 {
            after = self.newline_pos[line - 2];
        } else {
            after = self.newline_pos[line - 1];
        }

        (before, after)
    }
}

fn make_string_token(lexeme: String, byte_idx: usize) -> Result<Token> {
    let literal = lexeme[1..lexeme.len() - 1].to_string();
    let re: Regex = Regex::new(r#"^[a-zA-Z0-9 \-'()]+$"#).unwrap();

    if !re.is_match(&literal) {
        return Err(anyhow!(
            "Line {}: Invalid string on  near {}",
            byte_idx,
            lexeme
        ));
    }

    Ok(Token::String(TokenInfo {
        lexeme,
        literal,
        byte_idx,
    }))
}

fn make_newline_token(byte_idx: usize) -> Token {
    Token::NewLine(TokenInfo {
        lexeme: "\n".to_string(),
        literal: "\n".to_string(),
        byte_idx,
    })
}

fn make_time_token(lexeme: String, byte_idx: usize) -> Result<Token> {
    let mut parts = lexeme.split(':');
    let hour = parts.next().unwrap().parse::<u32>()?;
    let minute = parts.next().unwrap().parse::<u32>()?;

    Ok(Token::Time(TokenInfo {
        lexeme,
        literal: Time { hour, minute },
        byte_idx,
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
                tokens.push(Token::PeriodName(TokenInfo {
                    lexeme: lexeme.clone(),
                    literal: lexeme.clone(),
                    byte_idx: scanner.get_pos(),
                }));
                lexeme.clear();
            }
            tokens.push(make_newline_token(scanner.get_pos()));
        } else {
            lexeme.push(period_char);
        }
    }

    Ok(())
}

fn tokenizer(scanner: &mut Scanner) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();

    // let file = File::open(filename)?;
    // let mut buffer = BufReader::new(file);
    // let mut line_number: usize = 0;

    while let Ok(c) = scanner.read_char() {
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

                while let Ok(string_char) = scanner.read_char() {
                    lexeme.push(string_char);
                    if string_char == '"' {
                        break;
                    } else if string_char == '\n' {
                        return Err(anyhow!(LexerError {
                            from: byte_pos as u32,
                            to: (scanner.get_pos() as u32) - 1,
                            msg: "Unterminated string".to_string(),
                        }));
                        // return Err(anyhow!(
                        //     "Unterminated string on line {} near {}",
                        //     scanner.get_line_col(col).0,
                        //     lexeme
                        // ));
                    }
                }
                tokens.push(make_string_token(lexeme, byte_pos)?);
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
                // date (or range) or time
                let mut lexeme = String::new();
                lexeme.push(c);
                let byte_idx = scanner.get_pos();
                let mut is_range = false;
                let mut is_time = false;

                while let Ok(date_char) = scanner.peek_char() {
                    if date_char == ' ' || date_char == '\n' {
                        break;
                    } else {
                        is_range = is_range || date_char == '-';
                        is_time = is_time || date_char == ':';
                        lexeme.push(date_char);
                        scanner.read_char()?;
                    }
                }

                if is_range && is_time {
                    return Err(anyhow!(
                        "Line {}: unexpected token {}",
                        scanner.get_pos(),
                        lexeme
                    ));
                }

                if is_time {
                    tokens.push(make_time_token(lexeme, byte_idx)?);

                    // assert space after time
                    if let Ok(space_char) = scanner.read_char() {
                        if space_char != ' ' {
                            return Err(anyhow!(
                                "Line {}: unexpected token {}",
                                scanner.get_pos(),
                                space_char
                            ));
                        }
                    } else {
                        return Err(anyhow!(
                            "Line {}: unexpected end of file",
                            scanner.get_pos()
                        ));
                    }

                    // parse PeriodName after time
                    let byte_idx = scanner.get_pos();
                    let mut pn_lexeme = String::new();
                    while let Ok(pn_char) = scanner.peek_char() {
                        if pn_char == '\n' {
                            break;
                        } else {
                            pn_lexeme.push(pn_char);
                            scanner.read_char()?;
                        }
                    }

                    tokens.push(Token::PeriodName(TokenInfo {
                        lexeme: pn_lexeme.clone(),
                        literal: pn_lexeme.clone(),
                        byte_idx: byte_idx + 1,
                    }));
                } else if is_range {
                    let mut parts = lexeme.split('-');
                    let start = parts.next().unwrap().parse::<Date>()?;
                    let end = parts.next().unwrap().parse::<Date>()?;

                    tokens.push(Token::DateRange(TokenInfo {
                        lexeme,
                        literal: (start, end),
                        byte_idx,
                    }));
                } else {
                    let date = lexeme.parse::<Date>()?;

                    tokens.push(Token::Date(TokenInfo {
                        lexeme,
                        literal: date,
                        byte_idx,
                    }));
                }
            }
            'a'..='z' => {
                // identifier
                let byte_idx = scanner.get_pos();
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
                                return Err(anyhow!(
                                    "Line {}: periods must be preceded by an asterisk",
                                    scanner.get_pos()
                                ));
                            }
                        }
                        tokens.push(Token::Periods(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            byte_idx,
                        }));

                        // read until next directive (*)
                        read_periods(scanner, &mut tokens)?;
                    }
                    "non-periods" => {
                        match &tokens[tokens.len() - 1] {
                            Token::Asterisk(_) => {}
                            _ => {
                                return Err(anyhow!(
                                    "Line {}: non-periods must be preceded by an asterisk",
                                    scanner.get_pos()
                                ));
                            }
                        }

                        tokens.push(Token::NonPeriods(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            byte_idx,
                        }));

                        // read until next directive (*)
                        read_periods(scanner, &mut tokens)?;
                    }
                    "schedule" => {
                        match &tokens[tokens.len() - 1] {
                            Token::Asterisk(_) => {}
                            _ => {
                                return Err(anyhow!(
                                    "Line {}: schedule must be preceded by an asterisk",
                                    scanner.get_pos()
                                ));
                            }
                        }

                        tokens.push(Token::Schedule(TokenInfo {
                            lexeme: lexeme.clone(),
                            literal: lexeme.clone(),
                            byte_idx,
                        }));

                        // read until next directive (*)
                        // read_schedule(&mut scanner, &mut tokens)?;
                    }
                    _ => {
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
                return Err(anyhow!(
                    "Line {}: unexpected character: {} ({})",
                    scanner.get_pos(),
                    c,
                    c as u32
                ));
            }
        }
    }

    Ok(tokens)
}

impl LexerError {
    fn print(&self, scanner: &Scanner, filename: &str) -> Result<()> {
        let buffer = File::open(filename)?;
        let mut buffer = BufReader::new(buffer);

        let mut section = String::new();

        let (before, after) = scanner.get_surrounding_lines(self.from as usize);
        buffer.seek_relative(before as i64)?;
        buffer
            .take((after - before) as u64)
            .read_to_string(&mut section)?;

        let (line, col) = scanner.get_line_col(self.from as usize);
        println!("Error on line {}: {}\n", line, self.msg);

        let mut i = before;
        for sec in section.split('\n') {
            let line = scanner.get_line_col(i);
            // pad line number to be 3 digits by adding a space at the beginning if needed
            let line_str = format!("{}{}", if line.0 < 100 { " " } else { "" }, line.0);

            println!("        {}: {}", line_str, sec);
            i += sec.len() + 1;
        }

        println!(
            "{}{}",
            " ".repeat(col + 12),
            "^".repeat((self.to - self.from) as usize)
        );

        // let (line, col) = scanner.get_line_col(self.from as usize);
        // let (line1, col1) = scanner.get_line_col(self.to as usize);
        // println!(
        //     "Line {}, Col {} to Line {} Col {}: {}",
        //     line, col, line1, col1, self.msg
        // );
        Ok(())
    }
}

fn main() {
    let filename = "data/mvhs/schedule_new.txt";
    let mut scanner = Scanner::new(filename).unwrap();
    // let result = tokenizer("data/mvhs/school_new.txt".to_string()).unwrap();
    let result = tokenizer(&mut scanner);

    match result {
        Ok(tokens) => {
            println!("{:#?}", tokens);
        }
        Err(e) => match e.downcast::<LexerError>() {
            Ok(downcast) => {
                downcast.print(&scanner, filename).unwrap();
                // let (line, col) = scanner.get_line_col(downcast.from as usize);
                // println!("Line {}, Col {}: {}", line, col, downcast.msg);
            }
            Err(e) => {
                println!("{}", e);
            }
        },
    }

    // println!("{}", result);
}

// #[test]
// fn test_tokenizer() {
//     let computed = tokenizer("testdata/schedule.txt".to_string()).unwrap();
//     let actual = File::open("testdata/schedule_tokens.json").unwrap();
//     let actual: Vec<Token> = serde_json::from_reader(actual).unwrap();

//     assert_eq!(computed, actual);
// }
