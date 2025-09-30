use crate::parser::{BinaryOp, Expr, Program, Stmt};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i32),
    String(String),
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    None,
    Return(Value),
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
                ControlFlow::None => continue,
                ControlFlow::Return(_value) => {
                    // TODO: handle (throw error?), for now it's ok
                    continue;
                }
            }
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: &Stmt) -> Result<ControlFlow, String> {
        match stmt {
            Stmt::Let { name, value } => {
                let val = self.evaluate_expression(value)?;
                self.env.set_variable(name.clone(), val);
                Ok(ControlFlow::None)
            }
            Stmt::Function { name, params, body } => {
                self.env
                    .set_function(name.clone(), params.clone(), body.clone());
                Ok(ControlFlow::None)
            }
            Stmt::Return(expr) => {
                let val = self.evaluate_expression(expr)?;
                Ok(ControlFlow::Return(val))
            }
            Stmt::Print(expr) => {
                let val = self.evaluate_expression(expr)?;
                println!("{}", val);
                Ok(ControlFlow::None)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch: _,
            } => {
                let condition_value = self.evaluate_expression(condition)?;
                let is_truthy = match condition_value {
                    Value::Number(n) => n != 0,
                    Value::String(s) => !s.is_empty(),
                    Value::Void => false,
                };

                if is_truthy {
                    for stmt in then_branch {
                        match self.execute_statement(stmt)? {
                            ControlFlow::Return(value) => {
                                return Ok(ControlFlow::Return(value));
                            }
                            ControlFlow::None => continue,
                        }
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::Expression(expr) => {
                let _val = self.evaluate_expression(expr)?;
                Ok(ControlFlow::None)
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
                            BinaryOp::Equal => {
                                if l == r {
                                    1
                                } else {
                                    0
                                }
                            }
                            BinaryOp::NotEqual => {
                                if l != r {
                                    1
                                } else {
                                    0
                                }
                            }
                            BinaryOp::LessThan => {
                                if l < r {
                                    1
                                } else {
                                    0
                                }
                            }
                            BinaryOp::LessThanOrEqual => {
                                if l <= r {
                                    1
                                } else {
                                    0
                                }
                            }
                            BinaryOp::GreaterThan => {
                                if l > r {
                                    1
                                } else {
                                    0
                                }
                            }
                            BinaryOp::GreaterThanOrEqual => {
                                if l >= r {
                                    1
                                } else {
                                    0
                                }
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

                    // new interpreter for the function call with fresh variables)
                    let mut func_interpreter = Interpreter::new();
                    // Copy all functions to the new interpreter
                    func_interpreter.env.functions = self.env.functions.clone();

                    // Set function parameters in the new interpreter
                    for (param, value) in params.iter().zip(arg_values.iter()) {
                        func_interpreter
                            .env
                            .set_variable(param.clone(), value.clone());
                    }

                    // Execute the function body
                    for stmt in &body {
                        match func_interpreter.execute_statement(stmt)? {
                            ControlFlow::Return(value) => {
                                return Ok(value);
                            }
                            ControlFlow::None => continue,
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
