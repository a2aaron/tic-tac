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

impl FunctionCtx {
    pub fn new() -> FunctionCtx {
        FunctionCtx {
            vars: HashMap::new(),
            consts: Vec::new(),
            free_reg: 0,
            max_reg: 0,
        }
    }

    fn push_tmp(&mut self) -> Addr {
        let reg = self.free_reg;
        self.free_reg += 1;
        if self.free_reg > self.max_reg {
            self.max_reg = self.free_reg;
        }
        reg
    }

    fn pop_tmp(&mut self, addr: Addr) {
        assert_eq!(self.free_reg, addr + 1);
        self.free_reg = addr;
    }

    fn get_const(&mut self, val: &Val) -> Addr {
        for (i, k) in self.consts.iter().enumerate() {
            if k == val {
                return i as u8;
            }
        }
        self.consts.push(val.clone());
        (self.consts.len() - 1) as u8
    }

    /// Returns a tuple containing the register with the result of the expr
    /// and a Vect of Instrs that generate the expression
    pub fn compile_expr(&mut self, expr: &Expr<Name>) -> (Addr, Vec<Instr>) {
        use self::Expr::*;
        match *expr {
            Lit(ref val) => {
                let reg = self.push_tmp();
                let instr = Instr::Const(reg, self.get_const(val));
                (reg, vec![instr])
            }
            Var(ref name) => unimplemented!(),
            Unop(ref op, ref arg) => unimplemented!(),
            Binop(ref op, ref left, ref right) => unimplemented!(),
            Call(ref func, ref args) => unimplemented!(),
            Index(ref tup, ref idx) => unimplemented!(),
            Mktup(ref parts) => unimplemented!(),
        }
    }

    pub fn compile_stmt(&mut self, _stmt: &Stmt<Name>) -> Vec<Instr> {
        unimplemented!()
    }

    pub fn compile(&mut self, _code: &[Stmt<Name>]) -> Vec<Instr> {
        unimplemented!()
    }
}
