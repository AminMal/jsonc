use crate::constants::*;
use serde_json::Value;

#[inline]
pub fn format_member_definition<'a>(go_key: String, tpe: &str, json_key: &str) -> String {
    format!("\t{go_key}\t{tpe}\t\t`json:\"{json_key}\"`\n")
}

#[inline]
pub fn format_array_type(tpe: &str, type_prefix: &str) -> String {
    format!("[]{type_prefix}{tpe}")
}

pub fn get_primitive_type_name(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => BOOL,
        Value::Number(n) => {
            if n.is_f64() {
                FLOAT
            } else {
                INT
            }
        }
        Value::String(_) => STRING,
        Value::Null => ANY,
        // Non-primitives should not be passed to this function
        _ => ANY,
    }
}

pub fn first_char_upper(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(ch) => c.fold(ch.to_uppercase().to_string(), |mut buff, ch| {
            buff.push(ch);
            buff
        }),
    }
}

pub fn create_go_key(raw: &str) -> String {
    raw.split("_")
        .map(first_char_upper)
        .fold(String::new(), |mut buff, w| {
            buff.push_str(w.as_str());
            buff
        })
}
