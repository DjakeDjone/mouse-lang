#[cfg(test)]
mod tests {
    use crate::interpreter::{Interpreter, Value};
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn run_code(code: &str) -> Result<Interpreter, String> {
        let tokens = tokenize(code.to_string());
        let program = parse(&tokens).map_err(|e| format!("Parse error: {:?}", e))?;
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&program)?;
        Ok(interpreter)
    }

    fn run_and_get_var(code: &str, var_name: &str) -> Result<Value, String> {
        let interpreter = run_code(code)?;
        interpreter
            .env
            .get_variable(var_name)
            .cloned()
            .ok_or_else(|| format!("Variable {} not found", var_name))
    }

    // ===== Basic Variable Tests =====

    #[test]
    fn test_let_number() {
        let code = "let x = 42;";
        let result = run_and_get_var(code, "x").unwrap();
        assert_eq!(result, Value::Number(42));
    }

    #[test]
    fn test_let_string() {
        let code = r#"let greeting = "Hello, World!";"#;
        let result = run_and_get_var(code, "greeting").unwrap();
        assert_eq!(result, Value::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_let_negative_number() {
        let code = "let x = 0 - 10;";
        let result = run_and_get_var(code, "x").unwrap();
        assert_eq!(result, Value::Number(-10));
    }

    #[test]
    fn test_variable_assignment() {
        let code = "let x = 5; x = 10;";
        let result = run_and_get_var(code, "x").unwrap();
        assert_eq!(result, Value::Number(10));
    }

    #[test]
    fn test_assignment_to_undefined_variable_fails() {
        let code = "x = 10;";
        let result = run_code(code);
        assert!(result.is_err());
    }

    // ===== Arithmetic Tests =====

    #[test]
    fn test_addition() {
        let code = "let result = 5 + 3;";
        let result = run_and_get_var(code, "result").unwrap();
        assert_eq!(result, Value::Number(8));
    }

    #[test]
    fn test_subtraction() {
        let code = "let result = 10 - 3;";
        let result = run_and_get_var(code, "result").unwrap();
        assert_eq!(result, Value::Number(7));
    }

    #[test]
    fn test_multiplication() {
        let code = "let result = 6 * 7;";
        let result = run_and_get_var(code, "result").unwrap();
        assert_eq!(result, Value::Number(42));
    }

    #[test]
    fn test_division() {
        let code = "let result = 20 / 4;";
        let result = run_and_get_var(code, "result").unwrap();
        assert_eq!(result, Value::Number(5));
    }

    #[test]
    fn test_division_by_zero_fails() {
        let code = "let result = 10 / 0;";
        let result = run_code(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_operator_precedence_multiply_before_add() {
        let code = "let result = 2 + 3 * 4;";
        let result = run_and_get_var(code, "result").unwrap();
        assert_eq!(result, Value::Number(14)); // 2 + (3 * 4) = 14
    }

    #[test]
    fn test_operator_precedence_divide_before_subtract() {
        let code = "let result = 20 - 10 / 2;";
        let result = run_and_get_var(code, "result").unwrap();
        assert_eq!(result, Value::Number(15)); // 20 - (10 / 2) = 15
    }

    #[test]
    fn test_complex_arithmetic() {
        let code = "let result = 2 * 3 + 4 * 5;";
        let result = run_and_get_var(code, "result").unwrap();
        assert_eq!(result, Value::Number(26)); // (2 * 3) + (4 * 5) = 6 + 20 = 26
    }
}
