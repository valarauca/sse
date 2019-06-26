
use super::env;

use super::gtmpl::Value;
use super::regex::{Regex,Captures};

lazy_static! {
    static ref hex_value = Regex::new(r#"^\s*0x([A-Za-z0-9]+)\s*$"#).unwrap();
    static ref octal_value = Regex::new(r#"^\s*0o([0-7]+)\s*$"#).unwrap();
    static ref bool_value = Regex::new(r#"^\s*0b([0-1]+)\s*$"#).unwrap();
    static ref decimal_value = Regex::new(r#"^\s*0d([0-9]+)\s*$"#).unwrap();
    static ref decimal_literal = Regex::new(r#"^\s*([0-9]+)\s*$"#).unwrap();
    static ref float32_bool_literal = Regex::new(r#"^\s*0f32b([0-1]{32})\s*$"#).unwrap();
    static ref float32_hex_literal = Regex::new(r#"^\s*0f32x([A-Za-z0-9]{8})\s*$"#).unwrap();
    static ref float64_bool_literal = Regex::new(r#"^\s*0f64b([0-1]{64})\s*$"#).unwrap();
    static ref float64_hex_literal = Regex::new(r#"^\s*0f64x([A-Za-z0-9]{16})\s*$"#).unwrap();
}

pub fn envir_to_value() -> Value {
    Value::Object(env::vars()
        .map(|(k,v)| (k,to_template_context(v)))
        .collect())
}

fn add_captures<'a>(context: &Value, args: &'a Captures<'a>) -> Value {
    let mut map = match context.clone() {
        Value::Object(x) => x,
        cannot_handle => return cannot_handle,
    };
    let mut start = 0;
    loop {
         match args.get(start) {
             Option::Some(ref captured_data) => {
                 map.insert(format!("{}", start), Value::String(captured_data.as_str().to_string()));
             },
             Option::None => break,
         };
    }
    Value::Object(map)
}

fn to_template_context(arg: String) -> Value {
    if arg.len() == 0 {
        Value::NoValue
    } else if !arg.contains(":") {
        Value::String(arg)
    } else {
        Value::Array(arg.split(":")
            .map(|s| Value::String(s.to_string()))
            .collect())
    }
}
