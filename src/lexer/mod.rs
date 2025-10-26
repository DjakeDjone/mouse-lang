#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    KWLet,              // let
    KWFn,               // fn
    KWReturn,           // return
    KWIf,               // if
    KWWhile,            // while
    Identifier(String), // identifier (e.g. a)
    Number(i32),        // number literal
    String(String),     // string literal
    Operator(Operator), // operator (e.g. +)
    Equal,              // ==
    NotEqual,           // !=
    LessThan,           // <
    LessThanOrEqual,    // <=
    GreaterThan,        // >
    GreaterThanOrEqual, // >=
    Assign,             // =
    BracketOpen,        // (
    BracketClose,       // )
    BraceOpen,          // {
    BraceClose,         // }
    Comma,              // ,
    Semicolon,          // ;
    Dot,                // .
    Comment(String),    // comment (e.g. // comment or # comment)
}

impl From<TokenType> for String {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::KWLet => "let".to_string(),
            TokenType::KWFn => "fn".to_string(),
            TokenType::KWReturn => "return".to_string(),
            TokenType::KWIf => "if".to_string(),
            TokenType::KWWhile => "while".to_string(),
            TokenType::Identifier(name) => name,
            TokenType::Number(num) => num.to_string(),
            TokenType::String(str) => str,
            TokenType::Operator(op) => op.into(),
            TokenType::Equal => "==".to_string(),
            TokenType::NotEqual => "!=".to_string(),
            TokenType::LessThan => "<".to_string(),
            TokenType::LessThanOrEqual => "<=".to_string(),
            TokenType::GreaterThan => ">".to_string(),
            TokenType::GreaterThanOrEqual => ">=".to_string(),
            TokenType::Assign => "=".to_string(),
            TokenType::BracketOpen => "(".to_string(),
            TokenType::BracketClose => ")".to_string(),
            TokenType::BraceOpen => "{".to_string(),
            TokenType::BraceClose => "}".to_string(),
            TokenType::Comma => ",".to_string(),
            TokenType::Semicolon => ";".to_string(),
            TokenType::Dot => ".".to_string(),
            TokenType::Comment(comment) => comment,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Into<String> for Operator {
    fn into(self) -> String {
        match self {
            Operator::Add => "+".to_string(),
            Operator::Subtract => "-".to_string(),
            Operator::Multiply => "*".to_string(),
            Operator::Divide => "/".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub token: TokenType,
    pub line: u32,
    pub column: u32,
}

impl Token {
    pub fn new(token: TokenType, line: u32, column: u32) -> Self {
        Token {
            token,
            line,
            column,
        }
    }
}

pub fn tokenize(input: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    let mut line = 0;
    let mut column = 0;
    while let Some(c) = chars.next() {
        column += 1;
        match c {
            ' ' | '\t' | '\n' => {
                if c == '\n' {
                    line += 1;
                    column = 0;
                }
            }
            ';' => tokens.push(Token::new(TokenType::Semicolon, line, column)),
            ',' => tokens.push(Token::new(TokenType::Comma, line, column)),
            '.' => tokens.push(Token::new(TokenType::Dot, line, column)),
            '(' => tokens.push(Token::new(TokenType::BracketOpen, line, column)),
            ')' => tokens.push(Token::new(TokenType::BracketClose, line, column)),
            '{' => tokens.push(Token::new(TokenType::BraceOpen, line, column)),
            '}' => tokens.push(Token::new(TokenType::BraceClose, line, column)),
            '/' => {
                if let Some('/') = chars.peek() {
                    chars.next();
                    // Skip comment until end of line
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '\n' {
                            line += 1;
                            column = 0;
                            break;
                        }
                    }
                } else {
                    tokens.push(Token::new(
                        TokenType::Operator(Operator::Divide),
                        line,
                        column,
                    ));
                }
            }
            '#' => {
                chars.next();
                // Skip comment until end of line
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\n' {
                        line += 1;
                        column = 0;
                        break;
                    }
                }
            }
            '=' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(TokenType::Equal, line, column));
                } else {
                    tokens.push(Token::new(TokenType::Assign, line, column));
                }
            }
            '!' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(TokenType::NotEqual, line, column));
                }
            }
            '<' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(TokenType::LessThanOrEqual, line, column));
                } else {
                    tokens.push(Token::new(TokenType::LessThan, line, column));
                }
            }
            '>' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(TokenType::GreaterThanOrEqual, line, column));
                } else {
                    tokens.push(Token::new(TokenType::GreaterThan, line, column));
                }
            }
            '+' => tokens.push(Token::new(TokenType::Operator(Operator::Add), line, column)),
            '-' => tokens.push(Token::new(
                TokenType::Operator(Operator::Subtract),
                line,
                column,
            )),
            '*' => tokens.push(Token::new(
                TokenType::Operator(Operator::Multiply),
                line,
                column,
            )),
            '"' => {
                let mut string_val = String::new();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '"' {
                        break;
                    }
                    string_val.push(c);
                }
                tokens.push(Token::new(TokenType::String(string_val), line, column));
            }
            _ => {
                if c.is_ascii_digit() {
                    let mut number = String::new();
                    number.push(c);
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() {
                            number.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if let Ok(n) = number.parse::<i32>() {
                        tokens.push(Token::new(TokenType::Number(n), line, column));
                    }
                } else if c.is_alphabetic() || c == '_' {
                    let mut identifier = String::new();
                    identifier.push(c);
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            identifier.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    let token = match identifier.as_str() {
                        "let" | "var" | "const" => TokenType::KWLet,
                        "fn" | "function" | "def" => TokenType::KWFn,
                        "return" => TokenType::KWReturn,
                        "if" => TokenType::KWIf,
                        "while" => TokenType::KWWhile,
                        _ => TokenType::Identifier(identifier),
                    };
                    tokens.push(Token::new(token, line, column));
                }
            }
        }
    }

    tokens
}

/// fixes issues like missing semicolons at the end of lines
pub fn autofix(input: &str) -> String {
    let mut output = String::new();
    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim_end();
        // println!("Autofix processing line: '{}'", line);

        if !trimmed.is_empty()
            && !trimmed.ends_with(';')
            && !trimmed.ends_with('{')
            && !trimmed.ends_with('}')
            && !trimmed.ends_with(',')
            && !trimmed.ends_with('(')
        {
            output.push_str(trimmed);
            output.push_str(";\n");
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    // println!("Autofix output:\n{}", output);

    output
}
