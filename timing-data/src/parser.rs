use std::collections::VecDeque;

use crate::utils::CompilerError;
use anyhow::{anyhow, Result};

use crate::tokenizer::{Literal, Token};

#[derive(Debug)]
pub struct Event {
    from: Token,
    name: Token,
}

#[derive(Debug)]
pub struct Preset {
    ident: Token,
    name: Token,
    event_list: Vec<Event>,
}

#[derive(Debug)]
pub enum Date {
    Range(Token, Token),
    Date(Token),
}

#[derive(Debug)]
pub struct CalendarItem {
    dates: Date,
    name_override: Option<Token>,
    preset: Token,
}

#[derive(Debug)]
pub struct Repeat {
    date: Token,
    pattern: Vec<Token>,
}

#[derive(Debug)]
pub enum Directive {
    Periods(Vec<Token>),
    NonPeriods(Vec<Token>),
    Preset(Preset),
    Calendar(Vec<CalendarItem>),
    Repeat(Repeat),
}

fn skip_newlines(tokens: &mut VecDeque<Token>) {
    while let Some(tok) = tokens.front() {
        if matches!(tok.literal, Literal::NewLine) {
            tokens.pop_front();
        } else {
            break;
        }
    }
}

fn err_from_tok(tok: &Token, msg: &str, hint: &str) -> CompilerError {
    CompilerError::new(
        tok.byte_idx,
        tok.byte_idx + tok.lexeme.len() as u32 - 1,
        msg,
        hint,
    )
}

fn next_tok(tokens: &mut VecDeque<Token>) -> Result<Token, CompilerError> {
    if tokens.len() == 0 {
        return Err(CompilerError::new(
            u32::MAX,
            u32::MAX,
            "unexpected end of file",
            "",
        ));
    }
    Ok(tokens.pop_front().unwrap())
}

fn expect_newline_after_directive_header(
    tokens: &mut VecDeque<Token>,
) -> Result<(), CompilerError> {
    let tok = next_tok(tokens)?;
    if !matches!(tok.literal, Literal::NewLine) {
        return Err(err_from_tok(
            &tok,
            "expected newline after directive header",
            "",
        ));
    }
    Ok(())
}

fn parse_repeat(tokens: &mut VecDeque<Token>) -> Result<Directive, CompilerError> {
    // expect date
    let date = next_tok(tokens)?;

    if !matches!(date.literal, Literal::Date(_)) {
        return Err(err_from_tok(
            &date,
            "expected date after repeat directive",
            "this represents the date to start the repeat pattern",
        ));
    }

    expect_newline_after_directive_header(tokens)?;

    let mut pattern = Vec::new();

    while let Some(tok) = tokens.front() {
        match tok.literal {
            Literal::NewLine => {
                tokens.pop_front();
            }
            Literal::Identifier(_) => {
                pattern.push(tokens.pop_front().unwrap());
            }
            _ => break,
        }
    }

    Ok(Directive::Repeat(Repeat { date, pattern }))
}

fn parse_periods(tokens: &mut VecDeque<Token>) -> Result<Directive, CompilerError> {
    expect_newline_after_directive_header(tokens)?;

    let mut periods = Vec::new();

    while let Some(tok) = tokens.front() {
        match tok.literal {
            Literal::NewLine => {
                tokens.pop_front();
            }
            Literal::PeriodName(_) => {
                periods.push(tokens.pop_front().unwrap());
            }
            _ => break,
        }
    }

    Ok(Directive::Periods(periods))
}

fn parse_nonperiods(tokens: &mut VecDeque<Token>) -> Result<Directive, CompilerError> {
    expect_newline_after_directive_header(tokens)?;

    let mut periods = Vec::new();

    while let Some(tok) = tokens.front() {
        match tok.literal {
            Literal::NewLine => {
                tokens.pop_front();
            }
            Literal::PeriodName(_) => {
                periods.push(tokens.pop_front().unwrap());
            }
            _ => break,
        }
    }

    Ok(Directive::NonPeriods(periods))
}

fn parse_schedule(tokens: &mut VecDeque<Token>) -> Result<Directive, CompilerError> {
    // expect identifier
    let ident = next_tok(tokens)?;

    if !matches!(ident.literal, Literal::Identifier(_)) {
        return Err(err_from_tok(
            &ident,
            "expected schedule id after schedule directive",
            "a schedule identifier must be a-z, -, and 0-9",
        ));
    }

    // expect name
    let name = next_tok(tokens)?;

    if !matches!(name.literal, Literal::String(_)) {
        return Err(err_from_tok(
            &name,
            "expected human-readable name after schedule id",
            "please enclose your string in quotes — ex: \"Schedule A\"",
        ));
    }

    // expect newline
    expect_newline_after_directive_header(tokens)?;

    // read events
    let mut event_list = Vec::new();

    while let Some(tok) = tokens.front() {
        match tok.literal {
            Literal::NewLine => {
                tokens.pop_front();
            }
            Literal::Time(_) => {
                let time_tok = tokens.pop_front().unwrap();
                let period_name_tok = next_tok(tokens)?;

                if !matches!(period_name_tok.literal, Literal::PeriodName(_)) {
                    return Err(err_from_tok(
                        &period_name_tok,
                        "expected period name after time",
                        "",
                    ));
                }

                event_list.push(Event {
                    from: time_tok,
                    name: period_name_tok,
                });
            }
            _ => break,
        }
    }

    Ok(Directive::Preset(Preset {
        name,
        ident,
        event_list,
    }))
}

fn parse_calendar(tokens: &mut VecDeque<Token>) -> Result<Directive, CompilerError> {
    expect_newline_after_directive_header(tokens)?;

    let mut calendar_items = Vec::new();

    while let Some(tok) = tokens.front() {
        match tok.literal {
            Literal::Date(_) => {
                let from = tokens.pop_front().unwrap();

                // is range?
                let tok = next_tok(tokens)?;

                let dates;

                if matches!(tok.literal, Literal::Hyphen) {
                    let to = next_tok(tokens)?;

                    if !matches!(to.literal, Literal::Date(_)) {
                        return Err(err_from_tok(
                            &to,
                            "expected another date after hyphen",
                            "looking for date range expression of form {date}-{date}",
                        ));
                    }

                    dates = Date::Range(from, to);
                } else {
                    dates = Date::Date(from);
                }

                // parse schedule id
                let preset = match dates {
                    Date::Range(_, _) => next_tok(tokens)?,
                    Date::Date(_) => tok,
                };

                if !matches!(preset.literal, Literal::Identifier(_)) {
                    return Err(err_from_tok(
                        &preset,
                        "expected schedule id (preset) after date",
                        "",
                    ));
                }

                // custom name?
                // can assume file ends in newline, so more tokens
                let name_override = if matches!(tokens.front().unwrap().literal, Literal::String(_))
                {
                    Some(tokens.pop_front().unwrap())
                } else {
                    None
                };

                calendar_items.push(CalendarItem {
                    dates,
                    name_override,
                    preset,
                });
            }
            Literal::NewLine => {
                tokens.pop_front();
            }
            _ => break,
        }
    }

    Ok(Directive::Calendar(calendar_items))
}

fn ast_gen(tokens: Vec<Token>) -> Result<Vec<Directive>, CompilerError> {
    let mut directives = Vec::new();
    let mut tokens = VecDeque::from(tokens);

    skip_newlines(&mut tokens);

    while tokens.len() > 0 {
        let tok = tokens.pop_front().unwrap();
        if !matches!(tok.literal, Literal::Asterisk) {
            return Err(err_from_tok(
                &tok,
                "unexpected token",
                "ensure this token is part of some directive, which start with \"*\"",
            ));
        }

        let tok = next_tok(&mut tokens)?;
        let dir = match tok.literal {
            Literal::Repeat => parse_repeat(&mut tokens)?,
            Literal::Calendar => parse_calendar(&mut tokens)?,
            Literal::Periods => parse_periods(&mut tokens)?,
            Literal::NonPeriods => parse_nonperiods(&mut tokens)?,
            Literal::Schedule => parse_schedule(&mut tokens)?,
            _ => {
                return Err(err_from_tok(
                    &tok,
                    "invalid directive name",
                    "directives are denoted by an asterisk followed by the directive name (periods, non-periods, schedule, repeat, calendar)",
                ));
            }
        };
        directives.push(dir);

        skip_newlines(&mut tokens);
    }

    Ok(directives)
}

pub fn gen(tokens: Vec<Token>, scanner: &crate::utils::Scanner) -> Result<Vec<Directive>> {
    let ast = ast_gen(tokens);

    if let Err(e) = ast {
        let mut err_string = String::new();
        e.fmt_for_terminal(&mut err_string, scanner)?;
        return Err(anyhow!(err_string));
    }

    Ok(ast?)
}
