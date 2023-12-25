use anyhow::{anyhow, Result};

mod parser;
mod tokenizer;
mod utils;

use utils::Scanner;

fn load_school(school: &str) -> Result<()> {
    let mut schedule_scanner = Scanner::new(&format!("data/{}/schedule.txt", school))?;
    let schedule_tokens = tokenizer::make_tokens(&mut schedule_scanner);

    let mut school_scanner = Scanner::new(&format!("data/{}/school.txt", school))?;
    let school_tokens = tokenizer::make_tokens(&mut school_scanner);

    if schedule_tokens.is_err() || school_tokens.is_err() {
        if let Err(err) = schedule_tokens {
            println!("{}", err);
        }

        if let Err(err) = school_tokens {
            println!("{}", err);
        }
        return Err(anyhow!("Tokenizer failed"));
    }

    // let mut tokens = schedule_tokens.unwrap();
    // tokens.append(&mut school_tokens.unwrap());

    let school_data = parser::gen(schedule_tokens.unwrap());
    let school_data = parser::gen(school_tokens.unwrap());

    // if let Err(e) = school_data {
    //     println!("{}", e);
    //     return Err(anyhow!("Failed to parse"));
    // }

    // if let Err(e) = parser::validate(school_data.unwrap()) {
    //     println!("{}", e);
    // }

    Ok(())
}

fn main() {
    load_school("mvhs");
    load_school("smhs");
    load_school("lemanmiddle");
    load_school("paly");

    // println!("Hello world");
}
