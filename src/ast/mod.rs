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
    RawExpr(Expr<N>),
    Assign(N, Expr<N>),
    If(Expr<N>, Vec<Stmt<N>>, Vec<Stmt<N>>),
    While(Expr<N>, Vec<Stmt<N>>),
    Continue,
    Break,
    Return(Expr<N>),
    Defn(Vec<N>, Vec<Stmt<N>>),
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Name {
    name: String,
    id: usize,
}

#[derive(Debug, PartialEq)]
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
        // If the variable is not temp, do nothing
        if (addr as usize) < self.vars.len() {
            return;
        }
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
            Var(ref name) => (self.vars[name], vec![]),
            Unop(ref op, ref arg) => unimplemented!(),
            Binop(ref op, ref left, ref right) => unimplemented!(),
            Call(ref func, ref args) => unimplemented!(),
            Index(ref tup, ref idx) => unimplemented!(),
            Mktup(ref parts) => unimplemented!(),
        }
    }

    pub fn compile_stmt(&mut self, stmt: &Stmt<Name>) -> Vec<Instr> {
        use self::Stmt::*;
        match *stmt {
            Declare(ref name) => {
                // @Todo: Should we have a separate method for this?
                let reg = self.push_tmp();
                self.vars.insert(name.clone(), reg);
                vec![]
            }
            RawExpr(ref expr) => {
                let (reg, code) = self.compile_expr(expr);
                self.pop_tmp(reg);
                code
            }
            Assign(ref name, ref expr) => unimplemented!(),
            If(ref cond, ref true_block, ref false_block) => unimplemented!(),
            While(ref cond, ref block) => unimplemented!(),
            Continue => unimplemented!(),
            Break => unimplemented!(),
            Return(ref expr) => unimplemented!(),
            Defn(ref params, ref body) => unimplemented!(),
        }
    }

    pub fn compile(&mut self, code: &[Stmt<Name>]) -> Vec<Instr> {
        let mut result = Vec::new();
        for stmt in code {
            result.append(&mut self.compile_stmt(stmt));
        }
        result
    }
}
