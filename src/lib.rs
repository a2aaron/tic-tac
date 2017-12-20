use std::io::{Read, Write};
use std::ops::{Add, Div, Mul, Rem, Sub};

type Addr = u8;
type AddrSize = u8;
type FnId = u16;

#[derive(Debug)]
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
    /// Jumps program execution by n instructions
    Jump(i16),
    /// Jumps program execution by n instructions if a is true, else it jumps by m instructions
    /// Note that a must be a boolean, otherwise the program is invalid.
    CondJump(Addr, i8, i8),
    /// Constructs a tuple from a contiguous range of slots, a = (b..c)
    MkTup(Addr, Addr, Addr),
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
    pub fn eval<R: Read, W: Write>(&self, input: &mut R, output: &mut W) -> Result<Val, EvalError> {
        use Val::*;
        use Instr::*;

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
                &MkTup(a, b, c) => {
                    locals[a as usize] = T(locals[b as usize..c as usize + 1].into())
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
                    iptr += a as usize;
                    continue;
                }
                &CondJump(a, b, c) => {
                    if let Val::B(x) = locals[a as usize] {
                        if x {
                            iptr += b as usize;
                        } else {
                            iptr += c as usize;
                        }
                        continue;
                    } else {
                        return Err(EvalError {});
                    }
                }
            }
            iptr += 1;
        }
    }
}

impl<'a> Add for &'a Val {
    type Output = Result<Val, EvalError>;
    fn add(self, rhs: &Val) -> Self::Output {
        use Val::*;
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
        use Val::*;
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
        use Val::*;
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
        use Val::*;
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
        use Val::*;
        match (self, rhs) {
            (&I(b), &I(c)) => b.checked_rem(c).ok_or(EvalError {}).map(I),
            (&F(b), &F(c)) => Ok(F(b / c)),
            _ => Err(EvalError {}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cond_jump_false() {
        use Val::*;
        use Instr::*;

        let program = Program {
            defns: vec![
                Defn {
                    code: vec![
                        Const(0, 0),
                        Const(1, 1),
                        Const(2, 2),
                        CondJump(0, 1, 2),
                        Return(Some(1)),
                        Return(Some(2)),
                    ],
                    consts: vec![B(false), I(3), I(5)],
                    local_count: 3,
                },
            ],
            entry_point: 0,
        };
        assert_eq!(
            program.eval(&mut std::io::empty(), &mut std::io::sink()),
            Ok(I(5))
        );
    }

    #[test]
    fn test_cond_jump_true() {
        use Val::*;
        use Instr::*;

        let program = Program {
            defns: vec![
                Defn {
                    code: vec![
                        Const(0, 0),
                        Const(1, 1),
                        Const(2, 2),
                        CondJump(0, 1, 2),
                        Return(Some(1)),
                        Return(Some(2)),
                    ],
                    consts: vec![B(true), I(3), I(5)],
                    local_count: 3,
                },
            ],
            entry_point: 0,
        };
        assert_eq!(
            program.eval(&mut std::io::empty(), &mut std::io::sink()),
            Ok(I(3))
        );
    }

    #[test]
    fn test_cond_jump_err() {
        use Val::*;
        use Instr::*;

        let program = Program {
            defns: vec![
                Defn {
                    code: vec![
                        Const(0, 0),
                        Const(1, 1),
                        Const(2, 2),
                        CondJump(0, 1, 2),
                        Return(Some(1)),
                        Return(Some(2)),
                    ],
                    consts: vec![I(0), I(3), I(5)],
                    local_count: 3,
                },
            ],
            entry_point: 0,
        };
        assert_eq!(
            program.eval(&mut std::io::empty(), &mut std::io::sink()),
            Err(EvalError {})
        );
    }


    #[test]
    fn test_jump() {
        use Val::*;
        use Instr::*;

        let program = Program {
            defns: vec![
                Defn {
                    code: vec![Jump(1), Const(0, 0), Const(0, 1), Return(Some(0))],
                    consts: vec![I(3), I(5)],
                    local_count: 2,
                },
            ],
            entry_point: 0,
        };
        assert_eq!(
            program.eval(&mut std::io::empty(), &mut std::io::sink()),
            Ok(I(5))
        );
    }

    #[test]
    fn call_return() {
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
                        Return(Some(0)),
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
                        Return(Some(0)),
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

    #[test]
    fn arith() {
        use Val::*;
        use Instr::*;

        let program = Program {
            defns: vec![
                Defn {
                    code: vec![
                        Const(0, 0),
                        Const(1, 1),
                        Add(0, 0, 1),
                        Mul(0, 0, 0),
                        Const(1, 2),
                        Rem(0, 0, 1),
                        Const(1, 3),
                        Div(0, 1, 0),
                        Return(Some(0)),
                    ],
                    consts: vec![I(1), I(2), I(7), I(15)],
                    local_count: 3,
                },
            ],
            entry_point: 0,
        };

        assert_eq!(
            program.eval(&mut std::io::empty(), &mut std::io::sink()),
            Ok(I(15 / (((1 + 2) * (1 + 2)) % 7)))
        );
    }

    #[test]
    fn io() {
        use Val::*;
        use Instr::*;

        let program = Program {
            defns: vec![
                Defn {
                    code: vec![Read(0), Read(1), Add(1, 1, 1), Write(1), Write(0)],
                    consts: vec![],
                    local_count: 2,
                },
            ],
            entry_point: 0,
        };

        let mut input = std::io::Cursor::new([13, 2]);
        let mut output = Vec::new();
        assert_eq!(program.eval(&mut input, &mut output), Ok(T(Vec::new())));
        assert_eq!(output, vec![4, 13]);
    }
}
