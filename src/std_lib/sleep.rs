use crate::interpreter::{Interpreter, Value};
use std::thread;
use std::time::Duration;

pub fn sleep(_interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, String> {
    // Validate arguments: expects 1 argument (milliseconds)
    if args.len() != 1 {
        return Err(format!(
            "sleep expects 1 argument (milliseconds), got {}",
            args.len()
        ));
    }

    let milliseconds = match &args[0] {
        Value::Number(n) => {
            if *n < 0 {
                return Err("sleep duration must be non-negative".to_string());
            }
            *n as u64
        }
        _ => return Err("sleep argument must be a number (milliseconds)".to_string()),
    };

    thread::sleep(Duration::from_millis(milliseconds));

    Ok(Value::Void)
}
