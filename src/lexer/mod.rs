#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
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
    Comment(String),    // comment (e.g. // comment or # comment)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub fn tokenize(input: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' | '\n' => {}
            ';' => tokens.push(Token::Semicolon),
            ',' => tokens.push(Token::Comma),
            '(' => tokens.push(Token::BracketOpen),
            ')' => tokens.push(Token::BracketClose),
            '{' => tokens.push(Token::BraceOpen),
            '}' => tokens.push(Token::BraceClose),
            '/' => {
                if let Some('/') = chars.peek() {
                    chars.next();
                    // Skip comment until end of line
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '\n' {
                            break;
                        }
                    }
                } else {
                    tokens.push(Token::Operator(Operator::Divide));
                }
            }
            '#' => {
                chars.next();
                // Skip comment until end of line
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\n' {
                        break;
                    }
                }
            }
            '=' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Equal);
                } else {
                    tokens.push(Token::Assign);
                }
            }
            '!' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::NotEqual);
                }
            }
            '<' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::LessThanOrEqual);
                } else {
                    tokens.push(Token::LessThan);
                }
            }
            '>' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::GreaterThanOrEqual);
                } else {
                    tokens.push(Token::GreaterThan);
                }
            }
            '+' => tokens.push(Token::Operator(Operator::Add)),
            '-' => tokens.push(Token::Operator(Operator::Subtract)),
            '*' => tokens.push(Token::Operator(Operator::Multiply)),
            '"' => {
                let mut string_val = String::new();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '"' {
                        break;
                    }
                    string_val.push(c);
                }
                tokens.push(Token::String(string_val));
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
                        tokens.push(Token::Number(n));
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
                        "let" | "var" | "const" => Token::KWLet,
                        "fn" | "function" | "def" => Token::KWFn,
                        "return" => Token::KWReturn,
                        "if" => Token::KWIf,
                        "while" => Token::KWWhile,
                        _ => Token::Identifier(identifier),
                    };
                    tokens.push(token);
                }
            }
        }
    }

    tokens
}
