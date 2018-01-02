use std::collections::HashMap;
use bytecode::{Val, Instr, Addr};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum Expr<N> {
    Lit(Val),
    Var(N),
    Unop(Unop, Box<Expr<N>>),
    Binop(Binop, Box<Expr<N>>, Box<Expr<N>>),
    Call(Box<Expr<N>>, Vec<Expr<N>>),
    Index(Box<Expr<N>>, Box<Expr<N>>),
    Mktup(Vec<Expr<N>>),
}

#[derive(Debug)]
pub enum Stmt<N> {
    Declare(N),
    Assign(N, Expr<N>),
    If(Expr<N>, Box<Vec<Stmt<N>>>),
    While(Expr<N>, Box<Vec<Stmt<N>>>),
    For(Box<Stmt<N>>, Expr<N>, Box<Stmt<N>>, Vec<Stmt<N>>),
    Continue,
    Break,
    Return(Expr<N>),
    Defn(Vec<Stmt<N>>),
}

#[derive(Debug)]
pub enum Unop {
    Negate,
    Not,
}

#[derive(Debug)]
pub enum Binop {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Orr,
    Xor,
    Gt,
    Lt,
    Geq,
    Leq,
    Eq,
    Neq,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Name {
    name: String,
    id: usize,
}

#[derive(Debug)]
pub struct FunctionCtx {
    vars: HashMap<Name, Addr>,
    consts: Vec<Val>,
    free_reg: Addr,
    max_reg: Addr,
}

type Block<N> = Vec<Stmt<N>>;

impl FunctionCtx {
    fn push_tmp(&mut self) -> Addr {
        unimplemented!()
    }

    fn pop_tmp(&mut self, _addr: Addr) {
        unimplemented!()
    }
    /// Returns a tuple containing the register with the result of the expr
    /// and a Vect of Instrs that generate the expression
    pub fn compile_expr(&mut self, _expr: Expr<Name>) -> (Addr, Vec<Instr>) {
        unimplemented!()
    }

    pub fn compile_stmt(&mut self, _stmt: Stmt<Name>) -> Vec<Instr> {
        unimplemented!()
    }

    pub fn compile(&mut self, _code: Block<Name>) -> Vec<Instr> {
        unimplemented!()
    }
}
