use constants::*;
use formatting::*;
use serde_json::Value;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufRead};

pub mod constants;
pub mod formatting;

type StructValue = String;
type ArrayType = String;

fn infer_struct_name_from_array_key(arr_key: String) -> String {
    if let Some(stripped) = arr_key.strip_suffix("ies") {
        format!("{}y", create_go_key(stripped))
    } else if let Some(stripped) = arr_key.strip_suffix("s") {
        create_go_key(stripped)
    } else {
        create_go_key(arr_key.as_str())
    }
}

fn infer_array(
    key: Option<String>,
    value: &Value,
    structs_into: &mut Vec<StructValue>,
) -> ArrayType {
    if let Value::Array(arr) = value {
        let has_null_values = arr.iter().find(|&js| js.is_null()).is_some();
        let nullable = if has_null_values { PTR } else { NOT_NULL };

        let non_null_values: Vec<&Value> = arr.iter().filter(|js| !js.is_null()).collect();
        if non_null_values.is_empty() {
            format_array_type(ANY, nullable)
        } else {
            let first_inferrable_value = non_null_values[0];
            match first_inferrable_value {
                Value::Array(_) => {
                    let inner_arr_type = infer_array(key, first_inferrable_value, structs_into);
                    format_array_type(&inner_arr_type, nullable)
                }
                Value::Object(_) => {
                    let struct_name = infer_struct_name_from_array_key(
                        key.unwrap_or_else(|| String::from(AUTO_GENERATED)),
                    );
                    infer_struct(struct_name.clone(), first_inferrable_value)
                        .iter()
                        .for_each(|st| structs_into.push(st.to_owned()));
                    format_array_type(&struct_name, nullable)
                }
                other => format_array_type(get_primitive_type_name(other), nullable),
            }
        }
    } else {
        format_array_type(ANY, NOT_NULL)
    }
}

fn infer_struct(struct_name: String, obj: &Value) -> Vec<StructValue> {
    let mut result: Vec<StructValue> = vec![];
    let go_struct_name = create_go_key(&struct_name);
    let mut struct_content: StructValue =
        (format!("type {go_struct_name} struct") + " {\n").to_string();

    if let Value::Object(o) = obj {
        o.iter().for_each(|(json_key, json)| {
            let go_key = create_go_key(json_key.as_str());
            match json {
                Value::Object(_) => {
                    let inner_struct = infer_struct(json_key.to_owned(), json);
                    inner_struct.iter().for_each(|v| result.push(v.to_owned()));
                }
                Value::Array(_) => {
                    let arr_type = infer_array(Some(json_key.to_owned()), json, &mut result);
                    struct_content
                        .push_str(format_member_definition(go_key, &arr_type, json_key).as_str());
                }
                other => struct_content.push_str(
                    format_member_definition(go_key, get_primitive_type_name(other), &json_key)
                        .as_str(),
                ),
            }
        });
        struct_content.push_str("}");
    }
    result.push(struct_content.to_owned());
    result
}

fn generate_types(value: Value) -> Vec<StructValue> {
    let mut result: Vec<StructValue> = vec![];
    match value {
        Value::Array(_) => {
            infer_array(None, &value, &mut result);
        }
        Value::Object(_) => infer_struct(AUTO_GENERATED.to_string(), &value)
            .iter()
            .for_each(|s| result.push(s.to_owned())),
        _ => {}
    }
    result
}

fn usage(app: String) {
    eprintln!("usages of {app}:");
    eprintln!("\t--help:\t\tshow current window");
    eprintln!("\t{app} [FILE]:\tread json file and convert to go structs");
    eprintln!(
        "\t[SOME_COMMAND] | {app}:\n\t\t\tpipe the result of the previous command into {app}"
    );
}


fn main() {
    // first argument is usually the application name
    let result = if std::env::args().len() > 1 {
        match std::env::args().nth(1).unwrap().as_str() {
            "--help" => {
                usage(std::env::args().nth(0).unwrap());
                std::process::exit(0);
            }
            filepath => {
                // read from file
                let file = File::open(filepath).unwrap();
                let value: Value = serde_json::from_reader(file).unwrap();
                generate_types(value)
            }
        }
    } else {
        // read from pipe
        let stdin = io::stdin().lock();

        let all_lines = stdin.lines().fold(String::new(), |mut buff, line| {
            buff.push_str(line.unwrap().as_str());
            buff
        });

        let value: Value = serde_json::from_str(all_lines.as_str()).unwrap();
        generate_types(value)
    };

    result.iter().for_each(|structt| println!("{structt}\n"));
}
