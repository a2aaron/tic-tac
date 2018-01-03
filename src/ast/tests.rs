use super::*;

#[test]
fn test_compile_lit() {
    use bytecode::Instr::*;
    use bytecode::Val::*;

    let expr = Expr::Lit(I(42));
    let mut ctx = FunctionCtx::new();

    let result = (0, vec![Const(0, 0)]);
    assert_eq!(ctx.compile_expr(&expr), result);
    assert_eq!(
        ctx,
        FunctionCtx {
            vars: HashMap::new(),
            consts: vec![I(42)],
            free_reg: 1,
            max_reg: 1,
        }
    );
}

#[test]
fn test_compile_raw_lit() {
    use bytecode::Instr::*;
    use bytecode::Val::*;

    let stmt = Stmt::RawExpr(Expr::Lit(I(42)));
    let mut ctx = FunctionCtx::new();

    let result = vec![Const(0, 0)];
    assert_eq!(ctx.compile_stmt(&stmt), result);
    assert_eq!(
        ctx,
        FunctionCtx {
            vars: HashMap::new(),
            consts: vec![I(42)],
            free_reg: 0,
            max_reg: 1,
        }
    );
}

#[test]
fn test_compile_var() {
    use bytecode::Instr::*;

    let name = Name { id: 0 };
    let mut ctx = FunctionCtx::new();
    let code = vec![
        Stmt::Declare(name.clone()),
        Stmt::RawExpr(Expr::Var(name.clone())),
    ];

    let result = vec![];
    assert_eq!(ctx.compile(&code), result);
    assert_eq!(
        ctx,
        FunctionCtx {
            vars: vec![(name, 0)].into_iter().collect(),
            consts: vec![],
            free_reg: 1,
            max_reg: 1,
        }
    );
}

#[test]
fn test_compile_declare() {
    use bytecode::Instr::*;
    use bytecode::Val::*;
    let name = Name { id: 0 };
    let stmt = Stmt::Declare(name);
    let mut ctx = FunctionCtx::new();

    let result = vec![];
    assert_eq!(ctx.compile_stmt(&stmt), result);
    assert_eq!(
        ctx,
        FunctionCtx {
            vars: vec![(name, 0)].into_iter().collect(),
            consts: vec![],
            free_reg: 1,
            max_reg: 1,
        }
    );
}

#[test]
fn test_compile_assign() {
    use bytecode::Instr::*;
    use bytecode::Val::*;
    let name = Name { id: 0 };

    let code = vec![Stmt::Declare(name), Stmt::Assign(name, Expr::Lit(I(69)))];
    let mut ctx = FunctionCtx::new();

    let result = vec![Const(1, 0), Copy(0, 1)];
    assert_eq!(ctx.compile(&code), result);
    assert_eq!(
        ctx,
        FunctionCtx {
            vars: vec![(name, 0)].into_iter().collect(),
            consts: vec![I(69)],
            free_reg: 2,
            max_reg: 2,
        }
    );
}
