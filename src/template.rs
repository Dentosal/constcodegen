use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_PARAM: Regex = Regex::new(r"\$(\$|[a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
}

pub fn contains_parameter(text: &str, parameter: &str) -> bool {
    for cap in RE_PARAM.find_iter(text) {
        let name = cap.as_str();
        if name != "$$" && name == parameter {
            return true;
        }
    }
    false
}

pub fn replace_parameters(text: &str, context: &HashMap<&str, String>) -> String {
    let mut result = text.to_owned();
    for cap in RE_PARAM.find_iter(text) {
        let name = cap.as_str();
        if name == "$$" {
            continue;
        }
        if let Some(value) = context.get(name) {
            result = result.replace(name, value);
        } else {
            panic!("Unknown template parameter {:?}", name);
        }
    }
    result.replace("$$", "$")
}
