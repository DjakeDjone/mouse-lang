use chumsky::prelude::*;
use crate::lexer::{Token, Operator};

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Expr),
    Print(Expr),
    Expression(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl From<Operator> for BinaryOp {
    fn from(op: Operator) -> Self {
        match op {
            Operator::Add => BinaryOp::Add,
            Operator::Subtract => BinaryOp::Subtract,
            Operator::Multiply => BinaryOp::Multiply,
            Operator::Divide => BinaryOp::Divide,
        }
    }
}

pub fn parser<'src>() -> impl Parser<'src, &'src [Token], Program, extra::Err<Rich<'src, Token>>> {
    let expr_parser = recursive(|expr| {
        let primary = choice((
            // Numbers
            select! { Token::Number(n) => Expr::Number(n) },
            
            // Strings
            select! { Token::String(s) => Expr::String(s) },
            
            // Function calls
            select! { Token::Identifier(name) => name }
                .then(
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .collect()
                        .delimited_by(just(Token::BracketOpen), just(Token::BracketClose))
                )
                .map(|(name, args)| Expr::FunctionCall { name, args }),
            
            // Identifiers
            select! { Token::Identifier(name) => Expr::Identifier(name) },
            
            // Parenthesized expressions
            expr.clone()
                .delimited_by(just(Token::BracketOpen), just(Token::BracketClose)),
        ));

        // binary operations
        primary.clone()
            .foldl(
                select! { Token::Operator(op) => BinaryOp::from(op) }
                    .then(primary.clone())
                    .repeated(),
                |left, (op, right)| Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            )
    });

    // Statement parsers - use recursive to handle function bodies
    recursive(|stmt| {
        let let_stmt = just(Token::KWLet)
            .ignore_then(select! { Token::Identifier(name) => name })
            .then_ignore(just(Token::Assign))
            .then(expr_parser.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(name, value)| Stmt::Let { name, value });

        let return_stmt = just(Token::KWReturn)
            .ignore_then(expr_parser.clone())
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Return);

        let print_stmt = select! { Token::Identifier(s) if s == "print" => s }
            .ignore_then(just(Token::BracketOpen))
            .ignore_then(expr_parser.clone())
            .then_ignore(just(Token::BracketClose))
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Print);

        let function_stmt = just(Token::KWFn)
            .ignore_then(select! { Token::Identifier(name) => name })
            .then(
                select! { Token::Identifier(param) => param }
                    .separated_by(just(Token::Comma))
                    .collect()
                    .delimited_by(just(Token::BracketOpen), just(Token::BracketClose))
            )
            .then(
                stmt.clone()
                    .repeated()
                    .collect()
                    .delimited_by(just(Token::BraceOpen), just(Token::BraceClose))
            )
            .map(|((name, params), body)| Stmt::Function { name, params, body });

        let expr_stmt = expr_parser.clone()
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Expression);

        // Main statement parser
        choice((
            let_stmt,
            function_stmt,
            return_stmt, 
            print_stmt,
            expr_stmt,
        ))
    })
    // Program parser
    .repeated()
    .collect()
    .then_ignore(end())
    .map(|statements| Program { statements })
}

pub fn parse(tokens: &[Token]) -> Result<Program, Vec<Rich<'_, Token>>> {
    parser().parse(tokens).into_result()
}
