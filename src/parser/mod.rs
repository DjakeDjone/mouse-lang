use crate::lexer::{Operator, Token};
use chumsky::prelude::*;

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
    Print(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
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
            // Identifiers (potentially with member access or function calls)
            select! { Token::Identifier(name) => Expr::Identifier(name) },
            // Parenthesized expressions
            expr.clone()
                .delimited_by(just(Token::BracketOpen), just(Token::BracketClose)),
        ));

        // Handle member access and function calls (left-to-right)
        let member_or_call = primary.clone().foldl(
            choice((
                // Member access: obj.member
                just(Token::Dot)
                    .ignore_then(select! { Token::Identifier(member) => member })
                    .map(|member| (false, member, vec![])),
                // Function call: func(args)
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .collect()
                    .delimited_by(just(Token::BracketOpen), just(Token::BracketClose))
                    .map(|args| (true, String::new(), args)),
            ))
            .repeated(),
            |left, (is_call, member, args)| {
                if is_call {
                    // This is a function call
                    match left {
                        Expr::Identifier(name) => Expr::FunctionCall { name, args },
                        Expr::MemberAccess {
                            object,
                            member: method,
                        } => {
                            // For member function calls like std.socketServer()
                            // Create a special identifier format
                            let full_name = match *object {
                                Expr::Identifier(obj_name) => format!("{}.{}", obj_name, method),
                                _ => method,
                            };
                            Expr::FunctionCall {
                                name: full_name,
                                args,
                            }
                        }
                        _ => left, // Error case, but we'll handle it
                    }
                } else {
                    // This is member access
                    Expr::MemberAccess {
                        object: Box::new(left),
                        member,
                    }
                }
            },
        );

        // binary operation
        member_or_call.clone().foldl(
            choice((
                select! { Token::Operator(op) => BinaryOp::from(op) },
                select! { Token::Equal => BinaryOp::Equal },
                select! { Token::NotEqual => BinaryOp::NotEqual },
                select! { Token::LessThan => BinaryOp::LessThan },
                select! { Token::LessThanOrEqual => BinaryOp::LessThanOrEqual },
                select! { Token::GreaterThan => BinaryOp::GreaterThan },
                select! { Token::GreaterThanOrEqual => BinaryOp::GreaterThanOrEqual },
            ))
            .then(member_or_call.clone())
            .repeated(),
            |left, (op, right)| Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
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

        let assign_stmt = select! { Token::Identifier(name) => name }
            .then_ignore(just(Token::Assign))
            .then(expr_parser.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(name, value)| Stmt::Assign { name, value });

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
                    .delimited_by(just(Token::BracketOpen), just(Token::BracketClose)),
            )
            .then(
                stmt.clone()
                    .repeated()
                    .collect()
                    .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
            )
            .map(|((name, params), body)| Stmt::Function { name, params, body });

        let if_stmt = just(Token::KWIf)
            .ignore_then(expr_parser.clone())
            .then(
                stmt.clone()
                    .repeated()
                    .collect()
                    .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
            )
            .map(|(condition, then_branch)| Stmt::If {
                condition,
                then_branch,
                else_branch: None,
            });

        let while_stmt = just(Token::KWWhile)
            .ignore_then(expr_parser.clone())
            .then(
                stmt.clone()
                    .repeated()
                    .collect()
                    .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
            )
            .map(|(condition, body)| Stmt::While { condition, body });

        let expr_stmt = expr_parser
            .clone()
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Expression);

        // Main statement parser
        choice((
            let_stmt,
            assign_stmt,
            function_stmt,
            return_stmt,
            print_stmt,
            if_stmt,
            while_stmt,
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
