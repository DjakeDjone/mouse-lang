use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxError {
    UnexpectedToken(Option<Token>, String),
    UnimplementedToken(Token),
    UnexpectedEof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorTypes {
    LexicalError(String), // error for lexical/invalid_token
    SyntaxError(SyntaxError),
    RuntimeError(String),
}

impl std::fmt::Display for ErrorTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorTypes::LexicalError(msg) => write!(f, "Lexical error: {}", msg),
            ErrorTypes::SyntaxError(error) => write!(f, "Syntax error: {:?}", error),
            ErrorTypes::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    // position
    pub line: u32,
    pub column: u32,

    pub error_type: ErrorTypes,
}

impl Error {
    pub fn new(line: u32, column: u32, error_type: ErrorTypes) -> Self {
        Error {
            line,
            column,
            error_type,
        }
    }

    pub fn unimplemented_token(token: &Token) -> Self {
        Error {
            line: token.line,
            column: token.column,
            error_type: ErrorTypes::LexicalError(format!("Unimplemented token: {:?}", token)),
        }
    }

    pub fn syntax_error(token: &Token, expected: &str) -> Self {
        Error {
            line: token.line,
            column: token.column,
            error_type: ErrorTypes::SyntaxError(SyntaxError::UnexpectedToken(
                Some(token.clone()),
                expected.to_string(),
            )),
        }
    }

    pub fn unexpected_eof() -> Self {
        Error {
            line: 0,
            column: 0,
            error_type: ErrorTypes::SyntaxError(SyntaxError::UnexpectedEof),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Error at line {}, column {}: {}",
            self.line, self.column, self.error_type
        )
    }
}
