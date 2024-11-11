use anyhow::Result;

mod parser;
mod tokenizer;
mod utils;
mod validator;

use utils::Scanner;
use validator::SchoolInfo;

fn load_school(school: &str) -> Result<()> {
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

    let school_info = SchoolInfo {
        repeat,
        periods,
        non_periods,
        presets,
        calendar,
    };

    validator::high_level_verifier(school_info, &school_scanner, &schedule_scanner)?;

    Ok(())
}

fn main() {
    if let Err(err) = load_school("mvhs") {
        println!("{}", err);
    }
    // load_school("smhs");
    // load_school("lemanmiddle");
    // load_school("paly");

    // println!("Hello world");
}
