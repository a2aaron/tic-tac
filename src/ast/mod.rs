use bytecode::{Val, Instr};

#[cfg(test)]
mod tests;

#[derive(Debug)]
enum Expr<N> {
    Lit(Val),
    Var(N),
    Unop(Unop, Box<Expr<N>>),
    Binop(Binop, Box<Expr<N>>, Box<Expr<N>>),
    Call(Box<Expr<N>>, Vec<Expr<N>>),
    Index(Box<Expr<N>>, Box<Expr<N>>),
    Mktup(Vec<Expr<N>>),
}

#[derive(Debug)]
enum Stmt {
    Declare(N),
    Assign(N, Expr<N>),
    If(Expr<N>, Box<Vec<Stmt>>),
    While(Expr<N>, Box<Vec<Stmt>>),
    For(Box<Stmt>, Expr<N>, Box<Stmt>, Vec<Stmt>),
    Continue,
    Break,
    Return(Expr<N>),
    Defn(Vec<Stmt>),
}

fn compile(statement: Stmt) -> Vec<Instr> {
    unimplemented!()

}

#[derive(Debug)]
enum Unop {
    Negate,
    Not,
}

#[derive(Debug)]
enum Binop {
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

#[derive(Debug)]
struct N {
    name: String,
    // register: usize,
}
