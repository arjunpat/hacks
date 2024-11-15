use anyhow::Result;

mod parser;
mod serialize;
mod tokenizer;
mod utils;
mod validator;

use std::{fs, path::Path};
use utils::Scanner;
use validator::SchoolInfo;

struct SchoolJsonFiles {
    schedule: String,
    school: String,
}

fn load_school(school: &str) -> Result<SchoolJsonFiles> {
    let mut schedule_scanner = Scanner::new(&format!("data/{}/schedule.txt", school))?;
    let schedule_tokens = tokenizer::make_tokens(&mut schedule_scanner)?;

    let mut school_scanner = Scanner::new(&format!("data/{}/school.txt", school))?;
    let school_tokens = tokenizer::make_tokens(&mut school_scanner)?;

    let schedule_ast = parser::gen(schedule_tokens, &schedule_scanner)?;
    let school_ast = parser::gen(school_tokens, &school_scanner)?;

    // make school info
    let (periods, non_periods, presets) =
        validator::school_to_school_info(school_ast, &school_scanner)?;

    let (repeat, calendar) = validator::schedule_to_school_info(schedule_ast, &schedule_scanner)?;

    let mut school_info = SchoolInfo {
        repeat,
        periods,
        non_periods,
        presets,
        calendar,
    };

    validator::high_level_verifier(&school_info, &school_scanner, &schedule_scanner)?;

    validator::prune(&mut school_info);

    let schedule_json = serialize::serialize_to_schedule(&school_info)?;
    let school_json = serialize::serialize_to_school(&school_info)?;

    Ok(SchoolJsonFiles {
        schedule: schedule_json,
        school: school_json,
    })
}

fn main() {
    let out_path = Path::new("json_output");
    if out_path.exists() {
        fs::remove_dir_all(out_path).unwrap();
    }
    fs::create_dir(out_path).unwrap();

    for entry in fs::read_dir(Path::new("data")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let school = path.file_name().unwrap().to_str().unwrap();
            let result = load_school(school);

            match result {
                Ok(files) => {
                    fs::create_dir(out_path.join(school)).unwrap();
                    fs::write(out_path.join(school).join("schedule.json"), files.schedule).unwrap();
                    fs::write(out_path.join(school).join("school.json"), files.school).unwrap();
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    }
}
