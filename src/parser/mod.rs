use crate::{
    errors::Error,
    lexer::{Operator, Token, TokenType},
};
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Identifier(String),
    Number(i32),
    String(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Expr),
    // Print(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Expression(Expr), // e.g. let x = 5;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl From<&Operator> for BinaryOp {
    fn from(op: &Operator) -> Self {
        match op {
            Operator::Add => BinaryOp::Add,
            Operator::Subtract => BinaryOp::Subtract,
            Operator::Multiply => BinaryOp::Multiply,
            Operator::Divide => BinaryOp::Divide,
        }
    }
}

fn parse_params(tokens: &[Token], idx: usize) -> Result<(Vec<Expr>, u8), Error> {
    let mut params = Vec::new();
    let mut idx2 = idx;
    loop {
        let token = tokens.get(idx2).ok_or(Error::unexpected_eof())?;
        match &token.token {
            TokenType::Identifier(ident) => {
                params.push(Expr::Identifier(ident.to_owned()));
                idx2 += 1;
            }
            TokenType::Comma => {
                idx2 += 1;
            }
            TokenType::BracketClose => {
                return Ok((params, (idx2 - idx) as u8));
            }
            _ => return Err(Error::syntax_error(token, "identifier or closing bracket")),
        }
    }
}

fn parse_identifier(tokens: &[Token], name: String, idx: usize) -> Result<(Stmt, u8), Error> {
    println!("parse identifier");
    let token = tokens.get(idx + 1).ok_or(Error::unexpected_eof())?;
    match token.token {
        TokenType::Assign => {
            let value = parse_expr(tokens, idx + 2)?;
            Ok((
                Stmt::Assign {
                    name,
                    value: value.0,
                },
                2 + value.1,
            ))
        }
        TokenType::BracketOpen => {
            // parse expression
            let expr = parse_expr(tokens, idx)?;
            Ok((Stmt::Expression(expr.0), expr.1))
        }
        _ => Err(Error::syntax_error(token, "assignment operator")),
    }
}

fn parse_primitive(tokens: &[Token], idx: usize) -> Result<Expr, Error> {
    let token = tokens.get(idx).ok_or(Error::unexpected_eof())?;
    match token.token.to_owned() {
        TokenType::Number(num) => Ok(Expr::Number(num)),
        TokenType::String(str) => Ok(Expr::String(str)),
        TokenType::Identifier(ident) => Ok(Expr::Identifier(ident)), // TODO: function calls
        _ => Err(Error::syntax_error(token, "number or string literal")),
    }
}

/// expression
/// can be a number, string, function call, binary operation
/// returns the parsed expression and the number of tokens consumed
fn parse_expr(tokens: &[Token], idx: usize) -> Result<(Expr, u8), Error> {
    let token = tokens.get(idx).ok_or(Error::unexpected_eof())?;
    let next_token_option = tokens.get(idx + 1);

    if let Some(next_token) = next_token_option {
        // check for binary operator
        if let TokenType::Operator(op) = &next_token.token {
            let right = parse_expr(tokens, idx + 2)?;
            return Ok((
                Expr::Binary {
                    left: Box::new(parse_primitive(tokens, idx)?),
                    op: BinaryOp::from(op),
                    right: Box::new(right.0),
                },
                2 + right.1,
            ));
        }

        // function call
        if let TokenType::BracketOpen = &next_token.token {
            let args = parse_params(tokens, idx + 2)?;
            let name: String = token.clone().token.into();
            return Ok((Expr::FunctionCall { name, args: args.0 }, 2 + args.1));
        }
    }

    // check for number or string literal
    Ok((parse_primitive(tokens, idx)?, 1))
}

fn parse_let(tokens: &[Token], current_token: &Token, idx: usize) -> Result<(Stmt, u8), Error> {
    let name_token = tokens
        .get(idx + 1)
        .ok_or(Error::syntax_error(current_token, "identifier"))?;
    let name = match &name_token.token {
        TokenType::Identifier(name) => name,
        _ => return Err(Error::syntax_error(name_token, "identifier")),
    };

    // expect equal sign
    let equal_token = tokens
        .get(idx + 2)
        .ok_or(Error::syntax_error(current_token, "="))?;
    if equal_token.token != TokenType::Assign {
        return Err(Error::syntax_error(equal_token, "="));
    }

    // value can be a value or an expression
    let value = parse_expr(tokens, idx + 3)?;

    let let_stmt = Stmt::Let {
        name: name.to_owned(),
        value: value.0,
    };
    Ok((let_stmt, value.1 + 2))
}

pub fn parse(tokens: &[Token]) -> Result<Program, Error> {
    let mut program = Program {
        statements: Vec::new(),
    };

    let mut idx = 0;

    while idx < tokens.len() {
        let current_token = tokens.get(idx);
        if let Some(token) = current_token {
            println!("{:?}", token);

            // tokens to ignore
            if token.token == TokenType::Semicolon {
                idx += 1;
                continue;
            }

            let stmt = match &token.token {
                TokenType::KWLet => parse_let(tokens, token, idx),
                TokenType::Identifier(name) => parse_identifier(tokens, name.to_owned(), idx),
                _ => Err(Error::unimplemented_token(token)),
            }?;
            program.statements.push(stmt.0);
            idx += stmt.1 as usize;
        }

        idx += 1;
    }

    Ok(program)
}
