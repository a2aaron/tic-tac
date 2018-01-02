use super::*;

#[test]
fn test_compile_expr() {
    use bytecode::Instr::*;
    use bytecode::Val::*;

    let expr = Expr::Lit(I(42));
    let mut ctx = FunctionCtx::new();

    let result = (0, vec![Const(0, 0)]);
    assert_eq!(ctx.compile_expr(&expr), result);
}
