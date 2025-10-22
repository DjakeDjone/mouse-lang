use crate::interpreter::{Interpreter, Value};
use std::thread;
use std::time::Duration;

pub fn split_string(_interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, String> {
    let string_val = args.get(0).ok_or("Missing string argument")?;
    let delimiter = args.get(1).ok_or("Missing delimiter argument")?;

    // check if del and string are strings
    if let Value::String(delimiter) = delimiter {
        let string = format!("{}", string_val);
        let parts = string.split(delimiter).collect::<Vec<&str>>();

        let parts_value = parts
            .into_iter()
            .map(|part| Value::String(part.to_string()))
            .collect();

        Ok(Value::Array(parts_value))
    } else {
        Err("Delimiter must be a string".to_string())
    }
}
