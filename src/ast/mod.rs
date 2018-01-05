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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unop {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Name {
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
            Unop(op, ref arg) => {
                let reg = self.push_tmp();
                let (arg_dest, mut arg_code) = self.compile_expr(arg);
                use self::Unop::*;
                let instr = match op {
                    Negate => Instr::Neg,
                    Not => Instr::Not,
                }(reg, arg_dest);
                self.pop_tmp(arg_dest);

                arg_code.push(instr);
                (reg, arg_code)
            },
            Binop(op, ref left, ref right) => {
                let reg = self.push_tmp();
                let (left_dest, mut left_code) = self.compile_expr(left);
                let (right_dest, mut right_code) = self.compile_expr(right);
                use self::Binop::*;
                let instr = match op {
                    Add => Instr::Add,
                    Sub => Instr::Sub,
                    Mul => Instr::Mul,
                    Div => Instr::Div,
                    Rem => Instr::Rem,
                    And => Instr::And,
                    Orr => Instr::Orr,
                    Xor => Instr::Xor,
                    Gt => Instr::Gt,
                    Lt => Instr::Lt,
                    Geq => Instr::Geq,
                    Leq => Instr::Leq,
                    Eq => Instr::Eq,
                    Neq => Instr::Neq,
                }(reg, left_dest, right_dest);

                self.pop_tmp(right_dest);
                self.pop_tmp(left_dest);

                left_code.append(&mut right_code);
                left_code.push(instr);
                (reg, left_code)
            }
            Call(ref func, ref args) => unimplemented!(),
            Index(ref tup, ref idx) => {
                let reg = self.push_tmp();

                let (tup_dest, mut tup_code) = self.compile_expr(tup);
                let (idx_dest, mut idx_code) = self.compile_expr(idx);

                tup_code.append(&mut idx_code);
                tup_code.push(Instr::IdxTup(reg, tup_dest, idx_dest));

                self.pop_tmp(idx_dest);
                self.pop_tmp(tup_dest);
                (reg, tup_code)
            },
            Mktup(ref parts) => {
                let reg = self.push_tmp();
                let mut code = vec![];
                let mut part_addrs = vec![];
                // @TODO: any way to make start_addr not mutable?
                for part in parts.iter() {
                    let (part_dest, mut part_code) = self.compile_expr(part);
                    code.append(&mut part_code);
                    part_addrs.push(part_dest);
                }

                let start_addr = *part_addrs.first().unwrap_or(&0);

                // Must do this backwards due to the highest registers being popped first
                for addr in part_addrs.iter().rev() {
                    self.pop_tmp(*addr);
                }
                code.push(Instr::MkTup(reg, start_addr, parts.len() as u8));

                (reg, code)
            },
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
            Assign(ref name, ref expr) => {
                let dest = self.vars[name];
                let (reg, mut code) = self.compile_expr(expr);
                code.push(Instr::Copy(dest, reg));

                self.pop_tmp(reg);

                code
            }
            If(ref cond, ref true_block, ref false_block) => {
                use bytecode::Instr::*;

                let (cond_dest, mut code) = self.compile_expr(cond);
                // @TODO: improve short /long jump code
                code.push(CondJump(cond_dest, 2, 1));
                self.pop_tmp(cond_dest);

                let mut true_code = self.compile(true_block);
                let mut false_code = self.compile(false_block);

                code.push(Jump(true_code.len() as i16 + 2));
                code.append(&mut true_code);
                code.push(Jump(false_code.len() as i16 + 1));
                code.append(&mut false_code);
                code
            },
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
