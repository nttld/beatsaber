use logos::Span;

use crate::lexer::{Lexer, Token};

#[derive(Debug)]
pub struct Stmt {
    pub line: usize,
    pub expr: Option<Expr>,
    pub sep: Span,
    pub behaviour: Behaviour,
}

#[derive(Debug)]
pub enum Expr {
    Binop {
        lhs: Box<Expr>,
        op: Span,
        rhs: Box<Expr>,
    },
    Unop {
        expr: Box<Expr>,
        op: Span,
    },
    Paren {
        l: Span,
        expr: Box<Expr>,
        r: Span,
    },
    Ident(Span),
}

#[derive(Debug)]
pub enum Behaviour {
    StillIn {
        still_in: Span,
        ident: Span,
        behaviour: Box<Behaviour>,
    },
    Cond {
        if_: Span,
        cond: Span,
        behaviour: Box<Behaviour>,
    },
    Assign {
        target: AssignTarget,
        is: Span,
        value: AssignValue,
    },
}

#[derive(Debug)]
pub enum AssignTarget {
    Discard(Span),
    Return(Span),
    Goto(Span),
    Ident(Span),
}

#[derive(Debug)]
pub enum AssignValue {
    Ops(Vec<Op>),
    Fn(Fn),
    Number(Span, usize),
    NotHere(Span),
}

#[derive(Debug)]
pub struct Op {
    pub ident: Span,
    pub then: Option<Span>,
}

#[derive(Debug)]
pub struct Fn {
    pub with: Span,
    pub params: FnParams,
    pub ops: Vec<Op>,
}

#[derive(Debug)]
pub struct FnParams {
    pub p1: Span,
    pub and: Option<Span>,
    pub p2: Option<Span>,
}

pub fn parse(mut tokens: Lexer) -> Vec<Stmt> {
    let mut stmts = Vec::new();
    let mut line = 1;
    while let Some((token, span)) = tokens.peek() {
        match token {
            Token::Newline => {
                tokens.next();
            },
            Token::BehaviourStart => {
                tokens.next();
                let sep = span;
                let behaviour = parse_behaviour(&mut tokens);
                stmts.push(Stmt {
                    line,
                    expr: None,
                    sep,
                    behaviour,
                });
            },
            Token::Identifier | Token::ParenLeft => {
                let expr = parse_expr(&mut tokens);
                let sep = tokens.monch(Token::BehaviourStart);
                let behaviour = parse_behaviour(&mut tokens);
                stmts.push(Stmt {
                    line,
                    expr: Some(expr),
                    sep,
                    behaviour,
                });
            },
            _ => panic!("sorry can't put that here"),
        };
        line += 1;
    }
    stmts
}

fn parse_expr(tokens: &mut Lexer) -> Expr {
    fn parse_expr_inner(expr: Expr, tokens: &mut Lexer) -> Expr {
        let (token, op) = tokens.peek().unwrap();
        match token {
            Token::Operator => {
                tokens.next();
                match tokens.peek().unwrap() {
                    (Token::Operator, _) => parse_expr_inner(Expr::Unop { expr: Box::new(expr), op }, tokens),
                    (Token::ParenRight | Token::BehaviourStart, _) => Expr::Unop { expr: Box::new(expr), op },
                    _ => Expr::Binop { lhs: Box::new(expr), op, rhs: Box::new(parse_expr(tokens)) },
                }
            },
            _ => dbg!(expr),
        }
    }

    match tokens.next().unwrap() {
        (Token::Identifier, ident) => parse_expr_inner(Expr::Ident(ident), tokens),
        (Token::ParenLeft, l) => {
            let expr = parse_expr(tokens);
            let r = tokens.monch(Token::ParenRight);
            parse_expr_inner(Expr::Paren {
                l,
                expr: Box::new(expr),
                r,
            }, tokens)
        }
        t => panic!("{:?}", t),
    }
}

fn parse_behaviour(tokens: &mut Lexer) -> Behaviour {
    let (token, span) = tokens.next().unwrap();
    match token {
        Token::StillIn => {
            let still_in = span;
            let ident = tokens.monch(Token::Identifier);
            let behaviour = parse_behaviour(tokens);
            Behaviour::StillIn {
                still_in,
                ident,
                behaviour: Box::new(behaviour),
            }
        }
        Token::If => {
            let if_ = span;
            let cond = tokens.monch(Token::Identifier);
            let behaviour = parse_behaviour(tokens);
            Behaviour::Cond {
                if_,
                cond,
                behaviour: Box::new(behaviour),
            }
        }
        Token::Identifier | Token::Discard | Token::Return | Token::Goto => {
            let target = match token {
                Token::Identifier => AssignTarget::Ident(span),
                Token::Discard => AssignTarget::Discard(span),
                Token::Return => AssignTarget::Return(span),
                Token::Goto => AssignTarget::Goto(span),
                _ => unreachable!(),
            };
            let is = tokens.monch(Token::Is);
            let value = parse_assign_value(tokens);
            Behaviour::Assign {
                target,
                is,
                value,
            }
        }
        _ => panic!()
    }
}

fn parse_assign_value(tokens: &mut Lexer) -> AssignValue {
    let (token, span) = tokens.peek().unwrap();
    let value = match token {
        Token::Identifier => return AssignValue::Ops(parse_ops(tokens)),
        Token::With => return AssignValue::Fn(parse_fn(tokens)),
        Token::Number(n) => AssignValue::Number(span, n),
        Token::NotHere => AssignValue::NotHere(span),
        _ => panic!(),
    };
    tokens.next();
    value
}

fn parse_fn(tokens: &mut Lexer) -> Fn {
    let (_, with) = tokens.next().unwrap();
    let p1 = tokens.monch(Token::Identifier);
    let (and, p2) = match tokens.peek() {
        Some((Token::And, and)) => {
            tokens.next();
            let p2 = tokens.monch(Token::Identifier);
            (Some(and), Some(p2))
        }
        _ => (None, None),
    };
    let ops = parse_ops(tokens);
    Fn { with, params: FnParams {p1, and, p2}, ops }
}

fn parse_ops(tokens: &mut Lexer) -> Vec<Op> {
    let mut ops = Vec::new();
    while let Some((token, span)) = tokens.next() {
        match token {
            Token::Identifier => {
                let ident = span;
                let then = match tokens.peek() {
                    Some((Token::Then, span)) => {
                        tokens.next();
                        Some(span)
                    },
                    _ => None,
                };
                ops.push(Op { ident, then: then.clone() });

                if then.is_none() {
                    break;
                }
            }
            Token::Newline => break,
            _ => panic!("expected ident or newline"),
        }
    }
    ops
}
