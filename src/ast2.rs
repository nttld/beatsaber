use crate::ast1::{Expr, Behaviour, AssignTarget, AssignValue, Op, Parser1};
use std::collections::HashMap;
use logos::Span;

// Lets do something useful with it; like binding identifiers to operations
// CallExprs
// FunctionBlocks

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Identifier {
    pub id: usize
}

#[derive(Clone, Debug)]
pub struct CallExpr {
    pub function: Identifier,
    pub p1: Box<DecoratedExpr>,
    pub p2: Option<Box<DecoratedExpr>>
}

#[derive(Clone, Debug)]
pub enum DecoratedExpr {
    CallExpr(CallExpr),
    Identifier(Identifier)
}

#[derive(Clone, Debug)]
pub struct FuncBlock {
    // Function declaration
    pub decl: FuncDecl,
    pub block: Vec<DecoratedStmt>
}

#[derive(Clone, Debug)]
pub struct LoadLiteralNumber {
    pub line: usize,
    pub ident: Identifier,
    pub value: usize
}

#[derive(Clone, Debug)]
pub struct FuncDecl {
    pub line: usize,
    pub id: Identifier,
    pub p1: Identifier,
    pub p2: Option<Identifier>
}

// n.b // if a return is add
#[derive(Clone, Debug)]
pub struct Conditional {
    pub condition: Identifier,
    pub success: Box<DecoratedStmt>
}

#[derive(Clone, Debug)]
pub struct Assignment {
    pub line: usize,
    pub name: Option<Identifier>,
    pub value: DecoratedExpr
}

#[derive(Clone, Debug)]
pub struct ExternFunction {
    pub line: usize,
    pub name: String,
    pub ident: Identifier
}

#[derive(Clone, Debug)]
pub struct ReturnStmt {
    pub line: usize,
    pub expr: DecoratedExpr
}

#[derive(Clone, Debug)]
pub struct GotoStmt {
    pub line: usize,
    pub target: DecoratedExpr
}

#[derive(Clone, Debug)]
pub enum Callable {
    ExternFunction(ExternFunction),
    FuncBlock(FuncBlock),
}

#[derive(Clone, Debug)]
pub enum DecoratedStmt {
    LoadLiteralNumber(LoadLiteralNumber),
    Callable(Callable),
    Conditional(Conditional),
    Assignment(Assignment),
    ReturnStmt(ReturnStmt),
    GotoStmt(GotoStmt)
}

impl DecoratedStmt {
    pub fn line_number(&self) -> usize {
        match self {
            DecoratedStmt::LoadLiteralNumber(stmt) => stmt.line,
            DecoratedStmt::Callable(c) => match c {
                Callable::FuncBlock(FuncBlock { decl, .. }) => decl.line,
                Callable::ExternFunction(ExternFunction { line, .. }) => *line,
            },
            DecoratedStmt::Conditional(stmt) => stmt.success.line_number(),
            DecoratedStmt::Assignment(stmt) => stmt.line,
            DecoratedStmt::ReturnStmt(stmt) => stmt.line,
            DecoratedStmt::GotoStmt(stmt) => stmt.line,
        }
    }
}

fn create_identifier<'a>(ids: &mut HashMap<&'a str, Identifier>, id: Option<&'a str>) -> Identifier {
    let identifier = id.unwrap();
    if ids.contains_key(identifier) {
        panic!("Cannot create a new identifier for name: '{}' because it already exists!", identifier);
    }
    let ident = Identifier { id: ids.len() };
    ids.insert(identifier, ident);
    ident
}

fn create_or_shadow_ident_opt<'a>(ids: &mut HashMap<&'a str, Identifier>, src: &'a str, id: Option<&'a str>) -> Option<Identifier> {
    let len = ids.len();
    id.map(|id| ids.entry(id).or_insert_with(|| Identifier { id: len }).clone())
}

fn create_or_shadow_ident<'a>(ids: &mut HashMap<&'a str, Identifier>, src: &'a str, id: Span) -> Identifier {
    let len = ids.len();
    ids.entry(&src[id]).or_insert_with(|| Identifier { id: len }).clone()
}

fn parse_behaviour<'a>(line: usize, behaviour: Behaviour, expr: Option<Expr>, ids: &mut HashMap<&'a str, Identifier>, funcIds: &mut HashMap<Identifier, Callable>, src: &'a str) -> Option<DecoratedStmt> {
    match behaviour {
        Behaviour::Assign { target, value, .. } => {
            match target {
                AssignTarget::Ident(_) | AssignTarget::Discard(_) => {
                    let id = match target {
                        AssignTarget::Ident(span) => {
                            Some(&src[span])
                        },
                        _ => None,
                    };
                    match value {
                        AssignValue::Number(_, n) => {
                            // Must have an identifier for loading literals
                            Some(DecoratedStmt::LoadLiteralNumber(LoadLiteralNumber {line, ident: create_identifier(ids, id), value: n}))
                        }
                        AssignValue::NotHere(_) => {
                            // Must have an identifier for exported functions
                            let ident = create_identifier(ids, id);
                            funcIds.insert(ident, Callable::ExternFunction(ExternFunction {line, name: id.unwrap().to_owned(), ident}));
                            None
                        }
                        AssignValue::Ops(ops) => {
                            // Made up of CallExprs
                            // Each op needs to match the expression op
                            let ident = create_or_shadow_ident_opt(ids, src, id);
                            
                            Some(DecoratedStmt::Assignment(Assignment {
                                line,
                                name: ident,
                                value: zip_ops_with_expr(&expr.unwrap(), &ops, ids, src)
                            }))
                        }
                        AssignValue::Fn(f) => {
                            // Create a function declaration for this, make a function definition for this, add to function collection
                            // All functions will be added to the output vector before being returned
                            let ident = create_identifier(ids, id);
                            let p1 = create_or_shadow_ident(ids, src, f.params.p1);
                            let p2 = create_or_shadow_ident_opt(ids, src, f.params.p2.map(|v| &src[v]));
                            let mut block = FuncBlock {
                                decl: FuncDecl {
                                    line,
                                    id: ident,
                                    p1,
                                    p2
                                },
                                block: Vec::new()
                            };
                            if expr.is_some() {
                                block.block.push(DecoratedStmt::Assignment(Assignment {line, name: None, value: zip_ops_with_expr(&expr.unwrap(), &f.ops, ids, src)}));
                            }
                            funcIds.insert(ident, Callable::FuncBlock(block));
                            // This should NOT be added right away, since it will not hold all of the potential blocks.
                            // See funcIds instead.
                            None
                        }
                    }
                }
                AssignTarget::Goto(_) => {
                    if expr.is_none() {
                        panic!("Goto statement on line: {} cannot have no expression!", line);
                    }
                    match value {
                        AssignValue::Ops(ops) => {
                            Some(DecoratedStmt::GotoStmt(GotoStmt {line, target: zip_ops_with_expr(&expr.unwrap(), &ops, ids, src)}))
                        }
                        _ => panic!("Goto statement on line: {} must have operators only!", line),
                    }
                }
                AssignTarget::Return(_) => {
                    if expr.is_none() {
                        panic!("Return statement on line: {} cannot have no expression!", line);
                    }
                    match value {
                        AssignValue::Ops(ops) => {
                            Some(DecoratedStmt::ReturnStmt(ReturnStmt {line, expr: zip_ops_with_expr(&expr.unwrap(), &ops, ids, src)}))
                        }
                        _ => panic!("Return statement on line: {} must have operators only!", line),
                    }
                }
            }
        }
        Behaviour::StillIn { ident, behaviour, still_in } => {
            // Recursive parse behaviour
            // Map ident to correct function
            let identStr = &src[ident];
            let func = ids.get(identStr).cloned();
            if func.is_none() {
                panic!("'{}' is not a valid identifier for '{}'!", identStr, &src[still_in]);
            }
            let ret = parse_behaviour(line, *behaviour, expr, ids, funcIds, src).unwrap();
            let mut body = funcIds.get_mut(&func.unwrap());
            if body.is_none() {
                panic!("'{}' is not a function!", identStr);
            }
            match body.unwrap() {
                Callable::FuncBlock(FuncBlock { block, .. }) => block.push(ret.clone()),
                Callable::ExternFunction(ExternFunction { name, .. }) => panic!("Cannot be within an exported function: {}", name),
            }
            Some(ret)
        }
        Behaviour::Cond { cond, behaviour, if_ } => {
            // Recursive parse behaviour
            // Map cond to identifier
            let identStr = &src[cond];
            let ident = ids.get(identStr).cloned();
            if ident.is_none() {
                panic!("'{}' is not a valid identifier for '{}'!", identStr, &src[if_]);
            }
            parse_behaviour(line, *behaviour, expr, ids, funcIds, src)
        }
    }
}

pub fn parse(stmts: Parser1, src: &str) -> Vec<DecoratedStmt> {
    let mut outp = Vec::new();
    let mut ids = HashMap::new();
    let mut funcIds = HashMap::new();
    for stmt in stmts {
        let val = parse_behaviour(stmt.line, stmt.behaviour, stmt.expr, &mut ids, &mut funcIds, src);
        if val.is_some() {
            outp.push(val.unwrap());
        }
    }
    outp.extend(funcIds.into_values().map(|v| DecoratedStmt::Callable(v)));
    outp
}

fn zip_ops_with_expr<'a>(expr: &Expr, ops: &[Op], ids: &mut HashMap<&'a str, Identifier>, src: &'a str) -> DecoratedExpr {
    fn inner<'a, 'ops>(expr: &Expr, ops: &'ops [Op], ids: &mut HashMap<&'a str, Identifier>, src: &'a str) -> (DecoratedExpr, &'ops [Op]) {
        match expr {
            Expr::Binop { lhs, rhs, .. } => {
                let (lhs, ops) = inner(lhs, ops, ids, src);
                let (rhs, ops) = inner(rhs, ops, ids, src);
                let (op, ops) = ops.split_first().unwrap();

                let keyString = &src[op.ident.clone()];
                let ident = ids.get(keyString).unwrap_or_else(|| panic!("Could not find identifier for binary op: '{}'", keyString));
                (DecoratedExpr::CallExpr(CallExpr {
                    function: ident.clone(),
                    p1: Box::new(lhs),
                    p2: Some(Box::new(rhs)),
                }), ops)
            }
            Expr::Unop { expr, .. } => {
                let (expr, ops) = inner(expr, ops, ids, src);
                let (op, ops) = ops.split_first().unwrap();
                let keyString = &src[op.ident.clone()];
                let ident = ids.get(keyString).unwrap_or_else(|| panic!("Could not find identifier for unary op: '{}'", keyString));
                (DecoratedExpr::CallExpr(CallExpr {
                    function: ident.clone(),
                    p1: Box::new(expr),
                    p2: None,
                }), ops)
            }
            Expr::Paren { expr, .. } => inner(expr, ops, ids, src),
            Expr::Ident(span) => {
                let keyString = &src[span.clone()];
                let ident = ids.get(keyString).unwrap_or_else(|| panic!("Could not find identifier for single identifier: '{}'", keyString));
                (DecoratedExpr::Identifier(ident.clone()), ops)
            }
        }
    }

    let (expr, ops) = inner(expr, ops, ids, src);
    if !ops.is_empty() {
        panic!("Too many operations for the expression provided!");
    }
    expr
}

// fn parse(stmts: Vec<Stmt>) -> Vec<DecoratedAst> {
//     // Walk this vector
//     // Create funcblocks/calls as needed
//     // Tada!

// }