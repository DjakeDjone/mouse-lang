use crate::interpreter::{Interpreter, Value};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_string() {
        let mut interpreter = Interpreter::new();
        let result = split_string(
            &mut interpreter,
            vec![
                Value::String("hello world".to_string()),
                Value::String(" ".to_string()),
            ],
        );
        assert_eq!(
            result,
            Ok(Value::Array(vec![
                Value::String("hello".to_string()),
                Value::String("world".to_string())
            ]))
        );
    }

    #[test]
    fn test_split_string_empty() {
        let mut interpreter = Interpreter::new();
        let result = split_string(
            &mut interpreter,
            vec![
                Value::String("hi".to_string()),
                Value::String("".to_string()),
            ],
        );
        assert_eq!(
            result,
            Ok(Value::Array(vec![
                Value::String("".to_string()),
                Value::String("h".to_string()),
                Value::String("i".to_string()),
                Value::String("".to_string())
            ]))
        );
    }
}
