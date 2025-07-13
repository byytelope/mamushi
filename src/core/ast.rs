use crate::core::token::{LiteralValue, TokenType};

#[derive(Debug)]
#[allow(dead_code)]
pub enum Stmt {
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    ClassDef {
        name: String,
        base: Option<Expr>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Expression(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Print(Expr),
    Assign {
        target: Target,
        value: Expr,
    },
    For {
        target: Target,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    Import(Vec<String>),
    FromImport {
        module: String,
        names: Vec<String>,
    },
    Global(Vec<String>),
    Try {
        body: Vec<Stmt>,
        except_clauses: Vec<(Option<Expr>, Vec<Stmt>)>,
    },
    Raise(Option<Expr>),
    Del(Target),
    Pass,
    Break,
    Continue,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Expr {
    Literal(LiteralValue),
    Variable(String),
    Unary {
        op: TokenType,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: TokenType,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Tuple(Vec<Expr>),
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Get {
        object: Box<Expr>,
        name: String,
    },
    Set {
        object: Box<Expr>,
        name: String,
        value: Box<Expr>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Target {
    Name(String),
    Tuple(Vec<Target>),
    Attribute { object: Box<Expr>, name: String },
}
