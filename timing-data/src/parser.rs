use std::{any::Any, collections::VecDeque};

use crate::utils::CompilerError;
use anyhow::{anyhow, Result};

use crate::tokenizer::{self, TokenInfo};

pub struct Event {
    from: TokenInfo<tokenizer::Time>,
    name: TokenInfo<String>,
}

pub struct Preset {
    name: TokenInfo<String>,
    event_list: Vec<Event>,
}

pub enum Date {
    Range(TokenInfo<tokenizer::Date>, TokenInfo<tokenizer::Date>),
    Date(TokenInfo<tokenizer::Date>),
}

pub struct CalendarItem {
    dates: Date,
    name_override: Option<TokenInfo<String>>,
    preset: TokenInfo<String>,
}

pub struct Repeat {
    date: TokenInfo<tokenizer::Date>,
    pattern: Vec<TokenInfo<String>>,
}

pub enum Directive {
    periods(Vec<TokenInfo<String>>),
    non_periods(Vec<TokenInfo<String>>),
    presets(Vec<(TokenInfo<String>, Preset)>),
    calendar(Vec<CalendarItem>),
    repeat(Repeat),
}

fn skip_newlines(tokens: &mut VecDeque<tokenizer::Token>) {
    while let Some(tok) = tokens.front() {
        if matches!(tokenizer::Token::NewLine, tok) {
            tokens.pop_front();
        }
    }
}

pub fn gen(tokens: Vec<tokenizer::Token>) -> Result<Vec<Directive>> {
    let mut directives = Vec::new();
    let mut tokens = VecDeque::from(tokens);

    skip_newlines(&mut tokens);

    // if let Some(tok) = tokens.front() {
    //     if !matches!(tokenizer::Token::Asterisk, tok) {
    //         return Err(anyhow!(CompilerError::new(tok., to, msg, hint)));
    //     }
    // }

    Ok(directives)
}

// pub fn validate(school_data: SchoolData) -> Result<()> {
//     // check for overlapping overdetermined dates (calendar)
//     Ok(())
// }
