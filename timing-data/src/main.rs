use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
mod transformer;

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlDefaults {
    start: String,
    pattern: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]

pub struct YamlSchedule {
    pub defaults: YamlDefaults,
    pub calendar: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlSchoolSchedule {
    pub name: String,
    pub schedule: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlSchool {
    pub periods: Vec<String>,
    pub non_periods: Vec<String>,
    pub schedule_map: HashMap<String, YamlSchoolSchedule>,
}

fn yaml_value_to_school(mut value: serde_yaml::Value) -> Result<YamlSchool> {
    let value = value
        .as_mapping_mut()
        .ok_or(anyhow!("School not mapping"))?;

    // get periods
    let periods: Vec<String> = value
        .remove("periods")
        .ok_or(anyhow!("missing periods key"))?
        .as_sequence()
        .ok_or(anyhow!("periods not an arary"))?
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();

    let non_periods: Vec<String> = value
        .remove("non-periods")
        .ok_or(anyhow!("missing non-periods key"))?
        .as_sequence()
        .ok_or(anyhow!("non-periods not an arary"))?
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();

    let mut schedule_map: HashMap<String, YamlSchoolSchedule> = HashMap::new();

    for (k, v) in value {
        let k = k.as_str().unwrap().to_string();
        let v = v.as_mapping().ok_or(anyhow!("School not mapping"))?;

        let name = v
            .get("name")
            .ok_or(anyhow!("missing name key"))?
            .as_str()
            .ok_or(anyhow!("name not a string"))?
            .to_string();

        let schedule = v
            .get("schedule")
            .ok_or(anyhow!("missing schedule key"))?
            .as_sequence();

        let schedule = match schedule {
            Some(x) => x.iter().map(|x| x.as_str().unwrap().to_string()).collect(),
            None => vec![],
        };

        schedule_map.insert(k, YamlSchoolSchedule { name, schedule });
    }

    Ok(YamlSchool {
        periods,
        non_periods,
        schedule_map,
    })
}

fn load_school(school: String) -> Result<(), Error> {
    // file path to data/school/schedule.yml
    let schedule: YamlSchedule = serde_yaml::from_reader(BufReader::new(File::open(format!(
        "data/{}/schedule.yml",
        school
    ))?))?;

    let school: serde_yaml::Value = serde_yaml::from_reader(BufReader::new(File::open(format!(
        "data/{}/school.yml",
        school
    ))?))?;
    let school = yaml_value_to_school(school)?;

    // println!("{:#?}", schedule);
    // println!("{:#?}", school);

    Ok(())
}

fn main() {
    load_school("mvhs".to_string()).unwrap();
}
