use std::io::{Read, Write};

type Addr = u8;
type AddrSize = u8;
type FnId = u16;

#[derive(Debug)]
pub enum Instr {
    /// Loads a constant a = k[b]
    Const(Addr, Addr),
    /// a = b + c
    Add(Addr, Addr, Addr),
    /// a = b - c
    Sub(Addr, Addr, Addr),
    /// a = b * c
    Mul(Addr, Addr, Addr),
    /// a = b / c
    Div(Addr, Addr, Addr),
    /// a = b % c
    Rem(Addr, Addr, Addr),
    /// Copies a = b
    Copy(Addr, Addr),
    /// Constructs a tuple from a contiguous range of slots, a = (b..c)
    MkTup(Addr, Addr, Addr),
    /// Indexes a tuple a = b[c]
    IdxTup(Addr, Addr, Addr),
    /// Calls a function, a = b(c)
    /// This expects c to be a tuple of arguments to b, and b to be a function type
    Call(Addr, Addr, Addr),
    /// Return the value stored in a
    Return(Addr),
    /// Read a byte from stdin and store it in a
    Read(Addr),
    /// Write a byte stored in a to stdout
    Write(Addr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Val {
    B(bool),
    I(i64),
    F(f64),
    T(Vec<Val>),
    C(FnId),
}

pub struct Defn {
    consts: Vec<Val>,
    code: Vec<Instr>,
    local_count: AddrSize,
}

/// A piece of compiled code that's ready to be evaluated.
pub struct Program {
    defns: Vec<Defn>,
    entry_point: FnId,
}

/// Represents failures during execution.
///
/// Use it to get access to the cause, backtraces, etc.
#[derive(Debug, PartialEq)]
pub struct EvalError {}

impl Program {
    /// Evaluate a program with given I/O buffers.
    pub fn eval<R: Read, W: Write>(
        &self,
        _input: &mut R,
        _output: &mut W,
    ) -> Result<Val, EvalError> {
        use Val::*;
        use Instr::*;

        let mut stack = Vec::new();
        let mut code = &self.defns[self.entry_point as usize];
        let mut locals = vec![I(0); code.local_count as usize];
        let mut iptr = 0isize;
        loop {
            match code.code[iptr as usize] {
                Const(a, k) => locals[a as usize] = code.consts[k as usize].clone(),
                Add(a, b, c) => {
                    match (&locals[b as usize], &locals[c as usize]) {
                        (&I(b), &I(c)) => locals[a as usize] = I(b + c),
                        (&F(b), &F(c)) => locals[a as usize] = F(b + c),
                        _ => unimplemented!(),
                    }
                }
                MkTup(a, b, c) => locals[a as usize] = T(locals[b as usize..c as usize + 1].into()),
                IdxTup(a, t, i) => {
                    locals[a as usize] = match (&locals[t as usize], &locals[i as usize]) {
                        (&T(ref t), &I(i)) => Ok(t[i as usize].clone()),
                        _ => unimplemented!(),
                    }?;
                }
                Call(a, f, c) => {
                    let new_code = &self.defns[f as usize];
                    let mut new_locals = vec![I(0); new_code.local_count as usize];
                    new_locals[0] = locals[c as usize].clone();
                    stack.push((a, code, locals, iptr));
                    code = new_code;
                    locals = new_locals;
                    iptr = -1;
                }
                Return(a) => {
                    if let Some((addr, new_code, mut new_locals, new_iptr)) = stack.pop() {
                        new_locals[addr as usize] = locals.remove(a as usize);
                        locals = new_locals;
                        code = new_code;
                        iptr = new_iptr;
                    } else {
                        return Ok(locals.remove(a as usize));
                    }
                }
                _ => unimplemented!(),
            }
            iptr += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        use Val::*;
        use Instr::*;

        let program = Program {
            defns: vec![
                Defn {
                    code: vec![
                        Const(0, 0),
                        Const(1, 1),
                        MkTup(0, 0, 1),
                        Call(0, 1, 0),
                        Return(0),
                    ],
                    consts: vec![I(42), I(69)],
                    local_count: 2,
                },
                Defn {
                    code: vec![
                        Const(1, 0),
                        IdxTup(1, 0, 1),
                        Const(2, 1),
                        IdxTup(2, 0, 2),
                        Add(0, 1, 2),
                        Return(0),
                    ],
                    consts: vec![I(0), I(1)],
                    local_count: 3,
                },
            ],
            entry_point: 0,
        };

        assert_eq!(
            program.eval(&mut std::io::empty(), &mut std::io::sink()),
            Ok(I(111))
        );
    }
}
