pub mod constants;
pub mod language;

use std::fs::File;
use std::io::{self, BufRead, Error};
use std::rc::Rc;

use serde_json::Value;

use constants::*;
use language::*;

type StructValue = String;
type ArrayType = String;

fn infer_array(
    key: Option<String>,
    value: &Value,
    structs_into: &mut Vec<StructValue>,
    lang: Rc<dyn LanguageFormatter>,
) -> ArrayType {
    if let Value::Array(arr) = value {
        let optional = arr.iter().any(Value::is_null);

        let non_null_values: Vec<&Value> = arr.iter().filter(|js| !js.is_null()).collect();

        if non_null_values.is_empty() {
            let null = Value::Null;
            lang.format_arr_type(lang.premitive_type_name(&null).to_owned(), optional)
        } else {
            let first_inferrable_value = non_null_values[0];
            match first_inferrable_value {
                Value::Array(_) => {
                    let inner_arr_type =
                        infer_array(key, first_inferrable_value, structs_into, Rc::clone(&lang));
                    lang.format_arr_type(inner_arr_type, optional)
                }
                Value::Object(_) => {
                    let struct_name = lang.struct_or_class_name(
                        key.unwrap_or_else(|| String::from(GO_AUTO_GENERATED))
                            .as_str(),
                    );
                    infer_struct(
                        struct_name.clone(),
                        first_inferrable_value,
                        Rc::clone(&lang),
                    )
                    .iter()
                    .for_each(|st| structs_into.push(st.to_owned()));
                    lang.format_arr_type(struct_name, optional)
                }
                other => {
                    lang.format_arr_type(lang.premitive_type_name(&other).to_owned(), optional)
                }
            }
        }
    } else {
        let null: Value = Value::Null;
        lang.format_arr_type(lang.premitive_type_name(&null).to_string(), false)
    }
}

fn infer_struct(
    struct_name: String,
    obj: &Value,
    lang: Rc<dyn LanguageFormatter>,
) -> Vec<StructValue> {
    let mut result: Vec<StructValue> = vec![];
    let mut struct_content: String = lang.struct_or_class_header(struct_name.clone());

    if let Value::Object(o) = obj {
        o.iter().for_each(|(json_key, json)| match json {
            Value::Object(_) => {
                let inner_struct = infer_struct(json_key.to_owned(), json, Rc::clone(&lang));
                inner_struct.iter().for_each(|v| result.push(v.to_owned()));
                struct_content.push_str(
                    lang.format_field_type(
                        &lang.struct_or_class_name(json_key),
                        &lang.field_name(json_key),
                    )
                    .as_str(),
                );
            }
            Value::Array(_) => {
                let arr_type = infer_array(
                    Some(json_key.to_owned()),
                    json,
                    &mut result,
                    Rc::clone(&lang),
                );
                struct_content.push_str(lang.format_field_type(&arr_type, json_key).as_str());
            }
            other => struct_content.push_str(
                lang.format_field_type(lang.premitive_type_name(other), json_key)
                    .as_str(),
            ),
        });
        struct_content.push_str(
            lang.struct_or_class_footer(Some(struct_name.clone()))
                .as_str(),
        );
    }
    result.push(struct_content.to_owned());
    result
}

fn generate_types(value: Value, lang: Rc<dyn LanguageFormatter>) -> Vec<StructValue> {
    let mut result: Vec<StructValue> = vec![];
    match value {
        Value::Array(_) => {
            infer_array(None, &value, &mut result, lang);
        }
        Value::Object(_) => infer_struct(GO_AUTO_GENERATED.to_string(), &value, lang)
            .iter()
            .for_each(|s| result.push(s.to_owned())),
        _ => {}
    }
    result
}

fn usage(app: String) {
    eprintln!("usages of {app}:");
    eprintln!("OPTIONS: \n\t[-l|--language]: Specify the output programming language");
    eprintln!("\t--help:\t\tshow current window");
    eprintln!("\t{app} [FILE]:\tread json file and convert to go structs");
    eprintln!(
        "\t[SOME_COMMAND] | {app}:\n\t\t\tpipe the result of the previous command into {app}"
    );
}

fn from_filepath(
    filepath: &str,
    lang: Rc<dyn LanguageFormatter>,
) -> Result<Vec<StructValue>, Error> {
    let file = File::open(filepath)?;
    let value: Value = serde_json::from_reader(file)?;
    Ok(generate_types(value, lang))
}

fn acquire_pipe(lang: Rc<dyn LanguageFormatter>) -> Vec<StructValue> {
    let stdin = io::stdin().lock();

    let all_lines = stdin.lines().fold(String::new(), |mut buff, line| {
        buff.push_str(line.unwrap().as_str());
        buff
    });

    let value: Value = serde_json::from_str(all_lines.as_str()).unwrap();
    generate_types(value, lang)
}

fn main() {
    // first argument is usually the application name
    let result = if std::env::args().len() > 1 {
        match std::env::args().nth(1).unwrap().as_str() {
            "--help" => {
                usage(std::env::args().nth(0).unwrap());
                std::process::exit(0);
            }
            "-l" | "--language" => {
                let lang = std::env::args()
                    .nth(2)
                    .expect("Programming language not specified");
                let lang_specifier = get_language_formatter(lang.as_str())
                    .expect("Couldn't find the language specifier");

                if let Some(filepath) = std::env::args().nth(3) {
                    from_filepath(&filepath, lang_specifier).unwrap()
                } else {
                    acquire_pipe(lang_specifier)
                }
            }
            filepath => {
                from_filepath(filepath, get_language_formatter(DEFAULT_LANG).unwrap()).unwrap()
            }
        }
    } else {
        acquire_pipe(get_language_formatter(DEFAULT_LANG).unwrap())
    };

    println!("{}", &result[0]);
    result[1..].iter().for_each(|s| {println!("\n{s}");})
}
