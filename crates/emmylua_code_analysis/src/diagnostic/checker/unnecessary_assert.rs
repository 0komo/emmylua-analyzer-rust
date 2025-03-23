use emmylua_parser::{LuaAstNode, LuaCallExpr};

use crate::{DiagnosticCode, LuaType, SemanticModel};

use super::{Checker, DiagnosticContext};

pub struct UnnecessaryAssertChecker;

impl Checker for UnnecessaryAssertChecker {
    const CODES: &[DiagnosticCode] = &[DiagnosticCode::UnnecessaryAssert];

    fn check(context: &mut DiagnosticContext, semantic_model: &SemanticModel) {
        let root = semantic_model.get_root().clone();
        for call_expr in root.descendants::<LuaCallExpr>() {
            if call_expr.is_assert() {
                check_assert_rule(context, semantic_model, call_expr);
            }
        }
    }
}

fn check_assert_rule(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    call_expr: LuaCallExpr,
) -> Option<()> {
    let args = call_expr.get_args_list()?;
    let arg_exprs = args.get_args().collect::<Vec<_>>();
    if let Some(first_expr) = arg_exprs.first() {
        let expr_type = semantic_model.infer_expr(first_expr.clone())?;
        let first_type = match &expr_type {
            LuaType::MuliReturn(multi) => multi.get_type(0)?.clone(),
            _ => expr_type,
        };

        if first_type.is_always_truthy() {
            context.add_diagnostic(
                DiagnosticCode::UnnecessaryAssert,
                call_expr.get_range(),
                t!("Unnecessary assert: this expression is always truthy").to_string(),
                None,
            );
        }
    }
    Some(())
}
