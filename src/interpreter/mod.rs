use crate::parser::{BinaryOp, Expr, Program, Stmt};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i32),
    String(String),
    Void,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Void => write!(f, "()"),
        }
    }
}

pub struct Environment {
    variables: HashMap<String, Value>,
    functions: HashMap<String, (Vec<String>, Vec<Stmt>)>, // (params, body)
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_function(&self, name: &str) -> Option<&(Vec<String>, Vec<Stmt>)> {
        self.functions.get(name)
    }

    pub fn set_function(&mut self, name: String, params: Vec<String>, body: Vec<Stmt>) {
        self.functions.insert(name, (params, body));
    }
}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
        }
    }

    pub fn interpret(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            match self.execute_statement(stmt)? {
                Some(Value::Void) | None => continue,
                Some(_value) => {
                    // TODO: handle (throw error?), for now it's ok
                    continue;
                }
            }
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: &Stmt) -> Result<Option<Value>, String> {
        match stmt {
            Stmt::Let { name, value } => {
                let val = self.evaluate_expression(value)?;
                self.env.set_variable(name.clone(), val);
                Ok(Some(Value::Void))
            }
            Stmt::Function { name, params, body } => {
                self.env
                    .set_function(name.clone(), params.clone(), body.clone());
                Ok(Some(Value::Void))
            }
            Stmt::Return(expr) => {
                let val = self.evaluate_expression(expr)?;
                Ok(Some(val))
            }
            Stmt::Print(expr) => {
                let val = self.evaluate_expression(expr)?;
                println!("{}", val);
                Ok(Some(Value::Void))
            }
            Stmt::Expression(expr) => {
                let val = self.evaluate_expression(expr)?;
                Ok(Some(val))
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Identifier(name) => self
                .env
                .get_variable(name)
                .cloned()
                .ok_or_else(|| format!("Undefined variable: {}", name)),
            Expr::Binary { left, op, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;

                match (left_val, right_val) {
                    (Value::Number(l), Value::Number(r)) => {
                        let result = match op {
                            BinaryOp::Add => l + r,
                            BinaryOp::Subtract => l - r,
                            BinaryOp::Multiply => l * r,
                            BinaryOp::Divide => {
                                if r == 0 {
                                    return Err("Division by zero".to_string());
                                }
                                l / r
                            }
                        };
                        Ok(Value::Number(result))
                    }
                    (Value::String(l), Value::String(r)) => match op {
                        BinaryOp::Add => Ok(Value::String(format!("{}{}", l, r))),
                        _ => Err(format!("Unsupported operation {:?} for strings", op)),
                    },
                    _ => Err("Type mismatch in binary operation".to_string()),
                }
            }
            Expr::FunctionCall { name, args } => {
                // clone to avoid borrowing issues
                if let Some((params, body)) = self.env.get_function(name).cloned() {
                    if params.len() != args.len() {
                        return Err(format!(
                            "Function {} expects {} arguments, got {}",
                            name,
                            params.len(),
                            args.len()
                        ));
                    }

                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.evaluate_expression(arg)?);
                    }

                    let mut func_interpreter = Interpreter::new();
                    func_interpreter.env.functions = self.env.functions.clone();
                    
                    for (param, value) in params.iter().zip(arg_values.iter()) {
                        func_interpreter.env.set_variable(param.clone(), value.clone());
                    }

                    for stmt in &body {
                        match func_interpreter.execute_statement(stmt)? {
                            Some(value) => {
                                if matches!(stmt, Stmt::Return(_)) {
                                    return Ok(value);
                                }
                            }
                            None => continue,
                        }
                    }

                    Ok(Value::Void)
                } else {
                    Err(format!("Undefined function: {}", name))
                }
            }
        }
    }
}

pub fn interpret(program: &Program) {
    let mut interpreter = Interpreter::new();
    match interpreter.interpret(program) {
        Ok(()) => println!("Program executed successfully."),
        Err(e) => eprintln!("Runtime error: {}", e),
    }
}
