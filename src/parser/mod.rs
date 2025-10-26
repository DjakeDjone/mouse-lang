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

fn parse_fn_call_params(tokens: &[Token], idx: usize) -> Result<(Vec<Expr>, u8), Error> {
    let mut params = Vec::new();
    let mut idx2 = idx;
    loop {
        let token = tokens
            .get(idx2)
            .ok_or(Error::unexpected_eof("parse_fn_call_params"))?;
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
                2 + params.1 + body.1 as u8,
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

fn parse_primitive(tokens: &[Token], idx: usize) -> Result<Expr, Error> {
    let token = tokens
        .get(idx)
        .ok_or(Error::unexpected_eof("parse_primitive"))?;
    match token.token.to_owned() {
        TokenType::Number(num) => Ok(Expr::Number(num)),
        TokenType::String(str) => Ok(Expr::String(str)),
        TokenType::Identifier(ident) => Ok(Expr::Identifier(ident)), // TODO: function calls
        _ => Err(Error::syntax_error(
            token,
            "number or string literal",
            "parse_primitive",
        )),
    }
}

/// expression
/// can be a number, string, function call, binary operation
/// returns the parsed expression and the number of tokens consumed
/// expression until the next token is not a binary operator
fn parse_expr(tokens: &[Token], idx: usize) -> Result<(Expr, u8), Error> {
    let token = tokens.get(idx).ok_or(Error::unexpected_eof("parse_expr"))?;
    let next_token_option = tokens.get(idx + 1);

    if let Some(next_token) = next_token_option {
        println!("Parse expr: Next token: {:?}", next_token);
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
            let args = parse_fn_call_params(tokens, idx + 2)?;
            let name: String = token.clone().token.into();
            return Ok((Expr::FunctionCall { name, args: args.0 }, 3 + args.1));
        }
    }

    // check for number or string literal
    Ok((parse_primitive(tokens, idx)?, 1))
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
    Ok((let_stmt, value.1 + 2))
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
                return Ok((body, 3 + (idx - initial_idx) as u8));
            }

            let stmt = match &token.token {
                TokenType::KWLet => parse_let(tokens, token, idx),
                TokenType::Identifier(name) => parse_identifier(tokens, name.to_owned(), idx),
                TokenType::KWFn => parse_fn(tokens, idx),
                TokenType::KWReturn => {
                    let value = parse_expr(tokens, idx + 1)?;

                    let return_stmt = Stmt::Return(value.0);
                    Ok((return_stmt, value.1 + 2))
                }
                _ => Err(Error::unimplemented_token(token, "parse_block")),
            }?;
            body.push(stmt.0);
            println!("idx: {} + {}", idx, stmt.1);
            if token.token == TokenType::KWReturn {
                return Ok((body, stmt.1 + 3));
            }
            idx += stmt.1 as usize;
        }

        idx += 1;
    }
    Ok((body, idx as u8))
}

pub fn parse(tokens: &[Token]) -> Result<Program, Error> {
    let program = Program {
        statements: parse_block(tokens, 0)?.0,
    };

    Ok(program)
}
