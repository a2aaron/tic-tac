#[cfg(test)]
mod tests;
pub mod parse;

use std::fmt;
use std::io::{Read, Write};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Sub};
use std::cmp::{Ordering, PartialOrd};

type Addr = u8;
type AddrSize = u8;
type FnId = u16;

#[derive(Debug, PartialEq, Eq)]
pub enum Instr {
    /// Loads a constant a = k[b]
    Const(Addr, Addr),
    /// Copies a = b
    Copy(Addr, Addr),
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
    /// a = b & c
    ///
    /// This acts as a boolean and as well as a bitwise and.
    And(Addr, Addr, Addr),
    /// a = b | c
    ///
    /// This acts as a boolean or as well as a bitwise or.
    Orr(Addr, Addr, Addr),
    /// a = b ^ c
    Xor(Addr, Addr, Addr),
    /// a = b == c
    Eq(Addr, Addr, Addr),
    /// a = b != c
    Neq(Addr, Addr, Addr),
    /// a = b < c
    Lt(Addr, Addr, Addr),
    /// a = b > c
    Gt(Addr, Addr, Addr),
    /// a = b <= c
    Leq(Addr, Addr, Addr),
    /// a = b >= c
    Geq(Addr, Addr, Addr),
    /// Jumps program execution by n instructions
    Jump(i16),
    /// Jumps program execution by n instructions if a is true, else it jumps by m instructions
    /// Note that a must be a boolean, otherwise the program is invalid.
    CondJump(Addr, i8, i8),
    /// Constructs a tuple, a = (b; c)
    /// Takes a contiguous range of c slots starting at b, a = (_; 0) builds the empty tuple.
    MkTup(Addr, Addr, u8),
    /// Indexes a tuple a = b[c]
    IdxTup(Addr, Addr, Addr),
    /// Calls a function, a = b(c).
    /// This expects c to be a tuple of arguments to b, and b to be a function type.
    Call(Addr, Addr, Addr),
    /// Return the value stored in a.
    /// If a is None, then this returns an empty tuple.
    Return(Option<Addr>),
    /// Read a byte from stdin and store it in a
    Read(Addr),
    /// Write a byte stored in a to stdout
    Write(Addr),
}

impl fmt::Display for Instr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::Instr::*;
        match *self {
            Const(a, b) => write!(fmt, "x{} := k{}", a, b),
            Copy(a, b) => write!(fmt, "x{} := x{}", a, b),
            Add(a, b, c) => write!(fmt, "x{} := x{} + x{}", a, b, c),
            Sub(a, b, c) => write!(fmt, "x{} := x{} - x{}", a, b, c),
            Mul(a, b, c) => write!(fmt, "x{} := x{} * x{}", a, b, c),
            Div(a, b, c) => write!(fmt, "x{} := x{} / x{}", a, b, c),
            Rem(a, b, c) => write!(fmt, "x{} := x{} % x{}", a, b, c),
            And(a, b, c) => write!(fmt, "x{} := x{} & x{}", a, b, c),
            Orr(a, b, c) => write!(fmt, "x{} := x{} | x{}", a, b, c),
            Xor(a, b, c) => write!(fmt, "x{} := x{} ^ x{}", a, b, c),
            Eq(a, b, c) => write!(fmt, "x{} := x{} == x{}", a, b, c),
            Neq(a, b, c) => write!(fmt, "x{} := x{} != x{}", a, b, c),
            Lt(a, b, c) => write!(fmt, "x{} := x{} < x{}", a, b, c),
            Gt(a, b, c) => write!(fmt, "x{} := x{} > x{}", a, b, c),
            Leq(a, b, c) => write!(fmt, "x{} := x{} <= x{}", a, b, c),
            Geq(a, b, c) => write!(fmt, "x{} := x{} >= x{}", a, b, c),
            Jump(off) => write!(fmt, "jump {}", off),
            CondJump(a, b, c) => write!(fmt, "cond x{} {} {}", a, b, c),
            MkTup(a, b, c) => write!(fmt, "x{} := (x{}; {})", a, b, c),
            IdxTup(a, b, c) => write!(fmt, "x{} := x{}[x{}]", a, b, c),
            Call(a, b, c) => write!(fmt, "x{} := x{}(x{})", a, b, c),
            Return(None) => write!(fmt, "return"),
            Return(Some(a)) => write!(fmt, "return x{}", a),
            Read(a) => write!(fmt, "x{} := read", a),
            Write(a) => write!(fmt, "write x{}", a),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Val {
    B(bool),
    I(i64),
    F(f64),
    T(Vec<Val>),
    C(FnId),
}

impl fmt::Display for Val {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::Val::*;
        match *self {
            B(b) => write!(fmt, "{}", b),
            I(i) => write!(fmt, "{}", i),
            F(f) => {
                let text = format!("{}", f);
                if text.contains('.') {
                    write!(fmt, "{}", text)
                } else {
                    write!(fmt, "{}.0", text)
                }
            }
            T(ref t) => write!(
                fmt,
                "({})",
                t.iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            C(c) => write!(fmt, "f{}", c),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Defn {
    consts: Vec<Val>,
    code: Vec<Instr>,
    local_count: AddrSize,
}

/// A piece of compiled code that's ready to be evaluated.
#[derive(Debug, PartialEq)]
pub struct Program {
    defns: Vec<Defn>,
    entry_point: FnId,
}

impl fmt::Display for Program {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for (i, defn) in self.defns.iter().enumerate() {
            if i != 0 {
                writeln!(fmt, "\n")?;
            }
            let consts = defn.consts
                .iter()
                .map(|k| format!(" {}", k))
                .collect::<String>();
            write!(fmt, "defn f{} {} :{}", i, defn.local_count, consts)?;
            for line in &defn.code {
                write!(fmt, "\n    {}", line)?;
            }
        }
        Ok(())
    }
}

/// Represents failures during execution.
///
/// Use it to get access to the cause, backtraces, etc.
#[derive(Debug, PartialEq)]
pub struct EvalError {}

impl Program {
    /// Evaluate a program with given I/O buffers.
    pub fn eval<R: Read, W: Write>(&self, input: &mut R, output: &mut W) -> Result<Val, EvalError> {
        use self::Val::*;
        use self::Instr::*;

        let mut stack = Vec::new();
        let mut code = &self.defns[self.entry_point as usize];
        let mut locals = vec![I(0); code.local_count as usize];
        let mut iptr = 0;
        loop {
            match code.code.get(iptr).unwrap_or_else(|| &Return(None)) {
                &Const(a, k) => locals[a as usize] = code.consts[k as usize].clone(),
                &Copy(a, b) => locals[a as usize] = locals[b as usize].clone(),
                &Add(a, b, c) => locals[a as usize] = (&locals[b as usize] + &locals[c as usize])?,
                &Sub(a, b, c) => locals[a as usize] = (&locals[b as usize] - &locals[c as usize])?,
                &Mul(a, b, c) => locals[a as usize] = (&locals[b as usize] * &locals[c as usize])?,
                &Div(a, b, c) => locals[a as usize] = (&locals[b as usize] / &locals[c as usize])?,
                &Rem(a, b, c) => locals[a as usize] = (&locals[b as usize] % &locals[c as usize])?,
                &And(a, b, c) => locals[a as usize] = (&locals[b as usize] & &locals[c as usize])?,
                &Orr(a, b, c) => locals[a as usize] = (&locals[b as usize] | &locals[c as usize])?,
                &Xor(a, b, c) => locals[a as usize] = (&locals[b as usize] ^ &locals[c as usize])?,
                &Eq(a, b, c) => locals[a as usize] = B(&locals[b as usize] == &locals[c as usize]),
                &Neq(a, b, c) => locals[a as usize] = B(&locals[b as usize] != &locals[c as usize]),
                &Lt(a, b, c) => locals[a as usize] = B(&locals[b as usize] < &locals[c as usize]),
                &Gt(a, b, c) => locals[a as usize] = B(&locals[b as usize] > &locals[c as usize]),
                &Leq(a, b, c) => locals[a as usize] = B(&locals[b as usize] <= &locals[c as usize]),
                &Geq(a, b, c) => locals[a as usize] = B(&locals[b as usize] >= &locals[c as usize]),
                &MkTup(a, b, c) => {
                    locals[a as usize] = T(locals[b as usize..(b + c) as usize].into())
                }
                &IdxTup(a, t, i) => {
                    locals[a as usize] = match (&locals[t as usize], &locals[i as usize]) {
                        (&T(ref t), &I(i)) => t[i as usize].clone(),
                        _ => return Err(EvalError {}),
                    };
                }
                &Call(a, f, c) => {
                    let new_code = &self.defns[f as usize];
                    let mut new_locals = vec![I(0); new_code.local_count as usize];
                    new_locals[0] = locals[c as usize].clone();
                    stack.push((a, code, locals, iptr));
                    code = new_code;
                    locals = new_locals;
                    iptr = 0;
                    continue;
                }
                &Return(a) => {
                    let res = match a {
                        Some(a) => locals.remove(a as usize),
                        None => T(Vec::new()),
                    };

                    if let Some((addr, new_code, mut new_locals, new_iptr)) = stack.pop() {
                        new_locals[addr as usize] = res;
                        locals = new_locals;
                        code = new_code;
                        iptr = new_iptr;
                    } else {
                        return Ok(res);
                    }
                }
                &Read(a) => {
                    let mut buf = [0];
                    input.read(&mut buf[..]).map_err(|_| EvalError {})?;
                    locals[a as usize] = I(buf[0] as i64);
                }
                &Write(a) => {
                    match locals[a as usize] {
                        I(x) => {
                            output.write(&[x as u8]).map_err(|_| EvalError {})?;
                        }
                        _ => return Err(EvalError {}),
                    };
                }
                &Jump(a) => {
                    iptr = sum(iptr, a as isize);
                    continue;
                }
                &CondJump(a, b, c) => {
                    match locals[a as usize] {
                        B(true) => iptr = sum(iptr, b as isize),
                        B(false) => iptr = sum(iptr, c as isize),
                        _ => return Err(EvalError {}),
                    }
                    continue;
                }
            }
            iptr += 1;
        }
    }
}

fn sum(a: usize, b: isize) -> usize {
    if b > 0 {
        a + b as usize
    } else {
        a - (b.abs() as usize)
    }
}

impl<'a> Add for &'a Val {
    type Output = Result<Val, EvalError>;
    fn add(self, rhs: &Val) -> Self::Output {
        use self::Val::*;
        match (self, rhs) {
            (&I(b), &I(c)) => b.checked_add(c).ok_or(EvalError {}).map(I),
            (&F(b), &F(c)) => Ok(F(b + c)),
            _ => Err(EvalError {}),
        }
    }
}

impl<'a> Sub for &'a Val {
    type Output = Result<Val, EvalError>;
    fn sub(self, rhs: &Val) -> Self::Output {
        use self::Val::*;
        match (self, rhs) {
            (&I(b), &I(c)) => b.checked_sub(c).ok_or(EvalError {}).map(I),
            (&F(b), &F(c)) => Ok(F(b - c)),
            _ => Err(EvalError {}),
        }
    }
}

impl<'a> Mul for &'a Val {
    type Output = Result<Val, EvalError>;
    fn mul(self, rhs: &Val) -> Self::Output {
        use self::Val::*;
        match (self, rhs) {
            (&I(b), &I(c)) => b.checked_mul(c).ok_or(EvalError {}).map(I),
            (&F(b), &F(c)) => Ok(F(b * c)),
            _ => Err(EvalError {}),
        }
    }
}

impl<'a> Div for &'a Val {
    type Output = Result<Val, EvalError>;
    fn div(self, rhs: &Val) -> Self::Output {
        use self::Val::*;
        match (self, rhs) {
            (&I(b), &I(c)) => b.checked_div(c).ok_or(EvalError {}).map(I),
            (&F(b), &F(c)) => Ok(F(b / c)),
            _ => Err(EvalError {}),
        }
    }
}

impl<'a> Rem for &'a Val {
    type Output = Result<Val, EvalError>;
    fn rem(self, rhs: &Val) -> Self::Output {
        use self::Val::*;
        match (self, rhs) {
            (&I(b), &I(c)) => b.checked_rem(c).ok_or(EvalError {}).map(I),
            (&F(b), &F(c)) => Ok(F(b / c)),
            _ => Err(EvalError {}),
        }
    }
}

impl<'a> BitAnd for &'a Val {
    type Output = Result<Val, EvalError>;
    fn bitand(self, rhs: &Val) -> Self::Output {
        use self::Val::*;
        match (self, rhs) {
            (&I(b), &I(c)) => Ok(I(b & c)),
            (&B(b), &B(c)) => Ok(B(b && c)),
            _ => Err(EvalError {}),
        }
    }
}

impl<'a> BitOr for &'a Val {
    type Output = Result<Val, EvalError>;
    fn bitor(self, rhs: &Val) -> Self::Output {
        use self::Val::*;
        match (self, rhs) {
            (&I(b), &I(c)) => Ok(I(b | c)),
            (&B(b), &B(c)) => Ok(B(b || c)),
            _ => Err(EvalError {}),
        }
    }
}

impl<'a> BitXor for &'a Val {
    type Output = Result<Val, EvalError>;
    fn bitxor(self, rhs: &Val) -> Self::Output {
        use self::Val::*;
        match (self, rhs) {
            (&I(b), &I(c)) => Ok(I(b ^ c)),
            _ => Err(EvalError {}),
        }
    }
}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Val) -> Option<Ordering> {
        use self::Val::*;
        match (self, other) {
            (&I(b), &I(c)) => b.partial_cmp(&c),
            (&F(b), &F(c)) => b.partial_cmp(&c),
            (&B(b), &B(c)) => b.partial_cmp(&c),
            _ => None,
        }
    }
}
