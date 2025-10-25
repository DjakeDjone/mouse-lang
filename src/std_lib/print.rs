use crate::interpreter::{Interpreter, Value};

pub fn print(_interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, String> {
    let message = match &args[0] {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Array(a) => {
            let mut msg = String::new();
            let mut first = true;
            for v in a.iter() {
                if !first {
                    msg.push_str(", ");
                }
                first = false;
                match v {
                    Value::String(s) => msg.push_str(s.as_str()),
                    Value::Number(n) => msg.push_str(&n.to_string()),
                    Value::Array(_inner) => msg.push_str("[...]"),
                    _ => return Err("print argument must be a string or number".to_string()),
                }
            }
            msg
        }
        _ => return Err("print argument must be a string or number".to_string()),
    };

    println!("{}", message);

    Ok(Value::Void)
}
