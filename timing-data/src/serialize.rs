use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{parser::Date, tokenizer::Literal, validator::SchoolInfo};

#[derive(Debug, Serialize, Deserialize)]
struct ScheduleJson {
    defaults: Defaults,
    calendar: Vec<CalendarItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Defaults {
    start: String,
    pattern: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CalendarItemContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<String>,
    t: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CalendarItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    content: CalendarItemContent,
}

#[derive(Debug, Serialize, Deserialize)]
struct SchoolJson {
    periods: Vec<String>,
    #[serde(rename = "nonPeriods")]
    non_periods: Vec<String>,
    presets: HashMap<String, Preset>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Preset {
    n: String,
    s: Vec<PeriodItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PeriodItem {
    f: String,
    n: String,
}

pub fn serialize_to_school(school_info: &SchoolInfo) -> Result<String, serde_json::Error> {
    let mut school_json = SchoolJson {
        periods: school_info
            .periods
            .1
            .iter()
            .map(|t| t.lexeme.clone())
            .collect(),
        non_periods: school_info
            .non_periods
            .1
            .iter()
            .map(|t| t.lexeme.clone())
            .collect(),
        presets: HashMap::new(),
    };

    for (name, preset) in &school_info.presets {
        let n = match &preset.name.literal {
            Literal::String(n) => n.to_string(),
            _ => unreachable!(),
        };
        school_json.presets.insert(
            name.clone(),
            Preset {
                n,
                s: preset
                    .event_list
                    .iter()
                    .map(|e| PeriodItem {
                        f: e.from.lexeme.clone(),
                        n: e.name.lexeme.clone(),
                    })
                    .collect(),
            },
        );
    }

    serde_json::to_string_pretty(&school_json)
}

pub fn serialize_to_schedule(school_info: &SchoolInfo) -> Result<String, serde_json::Error> {
    let mut schedule_json = ScheduleJson {
        defaults: Defaults {
            start: "".to_string(),
            pattern: vec![],
        },
        calendar: vec![],
    };

    if let Literal::Date(date) = school_info.repeat.1.date.literal {
        schedule_json.defaults.start = date.to_american();
    } else {
        unreachable!();
    }

    for item in &school_info.repeat.1.pattern {
        schedule_json.defaults.pattern.push(item.lexeme.clone());
    }

    for item in &school_info.calendar {
        let n = match &item.name_override {
            Some(n) => match &n.literal {
                Literal::String(n) => Some(n.to_string()),
                _ => unreachable!(),
            },
            None => None
        };

        let mut calendar_item = CalendarItem {
            from: None,
            to: None,
            date: None,
            content: CalendarItemContent {
                n,
                t: item.preset.lexeme.clone(),
            },
        };

        match &item.dates {
            Date::Date(date) => {
                if let Literal::Date(date) = date.literal {
                    calendar_item.date = Some(date.to_american());
                } else {
                    unreachable!();
                }
            }
            Date::Range(from, to) => {
                if let Literal::Date(from) = from.literal {
                    calendar_item.from = Some(from.to_american());
                } else {
                    unreachable!();
                }

                if let Literal::Date(to) = to.literal {
                    calendar_item.to = Some(to.to_american());
                } else {
                    unreachable!();
                }
            }
        }

        schedule_json.calendar.push(calendar_item);
    }

    serde_json::to_string_pretty(&schedule_json)
}
