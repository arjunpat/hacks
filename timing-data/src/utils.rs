use anyhow::{anyhow, Result};
use core::{fmt, panic};
use std::{fs::File, io::Read};

pub struct Scanner {
    newline_pos: Vec<usize>,
    pos: usize,
    buffer: Vec<char>,
    filename: String,
}

impl Scanner {
    pub fn new(filename: &str) -> Result<Self> {
        let mut file = File::open(filename)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        // in case it is not already there
        buffer.push('\n');

        Ok(Scanner {
            newline_pos: Vec::new(),
            pos: 0,
            buffer: buffer.chars().collect(),
            filename: filename.to_string(),
        })
    }

    pub fn read_char(&mut self) -> Option<char> {
        let next_char = self.peek_char()?;
        if next_char == '\n' {
            self.newline_pos.push(self.pos);
        }
        self.pos += 1;
        Some(next_char)
    }

    // peek at the next character without consuming it
    pub fn peek_char(&mut self) -> Option<char> {
        if self.pos >= self.buffer.len() {
            return None;
        }

        Some(self.buffer[self.pos])
    }

    pub fn get_filename(&self) -> &str {
        &self.filename
    }

    pub fn get_pos(&self) -> u32 {
        if self.pos == 0 {
            panic!("Scanner::get_pos() called before reading any chars");
        }
        (self.pos - 1) as u32
    }

    pub fn get_line_col(&self, pos: usize) -> (usize, usize) {
        let mut line = 1;
        let col;

        while line <= self.newline_pos.len() && pos > self.newline_pos[line - 1] {
            line += 1;
        }

        if line > 1 {
            col = pos - self.newline_pos[line - 2];
        } else {
            col = pos + 1;
        }

        (line, col)
    }

    pub fn get_start_of_line(&self, line: usize) -> usize {
        match line {
            1 => 0,
            _ => self.newline_pos[line - 2] + 1,
        }
    }

    pub fn get_at(&self, pos: usize) -> Option<char> {
        if pos < self.buffer.len() {
            return Some(self.buffer[pos]);
        }
        None
    }

    pub fn len(&self) -> u32 {
        self.buffer.len() as u32
    }
}

#[derive(Debug)]
pub struct CompilerError {
    from: u32,
    to: u32,
    msg: String,
    hint: Option<String>,
}

impl std::error::Error for CompilerError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CompilerError: {}", self.msg)
    }
}

impl CompilerError {
    pub fn new(from: u32, to: u32, msg: &str, hint: &str) -> Self {
        let hint = if hint.len() != 0 {
            Some(hint.to_owned())
        } else {
            None
        };

        CompilerError {
            from,
            to,
            msg: msg.to_owned(),
            hint,
        }
    }

    pub fn get_err_str(&self, scanner: &Scanner) -> anyhow::Error {
        let mut err_str = String::new();
        self.fmt_for_terminal(&mut err_str, scanner).unwrap();
        anyhow!(err_str)
    }

    pub fn fmt_for_terminal(&self, f: &mut dyn fmt::Write, scanner: &Scanner) -> Result<()> {
        // assumes that self.from and self.to are on the same line
        // self.from and self.to can be u32::MAX if referring to the end of file
        let from = self.from.min(scanner.len());
        let to = self.to.min(scanner.len());
        let first_level_indent: String = " ".repeat(4);
        let excerpt_indent: String = " ".repeat(8);

        let (line, col) = scanner.get_line_col(from as usize);

        let mut section = String::new();
        {
            let mut idx = scanner.get_start_of_line(line) as usize;
            while let Some(c) = scanner.get_at(idx) {
                if c == '\n' {
                    break;
                }
                section.push(c);
                idx += 1;
            }
        }

        writeln!(f, "\x1b[1m\x1b[31mError\x1b[0m: \x1b[1m{}\x1b[0m", self.msg)?;
        writeln!(
            f,
            "{}in {}:{}:{}\n",
            first_level_indent,
            scanner.get_filename(),
            line,
            col
        )?;

        let line_padding = match line {
            line if line < 10 => "  ",
            line if line < 100 => " ",
            line if line < 1000 => "",
            _ => panic!("Too many lines in file"),
        };
        writeln!(
            f,
            "{}\x1b[1m\x1b[34m{}{}:\x1b[0m {}",
            excerpt_indent, line_padding, line, section
        )?;

        writeln!(
            f,
            "{}{}\x1b[1m\x1b[31m{}\x1b[0m\n",
            excerpt_indent,
            " ".repeat(col + 4),
            "^".repeat((to - from + 1) as usize)
        )?;
        if let Some(hint) = &self.hint {
            writeln!(f, "{}\x1b[1mhint:\x1b[0m {}", first_level_indent, hint)?;
        }
        Ok(())
    }
}
