use crate::constants::*;
use serde_json::Value;
use std::rc::Rc;

pub trait LanguageFormatter {
    fn struct_or_class_header(&self, raw: String) -> String;
    // It's usually a '}' or ')'
    fn struct_or_class_footer(&self, struct_name: Option<String>) -> String;

    fn field_name(&self, json_key: &str) -> String;

    fn format_field_type(&self, tpe: &str, json_key: &str) -> String;

    fn format_arr_type(&self, arr_type: String, optional: bool) -> String;

    fn premitive_type_name(&self, from: &Value) -> &'static str;

    fn struct_or_class_name(&self, key: &str) -> String;

    fn struct_name_from_array_key(&self, arr_key: &str) -> String {
        if let Some(stripped) = arr_key.strip_suffix("ies") {
            format!("{}y", self.field_name(stripped))
        } else if let Some(stripped) = arr_key.strip_suffix("s") {
            self.struct_or_class_name(stripped)
        } else {
            self.struct_or_class_name(arr_key)
        }
    }
}

fn first_char_upper(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(ch) => c.fold(ch.to_uppercase().to_string(), |mut buff, ch| {
            buff.push(ch);
            buff
        }),
    }
}

pub fn get_language_formatter(lang: &str) -> Option<Rc<dyn LanguageFormatter>> {
    match lang.to_lowercase().as_str() {
        "go" => Some(Rc::new(Go {})),
        "scala" => Some(Rc::new(Scala {})),
        "java" => Some(Rc::new(Java {})),
        "rust" => Some(Rc::new(Rust {})),
        _ => None,
    }
}

pub struct Rust {}
pub struct Scala {}
pub struct Go {}
pub struct Java {}

impl LanguageFormatter for Rust {
    fn struct_or_class_header(&self, raw: String) -> String {
        let rust_struct_name = self.struct_or_class_name(&raw);
        format!("pub struct {rust_struct_name} ") + "{\n"
    }

    fn struct_or_class_footer(&self, _struct_name: Option<String>) -> String {
        String::from("}")
    }

    fn field_name(&self, json_key: &str) -> String {
        String::from(json_key)
    }

    fn format_field_type(&self, tpe: &str, json_key: &str) -> String {
        format!("\tpub {json_key}: {tpe},\n")
    }

    fn format_arr_type(&self, arr_type: String, optional: bool) -> String {
        let tpe = if optional {
            format!("Option<{arr_type}>")
        } else {
            arr_type
        };
        format!("Vec<{tpe}>")
    }

    fn premitive_type_name(&self, from: &Value) -> &'static str {
        match from {
            Value::Bool(_) => RUST_BOOL,
            Value::Number(n) => {
                if n.is_f64() {
                    RUST_FLOAT
                } else {
                    RUST_INT
                }
            }
            Value::String(_) => RUST_STRING,
            Value::Null => RUST_ANY,
            // Non-primitives should not be passed to this function
            _ => RUST_ANY,
        }
    }

    fn struct_or_class_name(&self, key: &str) -> String {
        key.split('_')
            .map(first_char_upper)
            .fold(String::new(), |mut acc, w| {
                acc.push_str(&w);
                acc
            })
    }
}

impl LanguageFormatter for Scala {
    fn struct_or_class_header(&self, raw: String) -> String {
        let class_name = self.struct_or_class_name(&raw);

        format!("case class {class_name}(\n")
    }

    fn struct_or_class_footer(&self, struct_name: Option<String>) -> String {
        let header_len = self
            .struct_or_class_header(struct_name.unwrap_or(SCALA_AUTO_GENERATED.to_string()))
            .len();
        let tabs = header_len / 8;
        let mut padding = (1..=tabs).fold(String::new(), |mut acc, _| {
            acc.push('\t');
            acc
        });
        padding.push(')');
        padding
    }

    fn field_name(&self, json_key: &str) -> String {
        camelcase(json_key)
    }

    fn format_field_type(&self, tpe: &str, json_key: &str) -> String {
        let scala_field_name = self.field_name(json_key);
        format!("\t\t{scala_field_name}: {tpe},\n")
    }

    fn format_arr_type(&self, arr_type: String, optional: bool) -> String {
        let tpe = if optional {
            format!("Option[{arr_type}]")
        } else {
            arr_type
        };
        format!("Seq[{tpe}]")
    }

    fn premitive_type_name(&self, from: &Value) -> &'static str {
        match from {
            Value::Bool(_) => SCALA_BOOL,
            Value::Number(n) => {
                if n.is_f64() {
                    SCALA_FLOAT
                } else {
                    SCALA_INT
                }
            }
            Value::String(_) => SCALA_STRING,
            Value::Null => SCALA_ANY,
            // Non-primitives should not be passed to this function
            _ => SCALA_ANY,
        }
    }

    fn struct_or_class_name(&self, key: &str) -> String {
        key.split('_')
            .map(first_char_upper)
            .fold(String::new(), |mut acc, word| {
                acc.push_str(word.as_str());
                acc
            })
    }
}

impl LanguageFormatter for Go {
    fn struct_or_class_header(&self, raw: String) -> String {
        let go_struct_name = self.field_name(&raw);
        format!("type {go_struct_name} struct") + " {\n"
    }

    fn struct_or_class_footer(&self, _struct_name: Option<String>) -> String {
        String::from("}")
    }

    fn field_name(&self, json_key: &str) -> String {
        json_key
            .split('_')
            .map(first_char_upper)
            .fold(String::new(), |mut buff, w| {
                buff.push_str(w.as_str());
                buff
            })
    }

    fn format_field_type(&self, tpe: &str, json_key: &str) -> String {
        let go_key = self.field_name(&json_key);
        format!("\t{go_key}\t{tpe}\t\t`json:\"{json_key}\"`\n")
    }

    fn format_arr_type(&self, arr_type: String, optional: bool) -> String {
        let type_prefix = if optional { "*" } else { "" };
        format!("[]{type_prefix}{arr_type}")
    }

    fn premitive_type_name(&self, from: &Value) -> &'static str {
        match from {
            Value::Bool(_) => GO_BOOL,
            Value::Number(n) => {
                if n.is_f64() {
                    GO_FLOAT
                } else {
                    GO_INT
                }
            }
            Value::String(_) => GO_STRING,
            Value::Null => GO_ANY,
            // Non-primitives should not be passed to this function
            _ => GO_ANY,
        }
    }

    fn struct_or_class_name(&self, key: &str) -> String {
        self.field_name(key)
    }
}

fn camelcase(snake_case: &str) -> String {
    let mut split = snake_case.split('_');
    let mut first = String::from(split.next().unwrap_or("Unknown"));
    while let Some(w) = split.next() {
        first.push_str(first_char_upper(w).as_str());
    }
    first
}

impl LanguageFormatter for Java {
    fn struct_or_class_header(&self, raw: String) -> String {
        let java_class_name = self.struct_or_class_name(&raw);
        format!("public class {java_class_name} ") + "{\n"
    }

    fn struct_or_class_footer(&self, _struct_name: Option<String>) -> String {
        String::from("}")
    }

    fn field_name(&self, json_key: &str) -> String {
        camelcase(json_key)
    }

    fn format_field_type(&self, tpe: &str, json_key: &str) -> String {
        let java_field_name = self.field_name(json_key);
        format!("\tpublic {tpe} {java_field_name};\n")
    }

    fn format_arr_type(&self, arr_type: String, _optional: bool) -> String {
        format!("List<{arr_type}>")
    }

    fn premitive_type_name(&self, from: &Value) -> &'static str {
        match from {
            Value::Bool(_) => JAVA_BOOL,
            Value::Number(n) => {
                if n.is_f64() {
                    JAVA_FLOAT
                } else {
                    JAVA_INT
                }
            }
            Value::String(_) => JAVA_STRING,
            Value::Null => JAVA_ANY,
            // Non-primitives should not be passed to this function
            _ => JAVA_ANY,
        }
    }

    fn struct_or_class_name(&self, key: &str) -> String {
        key.split('_')
            .map(first_char_upper)
            .fold(String::new(), |mut acc, w| {
                acc.push_str(w.as_str());
                acc
            })
    }
}
