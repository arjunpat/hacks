use anyhow::Result;
use serde::{Deserialize, Serialize};

mod parser;
mod serialize;
mod tokenizer;
mod utils;
mod validator;

use std::{collections::HashMap, fs, path::Path};
use utils::Scanner;
use validator::SchoolInfo;

struct SchoolJsonFiles {
    schedule: String,
    school: String,
}

fn process_school(
    mut schedule_scanner: Scanner,
    mut school_scanner: Scanner,
) -> Result<SchoolJsonFiles> {
    let schedule_tokens = tokenizer::make_tokens(&mut schedule_scanner)?;
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

fn load_schools_from_directory() -> Result<()> {
    let out_path = Path::new("json_output");
    if out_path.exists() {
        fs::remove_dir_all(out_path)?;
    }
    fs::create_dir(out_path)?;

    #[derive(Deserialize, Debug)]
    struct DirItem {
        name: String,
        folder: Option<String>,
    }

    let dir: HashMap<String, DirItem> =
        serde_json::from_reader(fs::File::open("data/directory.json")?)?;

    for entry in dir.keys() {
        let folder = match &dir[entry].folder {
            Some(folder) => folder,
            None => entry,
        };

        let schedule_scanner = Scanner::new(&format!("data/{}/schedule.txt", folder))?;
        let school_scanner = Scanner::new(&format!("data/{}/school.txt", folder))?;

        let result = process_school(schedule_scanner, school_scanner);

        match result {
            Ok(files) => {
                fs::create_dir(out_path.join(entry))?;
                fs::write(out_path.join(entry).join("schedule.json"), files.schedule)?;
                fs::write(out_path.join(entry).join("school.json"), files.school)?;
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    // write school_directory.json
    #[derive(Serialize)]
    struct SchoolDirectory {
        n: String,
        id: String,
    }

    let school_directory: Vec<SchoolDirectory> = dir
        .keys()
        .map(|entry| SchoolDirectory {
            n: dir[entry].name.clone(),
            id: entry.clone(),
        })
        .collect();

    fs::write(
        out_path.join("school_directory.json"),
        serde_json::to_string(&school_directory)?,
    )?;

    Ok(())
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    if let Err(e) = load_schools_from_directory() {
        println!("{:?}", e);
    }
}
