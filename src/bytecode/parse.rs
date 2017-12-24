use super::{Addr, Defn, Program, Val};
use parse_util::{Buffer, ParseError, ParseResult};

/// Parses a constant for function declarations.
///
/// Accepted constants:
///
/// - booleans `true` and `false`
/// - integers
/// - floats (`.` mandatory)
/// - function numbers (such as `f0`)
fn parse_const(text: &str) -> Result<Val, ()> {
    if text == "true" {
        Ok(Val::B(true))
    } else if text == "false" {
        Ok(Val::B(false))
    } else if text.contains('.') {
        Ok(Val::F(text.parse().map_err(|_| ())?))
    } else if text.starts_with('f') {
        Ok(Val::C(text[1..].parse().map_err(|_| ())?))
    } else {
        Ok(Val::I(text.parse().map_err(|_| ())?))
    }
}

fn parse_constants<'a>(mut buf: Buffer<'a>) -> ParseResult<'a, Vec<Val>> {
    let mut consts = Vec::new();
    while !buf.text.is_empty() {
        let (new_buf, text) = buf.trim_left().til(char::is_whitespace)?;
        buf = new_buf;
        match parse_const(text) {
            Ok(c) => consts.push(c),
            Err(()) => return Err(buf.expected("a constant")),
        }
    }
    Ok((buf, consts))
}

trait ParseExt<'a> {
    fn addr(self, prefix: &str) -> ParseResult<'a, Addr>;
}

impl<'a> ParseExt<'a> for Buffer<'a> {
    fn addr(self, prefix: &str) -> ParseResult<'a, Addr> {
        self.trim_left()
            .token(prefix)?
            .parse_til(|c| !c.is_digit(10))
    }
}

pub fn parse(text: &str) -> Result<Program, ParseError> {
    use bytecode::Instr::*;

    let mut defns = Vec::new();
    for (row, line) in text.lines().enumerate() {
        let buf = Buffer {
            row: row + 1,
            col: 0,
            text: line,
        }.trim();

        // Handle comments and blank lines
        if buf.starts_with("#") || buf.text.is_empty() {
            continue;
        }

        // Every function starts with `defn fN N k1 k2 k3 ...` where `fN` is the
        // function number, `N` is the number of locals, and each `k` is a
        // constant.
        if buf.starts_with("defn") {
            let buf = buf.token("defn")?.space()?;
            let (buf, fn_number): (_, usize) = buf.token("f")?.parse_til(char::is_whitespace)?;
            if fn_number != defns.len() {
                return Err(buf.expected(format!(
                    "function id f{}, got f{}",
                    defns.len(),
                    fn_number
                )));
            }
            let (buf, local_count) = buf.trim_left().parse_til(char::is_whitespace)?;
            let buf = buf.trim_left().token(":")?.trim_left();
            let (buf, consts) = parse_constants(buf)?;
            buf.end()?;
            defns.push(Defn {
                code: Vec::new(),
                consts,
                local_count,
            });
        } else {
            if let Some(ref mut defn) = defns.last_mut() {
                if buf.starts_with("return") {
                    // return OR return x0
                    let buf = buf.token("return")?.space_or_end()?;
                    if buf.text.is_empty() {
                        buf.end()?;
                        defn.code.push(Return(None));
                    } else {
                        let (buf, addr) = buf.addr("x")?;
                        buf.end()?;
                        defn.code.push(Return(Some(addr)));
                    }
                } else if buf.starts_with("write") {
                    // write x0
                    let (buf, addr) = buf.token("write")?.space()?.addr("x")?;
                    buf.end()?;
                    defn.code.push(Write(addr));
                } else if buf.starts_with("jump") {
                    // jump 10
                    let (buf, br): (_, i16) = buf.token("jump")?
                        .space()?
                        .parse_til(|c| !(c.is_digit(10) || c == '-'))?;
                    buf.end()?;
                    defn.code.push(Jump(br));
                } else if buf.starts_with("cond") {
                    // cond x0 10 20
                    let (buf, addr) = buf.token("cond")?.space()?.addr("x")?;
                    let (buf, br1): (_, i8) =
                        buf.space()?.parse_til(|c| !(c.is_digit(10) || c == '-'))?;
                    let (buf, br2): (_, i8) =
                        buf.space()?.parse_til(|c| !(c.is_digit(10) || c == '-'))?;
                    buf.end()?;
                    defn.code.push(CondJump(addr, br1, br2));
                } else {
                    // x0 := ...
                    let (buf, dest) = buf.addr("x")?;
                    let buf = buf.trim_left().token(":=")?.trim_left();
                    if buf.starts_with("k") {
                        // x0 := k1
                        let (buf, k) = buf.addr("k")?;
                        buf.end()?;
                        defn.code.push(Const(dest, k));
                    } else if buf.starts_with("(") {
                        // x0 := (x1..x2)
                        let (buf, b) = buf.trim_left().token("(")?.addr("x")?;
                        let (buf, c) = buf.trim_left().token("..")?.addr("x")?;
                        buf.trim_left().token(")")?.end()?;
                        defn.code.push(MkTup(dest, b, c));
                    } else if buf.starts_with("read") {
                        // x0 := read
                        buf.token("read")?.end()?;
                        defn.code.push(Read(dest));
                    } else {
                        // x0 := x1 ...
                        let (buf, b) = buf.addr("x")?;
                        let buf = buf.trim_left();

                        if buf.text.is_empty() {
                            // x0 := x1
                            defn.code.push(Copy(dest, b));
                            continue;
                        }

                        let (buf, op) = buf.first_token_of(&[
                            "+", "-", "*", "/", "%", "&", "|", "^", "==", "!=", "<=", ">=", "<",
                            ">", "(", "[",
                        ])?;
                        match op {
                            // x0 := x1 op x2
                            "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^" | "==" | "!=" | "<="
                            | ">=" | "<" | ">" => {
                                let (buf, c) = buf.addr("x")?;
                                buf.end()?;
                                defn.code.push(match op {
                                    "+" => Add(dest, b, c),
                                    "-" => Sub(dest, b, c),
                                    "*" => Mul(dest, b, c),
                                    "/" => Div(dest, b, c),
                                    "%" => Rem(dest, b, c),
                                    "&" => And(dest, b, c),
                                    "|" => Orr(dest, b, c),
                                    "^" => Xor(dest, b, c),
                                    "==" => Eq(dest, b, c),
                                    "!=" => Neq(dest, b, c),
                                    "<=" => Leq(dest, b, c),
                                    ">=" => Geq(dest, b, c),
                                    "<" => Lt(dest, b, c),
                                    ">" => Gt(dest, b, c),
                                    _ => unreachable!("invalid ops"),
                                });
                            }
                            // x0 := x1(x2)
                            "(" => {
                                let (buf, c) = buf.addr("x")?;
                                buf.trim_left().token(")")?.end()?;
                                defn.code.push(Call(dest, b, c));
                            }
                            // x0 := x1[x2]
                            "[" => {
                                let (buf, c) = buf.addr("x")?;
                                buf.trim_left().token("]")?.end()?;
                                defn.code.push(IdxTup(dest, b, c));
                            }
                            _ => unreachable!("unmentioned op"),
                        }
                    }
                }
            } else {
                return Err(buf.expected("to be inside a definition"));
            }
        }
    }
    Ok(Program {
        defns,
        entry_point: 0,
    })
}
