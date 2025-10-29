use crate::{
    parser::{BinaryOp, Expr, Program, Stmt},
    std_lib,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    name: String,
    properties: HashMap<String, Value>,
}

impl Object {
    pub fn new(name: &str) -> Self {
        Object {
            name: name.to_string(),
            properties: HashMap::new(),
        }
    }

    pub fn with_properties(name: &str, properties: HashMap<String, Value>) -> Self {
        Object {
            name: name.to_string(),
            properties,
        }
    }

    pub fn set_property(&mut self, key: String, value: Value) {
        self.properties.insert(key, value);
    }

    pub fn get_property(&self, name: &str) -> Option<&Value> {
        self.properties.get(name)
    }

    pub fn register_native_fn(
        &mut self,
        name: &str,
        func: fn(&mut Interpreter, Vec<Value>) -> Result<Value, String>,
    ) {
        self.properties.insert(
            name.to_string(),
            Value::NativeFunction(name.to_string(), func),
        );
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{}: {}}}",
            self.name,
            self.properties
                .iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i32),
    String(String),
    Void,
    Array(Vec<Value>),
    Function(String, Vec<String>, Vec<Stmt>), // name, params, body
    #[allow(unpredictable_function_pointer_comparisons)]
    NativeFunction(
        String,
        fn(&mut Interpreter, Vec<Value>) -> Result<Value, String>,
    ),
    Object(Object),
}

impl Value {
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Number(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Void => false,
            Value::Array(arr) => !arr.is_empty(),
            Value::Function(_, _, _) => true,
            Value::NativeFunction(_, _) => true,
            Value::Object(obj) => !obj.properties.is_empty(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Void => write!(f, "()"),
            Value::Array(arr) => write!(
                f,
                "[{}]",
                arr.iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Function(name, _, _) => write!(f, "<function {}>", name),
            Value::NativeFunction(name, _) => write!(f, "<native function {}>", name),
            Value::Object(obj) => write!(f, "{}", obj),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    None,
    Return(Value),
}

pub struct Environment {
    pub variables: HashMap<String, Value>,
    pub objects: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Environment {
            variables: HashMap::new(),
            objects: HashMap::new(),
        };

        // Create global object for global functions
        env.objects
            .insert("global".to_string(), Object::new("global"));

        // Register standard library
        env.register_std_lib();

        env
    }

    fn register_std_lib(&mut self) {
        // Register global print function
        if let Some(global) = self.objects.get_mut("global") {
            global.register_native_fn("print", std_lib::print::print);
        }

        // Register std library
        let mut std_object = Object::new("std");
        std_object.register_native_fn("print", std_lib::print::print);
        std_object.register_native_fn("sleep", std_lib::sleep::sleep);
        std_object.register_native_fn("split_str", std_lib::str_utils::split_string);

        self.objects.insert("std".to_string(), std_object);
    }

    pub fn create_child(&self) -> Environment {
        let env = Environment {
            variables: HashMap::new(),
            objects: self.objects.clone(),
        };
        env
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.objects.get("global")?.get_property(name)
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        if let Some(global) = self.objects.get_mut("global") {
            global.set_property(name.clone(), value);
        }
    }

    pub fn get_object(&self, name: &str) -> Option<&Object> {
        self.objects.get(name)
    }

    pub fn get_object_mut(&mut self, name: &str) -> Option<&mut Object> {
        self.objects.get_mut(name)
    }

    pub fn get_global_function(&self, name: &str) -> Option<&Value> {
        self.objects.get("global")?.get_property(name)
    }

    pub fn set_global_function(&mut self, name: String, params: Vec<String>, body: Vec<Stmt>) {
        if let Some(global) = self.objects.get_mut("global") {
            global.set_property(name.clone(), Value::Function(name, params, body));
        }
    }
}

pub struct Interpreter {
    pub env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
        }
    }

    pub fn create_child(&self) -> Interpreter {
        Interpreter {
            env: self.env.create_child(),
        }
    }

    pub fn interpret(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            match self.execute_statement(stmt)? {
                ControlFlow::None => continue,
                ControlFlow::Return(_) => {
                    // Top-level return, we can ignore or handle as needed
                    continue;
                }
            }
        }
        Ok(())
    }

    pub fn execute_statement(&mut self, stmt: &Stmt) -> Result<ControlFlow, String> {
        match stmt {
            Stmt::Let { name, value } => {
                let val = self.evaluate_expression(value)?;
                self.env.set_variable(name.clone(), val);
                Ok(ControlFlow::None)
            }
            Stmt::Assign { name, value } => {
                if self.env.get_variable(name).is_none() {
                    return Err(format!("Cannot assign to undefined variable: {}", name));
                }
                let val = self.evaluate_expression(value)?;
                self.env.set_variable(name.clone(), val);
                Ok(ControlFlow::None)
            }
            Stmt::Function { name, params, body } => {
                self.env
                    .set_global_function(name.clone(), params.clone(), body.clone());
                Ok(ControlFlow::None)
            }
            Stmt::Return(expr) => {
                let val = self.evaluate_expression(expr)?;
                Ok(ControlFlow::Return(val))
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = self.evaluate_expression(condition)?;

                if condition_value.to_bool() {
                    self.execute_block(then_branch)
                } else if let Some(else_body) = else_branch {
                    self.execute_block(else_body)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            Stmt::While { condition, body } => {
                while self.evaluate_expression(condition)?.to_bool() {
                    match self.execute_block(body)? {
                        ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                        ControlFlow::None => continue,
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(ControlFlow::None)
            }
        }
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<ControlFlow, String> {
        for stmt in statements {
            match self.execute_statement(stmt)? {
                ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                ControlFlow::None => continue,
            }
        }
        Ok(ControlFlow::None)
    }

    fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Identifier(name) => {
                // First check variables
                if let Some(value) = self.env.get_variable(name) {
                    return Ok(value.clone());
                }

                // Then check global functions
                if let Some(func) = self.env.get_global_function(name) {
                    return Ok(func.clone());
                }

                // Finally check if it's an object
                if let Some(obj) = self.env.get_object(name) {
                    return Ok(Value::Object(obj.clone()));
                }

                Err(format!("Undefined identifier: {}", name))
            }
            Expr::Binary { left, op, right } => self.evaluate_binary_op(left, op, right),
            Expr::FunctionCall { name, args } => self.evaluate_function_call(name, args),
            Expr::ObjectCall(object_name, member_expr) => {
                self.evaluate_object_call(object_name, member_expr)
            }
        }
    }

    fn evaluate_binary_op(
        &mut self,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
    ) -> Result<Value, String> {
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
                BinaryOp::Equal => Ok(Value::Number(if l == r { 1 } else { 0 })),
                BinaryOp::NotEqual => Ok(Value::Number(if l != r { 1 } else { 0 })),
                _ => Err(format!("Unsupported operation {:?} for strings", op)),
            },
            (Value::String(l), r) => match op {
                BinaryOp::Add => Ok(Value::String(format!("{}{}", l, r))),
                _ => Err(format!(
                    "Unsupported operation {:?} for string and {}",
                    op, r
                )),
            },
            (l, Value::String(r)) => match op {
                BinaryOp::Add => Ok(Value::String(format!("{}{}", l, r))),
                _ => Err(format!(
                    "Unsupported operation {:?} for {} and string",
                    op, l
                )),
            },
            _ => Err("Type mismatch in binary operation".to_string()),
        }
    }

    fn evaluate_function_call(&mut self, name: &str, args: &[Expr]) -> Result<Value, String> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.evaluate_expression(arg)?);
        }

        // Check for global functions
        if let Some(func) = self.env.get_global_function(name).cloned() {
            match func {
                Value::Function(_, params, body) => {
                    self.call_user_function(&params, &body, arg_values)
                }
                Value::NativeFunction(_, native_fn) => native_fn(self, arg_values),
                _ => Err(format!("{} is not a function", name)),
            }
        } else {
            Err(format!("Undefined function: {}", name))
        }
    }

    fn evaluate_object_call(
        &mut self,
        object_name: &str,
        member_expr: &Expr,
    ) -> Result<Value, String> {
        // Get the object
        let object = self
            .env
            .get_object(object_name)
            .ok_or_else(|| format!("Undefined object: {}", object_name))?
            .clone();

        // Handle the member expression
        match member_expr {
            Expr::Identifier(prop_name) => {
                // Simple property access: obj.prop
                object.get_property(prop_name).cloned().ok_or_else(|| {
                    format!(
                        "Property '{}' not found on object '{}'",
                        prop_name, object_name
                    )
                })
            }
            Expr::FunctionCall { name, args } => {
                // Method call: obj.method(args)
                let method = object.get_property(name).ok_or_else(|| {
                    format!("Method '{}' not found on object '{}'", name, object_name)
                })?;

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate_expression(arg)?);
                }

                // Call the method
                match method.clone() {
                    Value::Function(_, params, body) => {
                        self.call_user_function(&params, &body, arg_values)
                    }
                    Value::NativeFunction(_, native_fn) => native_fn(self, arg_values),
                    _ => Err(format!(
                        "'{}' is not a method on object '{}'",
                        name, object_name
                    )),
                }
            }
            Expr::ObjectCall(nested_obj, nested_member) => {
                // Nested object call: obj.nested.member
                // First get the nested object from the parent
                let nested_value = object.get_property(nested_obj).ok_or_else(|| {
                    format!(
                        "Property '{}' not found on object '{}'",
                        nested_obj, object_name
                    )
                })?;

                match nested_value {
                    Value::Object(nested_object) => {
                        // Recursively evaluate the nested member
                        self.evaluate_nested_object_call(nested_object, nested_member)
                    }
                    _ => Err(format!(
                        "'{}' is not an object on '{}'",
                        nested_obj, object_name
                    )),
                }
            }
            _ => Err(format!("Invalid member access on object '{}'", object_name)),
        }
    }

    fn evaluate_nested_object_call(
        &mut self,
        object: &Object,
        member_expr: &Expr,
    ) -> Result<Value, String> {
        match member_expr {
            Expr::Identifier(prop_name) => object
                .get_property(prop_name)
                .cloned()
                .ok_or_else(|| format!("Property '{}' not found on object", prop_name)),
            Expr::FunctionCall { name, args } => {
                let method = object
                    .get_property(name)
                    .cloned()
                    .ok_or_else(|| format!("Method '{}' not found on object", name))?;

                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate_expression(arg)?);
                }

                match method {
                    Value::Function(_, params, body) => {
                        self.call_user_function(&params, &body, arg_values)
                    }
                    Value::NativeFunction(_, native_fn) => native_fn(self, arg_values),
                    _ => Err(format!("'{}' is not a method", name)),
                }
            }
            _ => Err("Invalid nested member access".to_string()),
        }
    }

    fn call_user_function(
        &mut self,
        params: &[String],
        body: &[Stmt],
        arg_values: Vec<Value>,
    ) -> Result<Value, String> {
        if params.len() != arg_values.len() {
            return Err(format!(
                "Function expects {} arguments, got {}",
                params.len(),
                arg_values.len()
            ));
        }

        // Create new interpreter scope for function
        let mut func_interpreter = self.create_child();

        // Set parameters as local variables
        for (param, value) in params.iter().zip(arg_values.iter()) {
            func_interpreter
                .env
                .set_variable(param.clone(), value.clone());
        }

        // Execute function body
        for stmt in body {
            match func_interpreter.execute_statement(stmt)? {
                ControlFlow::Return(value) => return Ok(value),
                ControlFlow::None => continue,
            }
        }

        Ok(Value::Void)
    }
}

pub fn interpret(program: &Program) {
    let mut interpreter = Interpreter::new();
    match interpreter.interpret(program) {
        Ok(()) => println!("Program executed successfully."),
        Err(e) => eprintln!("Runtime error: {}", e),
    }
}
