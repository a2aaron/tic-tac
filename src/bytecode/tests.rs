use super::*;

use std::io;

macro_rules! test_program {
(
    name: $name:ident;
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
            #[test]
            fn test_program() {
                use self::Val::*;
                use self::Instr::*;
                let program = Program {
                    defns: vec![
                    $(Defn {
                        code: vec![$($instrs),*],
                        consts: vec![$($consts),*],
                        local_count: $count,
                    },)*],
                    entry_point: 0,
                };
                assert_eq!(
                    program.eval(&mut io::empty(), &mut io::sink()),
                    $result
                );
            }
        }
    }
}

test_program! {
    name: test_cond_jump_false;
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
    name: test_jump;
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
    defn {
        code: [
            Const(0, 0),
            Const(1, 1),
            MkTup(0, 0, 1),
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
}
