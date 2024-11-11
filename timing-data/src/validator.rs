use std::{collections::HashMap, vec};

use anyhow::Result;
use regex::Regex;

use crate::{
    parser::{CalendarItem, Directive, DirectiveItem, Preset, Repeat},
    tokenizer::{Literal, Time, Token},
    utils::{CompilerError, Scanner},
};

#[derive(Debug)]
pub struct SchoolInfo {
    pub repeat: (Token, Repeat),
    pub periods: (Token, Vec<Token>),
    pub non_periods: (Token, Vec<Token>),
    pub presets: HashMap<String, (Token, Preset)>,
    pub calendar: Vec<CalendarItem>,
}

fn err_from_tok(tok: &Token, scanner: &Scanner, msg: &str, hint: &str) -> anyhow::Error {
    CompilerError::new(
        tok.byte_idx,
        tok.byte_idx + tok.lexeme.len() as u32 - 1,
        msg,
        hint,
    )
    .get_err_str(scanner)
}

pub fn schedule_to_school_info(
    mut ast: Vec<Directive>,
    scanner: &Scanner,
) -> Result<((Token, Repeat), Vec<CalendarItem>)> {
    let mut repeat = None;
    let mut calendar = None;

    while let Some(directive) = ast.pop() {
        match directive.content {
            DirectiveItem::Repeat(r) => {
                if repeat.is_some() {
                    return Err(err_from_tok(
                        &directive.begin_tok,
                        scanner,
                        "found more than one repeat directive",
                        "",
                    ));
                }

                repeat = Some((directive.begin_tok, r));
            }
            DirectiveItem::Calendar(cal) => {
                if calendar.is_some() {
                    return Err(err_from_tok(
                        &directive.begin_tok,
                        scanner,
                        "found more than one calendar directive",
                        "",
                    ));
                }

                calendar = Some(cal);
            }
            _ => {
                return Err(err_from_tok(
                    &directive.begin_tok,
                    scanner,
                    "found invalid directive in schedule",
                    "expected repeat or calendar",
                ));
            }
        }
    }

    if repeat.is_none() {
        let err = CompilerError::new(
            scanner.len() - 1,
            scanner.len() - 1,
            "found no repeat directive in schedule",
            "",
        );
        return Err(err.get_err_str(scanner));
    }

    if calendar.is_none() {
        let err = CompilerError::new(
            scanner.len() - 1,
            scanner.len() - 1,
            "found no calendar directive in schedule",
            "",
        );
        return Err(err.get_err_str(scanner));
    }

    Ok((repeat.unwrap(), calendar.unwrap()))
}

pub fn school_to_school_info(
    mut ast: Vec<Directive>,
    scanner: &Scanner,
) -> Result<(
    (Token, Vec<Token>),
    (Token, Vec<Token>),
    HashMap<String, (Token, Preset)>,
)> {
    let mut periods = None;
    let mut non_periods = None;
    let mut presets = HashMap::new();

    while let Some(directive) = ast.pop() {
        match directive.content {
            DirectiveItem::Periods(periods_vec) => {
                if periods.is_some() {
                    return Err(err_from_tok(
                        &directive.begin_tok,
                        scanner,
                        "found more than one periods directive",
                        "",
                    ));
                }

                periods = Some((directive.begin_tok, periods_vec));
            }
            DirectiveItem::NonPeriods(nonperiods_vec) => {
                if non_periods.is_some() {
                    return Err(err_from_tok(
                        &directive.begin_tok,
                        scanner,
                        "found more than one non-periods directive",
                        "",
                    ));
                }

                non_periods = Some((directive.begin_tok, nonperiods_vec));
            }
            DirectiveItem::Preset(preset) => {
                presets.insert(preset.ident.lexeme.clone(), (preset.ident.clone(), preset));
            }
            _ => {
                return Err(err_from_tok(
                    &directive.begin_tok,
                    scanner,
                    "found invalid directive in school file",
                    "expected periods, non-periods, or preset",
                ));
            }
        }
    }

    if periods.is_none() {
        let err = CompilerError::new(
            scanner.len() - 1,
            scanner.len() - 1,
            "found no periods directive in school file",
            "",
        );
        return Err(err.get_err_str(scanner));
    }

    if non_periods.is_none() {
        let err = CompilerError::new(
            scanner.len() - 1,
            scanner.len() - 1,
            "found no non-periods directive in school file",
            "",
        );
        return Err(err.get_err_str(scanner));
    }

    if presets.len() == 0 {
        let err = CompilerError::new(
            scanner.len() - 1,
            scanner.len() - 1,
            "found no presets in school file",
            "",
        );
        return Err(err.get_err_str(scanner));
    }

    Ok((periods.unwrap(), non_periods.unwrap(), presets))
}

const DEFAULT_NON_PERIODS: [&str; 5] = ["Free", "Brunch", "Break", "Lunch", "Passing"];

pub fn high_level_verifier(
    school_info: SchoolInfo,
    school_scanner: &Scanner,
    schedule_scanner: &Scanner,
) -> Result<()> {
    // ensure no overlap between periods and non-periods

    let mut periods = vec![];
    let mut non_periods = vec![];
    let event_regex = Regex::new(r"^[a-zA-Z0-9\s:&/]+$").unwrap();

    for period in school_info.periods.1.iter() {
        if DEFAULT_NON_PERIODS.contains(&period.lexeme.as_str()) {
            return Err(err_from_tok(
                &period,
                school_scanner,
                &format!("found non-period `{}` in periods", period.lexeme),
                &format!(
                    "`{}` is, by default, a non-period; please choose a different name",
                    period.lexeme
                ),
            ));
        }
        if !event_regex.is_match(&period.lexeme) {
            return Err(err_from_tok(
                &period,
                school_scanner,
                &format!(
                    "invalid period name `{}`; contains invalid characters",
                    period.lexeme
                ),
                &format!(
                    "period names must only contain letters, numbers, spaces, colons, and slashes"
                ),
            ));
        }
        periods.push(period.lexeme.clone());
    }

    for non_period in school_info.non_periods.1.iter() {
        if periods.contains(&non_period.lexeme) {
            return Err(err_from_tok(
                &non_period,
                school_scanner,
                &format!("found `{}` in non-periods", non_period.lexeme),
                &format!(
                    "`{}` cannot be both a period and a non-period",
                    non_period.lexeme
                ),
            ));
        }

        if DEFAULT_NON_PERIODS.contains(&non_period.lexeme.as_str()) {
            return Err(err_from_tok(
                &non_period,
                school_scanner,
                &format!(
                    "found default non-period `{}` in non-periods",
                    non_period.lexeme
                ),
                &format!(
                    "`{}` is, by default, a non-period, so it can be removed from non-periods",
                    non_period.lexeme
                ),
            ));
        }

        if !event_regex.is_match(&non_period.lexeme) {
            return Err(err_from_tok(
                &non_period,
                school_scanner,
                &format!("invalid non-period name `{}`; contains invalid characters", non_period.lexeme),
                &format!(
                    "non-period names must only contain letters, numbers, spaces, colons, and slashes"
                ),
            ));
        }

        non_periods.push(non_period.lexeme.clone());
    }

    // vector all_events = periods + non_periods
    let mut all_events: Vec<String> = periods.clone();
    all_events.extend(non_periods.clone());
    all_events.extend(DEFAULT_NON_PERIODS.iter().map(|s| s.to_string()));

    // ensure valid presets
    for (ident, (tok, preset)) in school_info.presets.iter() {
        let mut last_time: Option<Time> = None;
        // verify high-level schedule
        for i in 0..preset.event_list.len() {
            let event = &preset.event_list[i];
            if !all_events.contains(&event.name.lexeme) {
                return Err(err_from_tok(
                    &event.name,
                    school_scanner,
                    &format!(
                        "found invalid event name `{}` in preset `{}`",
                        event.name.lexeme, ident
                    ),
                    &format!(
                        "event names must be one of the following: {}",
                        all_events.join(", ")
                    ),
                ));
            }

            // ensure time orders are increasing and 24-hour time
            if let Literal::Time(time) = &event.from.literal {
                if last_time.is_some() {
                    if time.hour < last_time.unwrap().hour
                        || (time.hour == last_time.unwrap().hour
                            && time.minute < last_time.unwrap().minute)
                    {
                        return Err(err_from_tok(
                            &event.from,
                            school_scanner,
                            "time order in a preset must be increasing",
                            "are you using 24-hour time in hh:mm format?",
                        ));
                    }
                }
                last_time = Some(time.clone());
            } else {
                unreachable!();
            }

            // ensure each presets ends with a "Free"
            if i == preset.event_list.len() - 1 {
                if event.name.lexeme != "Free" {
                    return Err(err_from_tok(
                        &event.name,
                        school_scanner,
                        &format!("last event in preset `{}` must be `Free`", ident),
                        "all presets must end with `Free`",
                    ));
                }
            }
        }
    }

    let preset_names: Vec<String> = school_info.presets.keys().cloned().collect();

    // validate calendar array
    for item in school_info.calendar.iter() {
        if !preset_names.contains(&item.preset.lexeme) {
            return Err(err_from_tok(
                &item.preset,
                schedule_scanner,
                &format!(
                    "found invalid preset name `{}` in calendar",
                    item.preset.lexeme
                ),
                &format!(
                    "preset names must be one of the following: {}",
                    preset_names.join(", ")
                ),
            ));
        }
    }

    // assert repeats are in the presets
    for repeat in school_info.repeat.1.pattern.iter() {
        if !preset_names.contains(&repeat.lexeme) {
            return Err(err_from_tok(
                &repeat,
                schedule_scanner,
                &format!("found invalid preset name `{}` in repeat", repeat.lexeme),
                &format!(
                    "preset names must be one of the following: {}",
                    preset_names.join(", ")
                ),
            ));
        }
    }

    return Ok(());
}
