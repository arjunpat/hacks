use std::collections::HashSet;

pub struct Validator {
    pub periods: HashSet<String>,
    pub non_periods: HashSet<String>,
}
