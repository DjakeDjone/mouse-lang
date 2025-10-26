use crate::{
    errors::Error,
    lexer::{Comparison, Operator, Token, TokenType},
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

impl From<&Comparison> for BinaryOp {
    fn from(op: &Comparison) -> Self {
        match op {
            Comparison::Equal => BinaryOp::Equal,
            Comparison::NotEqual => BinaryOp::NotEqual,
            Comparison::LessThan => BinaryOp::LessThan,
            Comparison::LessThanOrEqual => BinaryOp::LessThanOrEqual,
            Comparison::GreaterThan => BinaryOp::GreaterThan,
            Comparison::GreaterThanOrEqual => BinaryOp::GreaterThanOrEqual,
        }
    }
}

fn parse_fn_call_params(tokens: &[Token], idx: usize) -> Result<(Vec<Expr>, u8), Error> {
    let mut params = Vec::new();
    let mut idx2 = idx;
    loop {
        let token = tokens
            .get(idx2)
            .ok_or(Error::unexpected_eof("parse_fn_call_params"))?;
        match &token.token {
            // TokenType::Identifier(ident) => {
            //     params.push(Expr::Identifier(ident.to_owned()));
            //     idx2 += 1;
            // }
            TokenType::Comma => {
                idx2 += 1;
            }
            TokenType::BracketClose => {
                return Ok((params, (idx2 - idx) as u8));
            }
            _ => {
                let (expr, len) = parse_expr(tokens, idx2)?;
                params.push(expr);
                idx2 += len as usize;
            }
        }
    }
}

fn parse_params(tokens: &[Token], idx: usize) -> Result<(Vec<String>, u8), Error> {
    let mut params: Vec<String> = Vec::new();
    let mut idx2 = idx;
    // only strings
    while idx2 < tokens.len() {
        let token = tokens
            .get(idx2)
            .ok_or(Error::unexpected_eof("parse_params"))?;
        match &token.token {
            TokenType::Identifier(ident) => {
                params.push(ident.to_owned());
                idx2 += 1;
            }
            TokenType::Comma => {
                idx2 += 1;
            }
            TokenType::BracketClose => {
                return Ok((params, (idx2 - idx) as u8));
            }
            _ => {
                return Err(Error::syntax_error(
                    token,
                    "string literal or closing bracket",
                    "parse_params",
                ))
            }
        }
    }
    Err(Error::unexpected_eof("parse_params"))
}

fn parse_fn(tokens: &[Token], idx: usize) -> Result<(Stmt, u8), Error> {
    let token = tokens
        .get(idx + 1)
        .ok_or(Error::unexpected_eof("parse_fn"))?;
    match &token.token {
        TokenType::Identifier(name) => {
            let params = parse_params(tokens, idx + 3)?;
            let body = parse_block(tokens, idx + params.1 as usize + 5)?;
            Ok((
                Stmt::Function {
                    name: name.to_owned(),
                    params: params.0,
                    body: body.0,
                },
                5 + params.1 + body.1 as u8,
            ))
        }
        _ => Err(Error::syntax_error(token, "function name", "parse_fn")),
    }
}

fn parse_identifier(tokens: &[Token], name: String, idx: usize) -> Result<(Stmt, u8), Error> {
    println!("parse identifier");
    let token = tokens
        .get(idx + 1)
        .ok_or(Error::unexpected_eof("parse_identifier"))?;
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
        _ => Err(Error::syntax_error(
            token,
            "assignment operator",
            "parse_identifier",
        )),
    }
}

/// Parses a primary expression: number, string, identifier, or function call
/// Returns the parsed expression and the number of tokens consumed
fn parse_primary(tokens: &[Token], idx: usize) -> Result<(Expr, u8), Error> {
    let token = tokens
        .get(idx)
        .ok_or(Error::unexpected_eof("parse_primary"))?;

    match &token.token {
        TokenType::Number(num) => Ok((Expr::Number(*num), 1)),
        TokenType::String(str) => Ok((Expr::String(str.clone()), 1)),
        TokenType::Identifier(ident) => {
            // Check if this is a function call
            if let Some(next_token) = tokens.get(idx + 1) {
                if next_token.token == TokenType::BracketOpen {
                    let args = parse_fn_call_params(tokens, idx + 2)?;
                    return Ok((
                        Expr::FunctionCall {
                            name: ident.clone(),
                            args: args.0,
                        },
                        3 + args.1,
                    ));
                }
            }
            Ok((Expr::Identifier(ident.clone()), 1))
        }
        _ => Err(Error::syntax_error(
            token,
            "number, string, or identifier",
            "parse_primary",
        )),
    }
}

/// Parses multiplication and division (higher precedence)
/// Returns the parsed expression and the number of tokens consumed
fn parse_term(tokens: &[Token], idx: usize) -> Result<(Expr, u8), Error> {
    let (mut left, mut consumed) = parse_primary(tokens, idx)?;

    loop {
        let next_idx = idx + consumed as usize;
        if let Some(next_token) = tokens.get(next_idx) {
            match &next_token.token {
                TokenType::Operator(Operator::Multiply) | TokenType::Operator(Operator::Divide) => {
                    let op = BinaryOp::from(match &next_token.token {
                        TokenType::Operator(op) => op,
                        _ => unreachable!(),
                    });
                    let (right, right_consumed) = parse_primary(tokens, next_idx + 1)?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    };
                    consumed += 1 + right_consumed;
                }
                _ => break,
            }
        } else {
            break;
        }
    }

    Ok((left, consumed))
}

/// Parses addition, subtraction, and comparisons (lower precedence)
/// Returns the parsed expression and the number of tokens consumed
fn parse_expr(tokens: &[Token], idx: usize) -> Result<(Expr, u8), Error> {
    let (mut left, mut consumed) = parse_term(tokens, idx)?;

    loop {
        let next_idx = idx + consumed as usize;
        if let Some(next_token) = tokens.get(next_idx) {
            println!("Parse expr: Next token: {:?}", next_token);
            match &next_token.token {
                TokenType::Operator(Operator::Add) | TokenType::Operator(Operator::Subtract) => {
                    let op = BinaryOp::from(match &next_token.token {
                        TokenType::Operator(op) => op,
                        _ => unreachable!(),
                    });
                    let (right, right_consumed) = parse_term(tokens, next_idx + 1)?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    };
                    consumed += 1 + right_consumed;
                }
                TokenType::Comparison(cmp) => {
                    let op = BinaryOp::from(cmp);
                    let (right, right_consumed) = parse_term(tokens, next_idx + 1)?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    };
                    consumed += 1 + right_consumed;
                }
                _ => {
                    println!("End of expression: {:?}", next_token);
                    break;
                }
            }
        } else {
            break;
        }
    }

    Ok((left, consumed))
}

fn parse_let(tokens: &[Token], current_token: &Token, idx: usize) -> Result<(Stmt, u8), Error> {
    let name_token = tokens.get(idx + 1).ok_or(Error::syntax_error(
        current_token,
        "identifier",
        "parse_let",
    ))?;
    let name = match &name_token.token {
        TokenType::Identifier(name) => name,
        _ => return Err(Error::syntax_error(name_token, "identifier", "parse_let")),
    };

    // expect equal sign
    let equal_token =
        tokens
            .get(idx + 2)
            .ok_or(Error::syntax_error(current_token, "=", "parse_let"))?;
    if equal_token.token != TokenType::Assign {
        return Err(Error::syntax_error(equal_token, "=", "parse_let"));
    }

    // value can be a value or an expression
    let value = parse_expr(tokens, idx + 3)?;

    let let_stmt = Stmt::Let {
        name: name.to_owned(),
        value: value.0,
    };
    Ok((let_stmt, value.1 + 3))
}

fn parse_if(tokens: &[Token], idx: usize) -> Result<(Stmt, u8), Error> {
    let condition = parse_expr(tokens, idx + 1)?;
    println!("condition: {:?}", condition);
    // expect {
    let open_brace_token = tokens
        .get(idx + condition.1 as usize + 1)
        .ok_or(Error::unexpected_eof("parse_if"))?;
    if open_brace_token.token != TokenType::BraceOpen {
        return Err(Error::syntax_error(open_brace_token, "{", "parse_if"));
    }

    // then
    let then_branch = parse_block(tokens, idx + condition.1 as usize + 2)?;

    let if_stmt = Stmt::If {
        condition: condition.0,
        then_branch: then_branch.0,
        else_branch: Option::None, // TODO
    };
    Ok((if_stmt, 2 + condition.1 + then_branch.1))
}

pub fn parse_block(tokens: &[Token], mut idx: usize) -> Result<(Vec<Stmt>, u8), Error> {
    let mut body = Vec::new();
    let initial_idx = idx;

    while idx < tokens.len() {
        let current_token = tokens.get(idx);
        if let Some(token) = current_token {
            println!("{:?}", token);

            // tokens to ignore
            if token.token == TokenType::Semicolon {
                idx += 1;
                continue;
            }

            // end of block
            if token.token == TokenType::BraceClose {
                return Ok((body, (idx - initial_idx + 1) as u8));
            }

            let stmt = match &token.token {
                TokenType::KWLet => parse_let(tokens, token, idx),
                TokenType::Identifier(name) => parse_identifier(tokens, name.to_owned(), idx),
                TokenType::KWFn => parse_fn(tokens, idx),
                TokenType::KWIf => parse_if(tokens, idx),
                TokenType::KWReturn => {
                    let value = parse_expr(tokens, idx + 1)?;

                    let return_stmt = Stmt::Return(value.0);
                    Ok((return_stmt, value.1 + 1))
                }
                _ => Err(Error::unimplemented_token(token, "parse_block")),
            }?;
            body.push(stmt.0);
            println!("idx: {} + {}", idx, stmt.1);
            idx += stmt.1 as usize;
        } else {
            break;
        }
    }
    Ok((body, (idx - initial_idx) as u8))
}

pub fn parse(tokens: &[Token]) -> Result<Program, Error> {
    let program = Program {
        statements: parse_block(tokens, 0)?.0,
    };

    Ok(program)
}
