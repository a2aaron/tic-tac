use super::*;

use std::io;

macro_rules! test_program {
(
    name: $name:ident;
    text: $text:expr;
    $(defn {
        code: [$($instrs:expr),* $(,)*],
        consts: [$($consts:expr),* $(,)*],
        local_count: $count:expr,
    })*
    result: $result:expr;
) => {
        mod $name {
            use std::io;
            use super::*;

            #[allow(unused)]
            fn program() -> Program {
                use self::Val::*;
                use self::Instr::*;
                Program {
                    defns: vec![
                        $(Defn {
                            code: vec![$($instrs),*],
                            consts: vec![$($consts),*],
                            local_count: $count,
                        },)*],
                    entry_point: 0,
                }
            }

            #[test]
            #[allow(unused)]
            fn test_eval() {
                use self::Val::*;
                assert_eq!(
                    program().eval(&mut io::empty(), &mut io::sink()),
                    $result
                );
            }

            #[test]
            fn test_parse() {
                assert_eq!(parse::parse($text), Ok(program()));
            }
        }
    }
}

test_program! {
    name: test_cond_jump_false;
    text: r#"
defn f0 3 : false 3 5
x0 := k0
x1 := k1
x2 := k2
cond x0 1 2
return x1
return x2
"#;
    defn {
        code: [
            Const(0, 0),
            Const(1, 1),
            Const(2, 2),
            CondJump(0, 1, 2),
            Return(Some(1)),
            Return(Some(2)),
        ],
        consts: [B(false), I(3), I(5)],
        local_count: 3,
    }
    result: Ok(I(5));
}

test_program! {
    name: test_cond_jump_true;
    text: r#"
defn f0 3 : true 3 5
x0 := k0
x1 := k1
x2 := k2
cond x0 1 2
return x1
return x2
"#;
    defn {
        code: [
            Const(0, 0),
            Const(1, 1),
            Const(2, 2),
            CondJump(0, 1, 2),
            Return(Some(1)),
            Return(Some(2)),
        ],
        consts: [B(true), I(3), I(5)],
        local_count: 3,
    }
    result: Ok(I(3));
}

test_program! {
    name: test_cond_jump_err;
    text: r#"
defn f0 3 : 0 3 5
x0 := k0
x1 := k1
x2 := k2
cond x0 1 2
return x1
return x2
"#;
    defn {
        code: [
            Const(0, 0),
            Const(1, 1),
            Const(2, 2),
            CondJump(0, 1, 2),
            Return(Some(1)),
            Return(Some(2)),
        ],
        consts: [I(0), I(3), I(5)],
        local_count: 3,
    }
    result: Err(EvalError {});
}

test_program! {
    name: test_bitwise;
    text: r#"
defn f0 5 : 1 2 4 -1 -8
x0 := k0
x1 := k1
# 3
x0 := x0 | x1
x2 := k2
# 7
x1 := x0 | x2
# 3
x2 := x0 & x1
x0 := x2 == x0
x4 := k3
x3 := x1 ^ x4
x4 := k4
x1 := x3 == x4
x0 := x0 & x1
return x0
"#;
    defn {
        code: [
            Const(0, 0),
            Const(1, 1),
            Orr(0, 0, 1),
            Const(2, 2),
            Orr(1, 0, 2),
            And(2, 0, 1),
            Eq(0, 2, 0),
            Const(4, 3),
            Xor(3, 1, 4),
            Const(4, 4),
            Eq(1, 3, 4),
            And(0, 0, 1),
            Return(Some(0)),
        ],
        consts: [I(1), I(2), I(4), I(-1), I(-8)],
        local_count: 5,
    }
    result: Ok(B(true));
}

test_program! {
    name: test_jump;
    text: r#"
defn f0 2 : 3 5
jump 1
x0 := k0
x0 := k1
return x0
"#;
    defn {
        code: [
            Jump(1),
            Const(0, 0),
            Const(0, 1),
            Return(Some(0))
        ],
        consts: [I(3), I(5)],
        local_count: 2,
    }
    result: Ok(I(5));
}

test_program! {
    name: test_backwards_jump;
    text: r#"
defn f0 2 : 3 5
jump 3
x0 := k0
jump 3
x0 := k1
jump -3
return x0
"#;
    defn {
        code: [
            Jump(3),
            Const(0, 0),
            Jump(3),
            Const(0, 1),
            Jump(-3),
            Return(Some(0)),
        ],
        consts: [I(3), I(5)],
        local_count: 2,
    }
    result: Ok(I(3));
}

test_program! {
    name: call_return;
    text: r#"
defn f0 2 : 42 69 f1
x0 := k0
x1 := k1
x0 := (x0; 2)
x1 := k2
x0 := x1(x0)
return x0

defn f1 3 : 0 1
x1 := k0
x1 := x0[x1]
x2 := k1
x2 := x0[x2]
x0 := x1 + x2
return x0
"#;
    defn {
        code: [
            Const(0, 0),
            Const(1, 1),
            MkTup(0, 0, 2),
            Const(1, 2),
            Call(0, 1, 0),
            Return(Some(0)),
        ],
        consts: [I(42), I(69), C(1)],
        local_count: 2,
    }
    defn {
        code: [
            Const(1, 0),
            IdxTup(1, 0, 1),
            Const(2, 1),
            IdxTup(2, 0, 2),
            Add(0, 1, 2),
            Return(Some(0)),
        ],
        consts: [I(0), I(1)],
        local_count: 3,
    }
    result: Ok(I(111));
}

test_program! {
    name: arith;
    text: r#"
defn f0 3 : 1 2 7 15
x0 := k0
x1 := k1
x0 := x0 + x1
x0 := x0 * x0
x1 := k2
x0 := x0 % x1
x1 := k3
x0 := x1 / x0
return x0
"#;
    defn {
        code: [
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
        consts: [I(1), I(2), I(7), I(15)],
        local_count: 3,
    }
    result: Ok(I(15 / (((1 + 2) * (1 + 2)) % 7)));
}

test_program! {
    name: untup;
    text: r#"
defn f0 2 : f1
  x0 := read
  x1 := read
  x0 := (x0; 2)
  x1 := k0
  x0 := x1(x0)
  (x0; 2) := x0
  write x0
  write x1

defn f1 3 :
  (x0; 2) := x0
  x2 := x0
  x0 := (x1; 2)
  return x0
"#;
    defn {
        code: [
            Read(0),
            Read(1),
            MkTup(0, 0, 2),
            Const(1, 0),
            Call(0, 1, 0),
            UnTup(0, 2, 0),
            Write(0),
            Write(1),
        ],
        consts: [C(1)],
        local_count: 2,
    }
    defn {
        code: [
            UnTup(0, 2, 0),
            Copy(2, 0),
            MkTup(0, 1, 2),
            Return(Some(0)),
        ],
        consts: [],
        local_count: 3,
    }
    result: Ok(T(vec![]));
}

#[test]
fn io() {
    use self::Val::*;
    use self::Instr::*;

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

    let mut input = io::Cursor::new([13, 2]);
    let mut output = Vec::new();
    assert_eq!(program.eval(&mut input, &mut output), Ok(T(Vec::new())));
    assert_eq!(output, vec![4, 13]);

    assert_eq!(
        parse::parse(
            r#"
defn f0 2 :
x0 :=read
x1 := read
x1 := x1 + x1
write x1
write x0
"#,
        ),
        Ok(Program {
            defns: vec![
                Defn {
                    code: vec![Read(0), Read(1), Add(1, 1, 1), Write(1), Write(0)],
                    consts: vec![],
                    local_count: 2,
                },
            ],
            entry_point: 0,
        })
    );
}

#[test]
fn test_format() {
    use self::Val::*;
    use self::Instr::*;
    assert_eq!(
        format!(
            "{}",
            Program {
                defns: vec![
                    Defn {
                        code: vec![
                            Const(0, 0),
                            Const(1, 1),
                            MkTup(0, 0, 2),
                            Const(1, 2),
                            Call(0, 1, 0),
                            Return(Some(0)),
                        ],
                        consts: vec![I(42), I(69), C(1)],
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
            }
        ),
        r#"defn f0 2 : 42 69 f1
    x0 := k0
    x1 := k1
    x0 := (x0; 2)
    x1 := k2
    x0 := x1(x0)
    return x0

defn f1 3 : 0 1
    x1 := k0
    x1 := x0[x1]
    x2 := k1
    x2 := x0[x2]
    x0 := x1 + x2
    return x0"#
    );
}
