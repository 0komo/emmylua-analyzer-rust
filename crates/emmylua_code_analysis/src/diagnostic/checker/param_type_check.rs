use emmylua_parser::{LuaAst, LuaAstNode, LuaAstToken, LuaCallExpr, LuaExpr};
use rowan::TextRange;

use crate::{
    humanize_type, DiagnosticCode, LuaType, RenderLevel, SemanticModel, TypeCheckFailReason,
    TypeCheckResult,
};

use super::DiagnosticContext;

pub const CODES: &[DiagnosticCode] = &[DiagnosticCode::ParamTypeNotMatch];

/// a simple implementation of param type check, later we will do better
pub fn check(context: &mut DiagnosticContext, semantic_model: &SemanticModel) -> Option<()> {
    let root = semantic_model.get_root().clone();
    for node in root.descendants::<LuaAst>() {
        match node {
            LuaAst::LuaCallExpr(call_expr) => {
                check_call_expr(context, semantic_model, call_expr);
            }
            _ => {}
        }
    }

    Some(())
}

fn check_call_expr(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    call_expr: LuaCallExpr,
) -> Option<()> {
    let func = semantic_model.infer_call_expr_func(call_expr.clone(), None)?;
    let mut params = func.get_params().to_vec();
    let arg_exprs: Vec<LuaExpr> = call_expr.get_args_list()?.get_args().collect::<Vec<_>>();
    let (mut arg_types, mut arg_ranges) = {
        let infos = semantic_model.infer_value_expr_infos(&arg_exprs)?;
        let arg_types = infos.iter().map(|(typ, _)| typ.clone()).collect::<Vec<_>>();
        let arg_ranges = infos
            .iter()
            .map(|(_, range)| range.clone())
            .collect::<Vec<_>>();
        (arg_types, arg_ranges)
    };

    let colon_call = call_expr.is_colon_call();
    let colon_define = func.is_colon_define();
    match (colon_call, colon_define) {
        (true, true) | (false, false) => {}
        (false, true) => {
            // 插入 self 参数
            params.insert(0, ("self".into(), Some(LuaType::SelfInfer)));
        }
        (true, false) => {
            // 往调用参数插入插入调用者类型
            arg_types.insert(0, get_call_source_type(semantic_model, &call_expr)?);
            arg_ranges.insert(0, call_expr.get_colon_token()?.get_range());
        }
    }

    for (idx, param) in params.iter().enumerate() {
        if param.0 == "..." {
            if let Some(variadic_type) = param.1.clone() {
                check_variadic_param_match_args(
                    context,
                    semantic_model,
                    &variadic_type,
                    &arg_types[idx..],
                    &arg_ranges[idx..],
                );
            }

            break;
        }

        if let Some(param_type) = param.1.clone() {
            let arg_type = arg_types.get(idx).unwrap_or(&LuaType::Any);
            let mut check_type = param_type.clone();
            // 对于第一个参数, 他有可能是`:`调用, 所以需要特殊处理
            if idx == 0 && param_type.is_self_infer() {
                if let Some(result) = get_call_source_type(semantic_model, &call_expr) {
                    check_type = result;
                }
            }
            let result = semantic_model.type_check(&check_type, arg_type);
            if !result.is_ok() {
                add_type_check_diagnostic(
                    context,
                    semantic_model,
                    *arg_ranges.get(idx)?,
                    &param_type,
                    arg_type,
                    result,
                );
            }
        }
    }

    Some(())
}

fn check_variadic_param_match_args(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    variadic_type: &LuaType,
    arg_types: &[LuaType],
    arg_ranges: &[TextRange],
) {
    for (arg_type, arg_range) in arg_types.iter().zip(arg_ranges.iter()) {
        let result = semantic_model.type_check(variadic_type, arg_type);
        if !result.is_ok() {
            add_type_check_diagnostic(
                context,
                semantic_model,
                *arg_range,
                variadic_type,
                arg_type,
                result,
            );
        }
    }
}

fn add_type_check_diagnostic(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    range: TextRange,
    param_type: &LuaType,
    expr_type: &LuaType,
    result: TypeCheckResult,
) {
    let db = semantic_model.get_db();
    match result {
        Ok(_) => return,
        Err(reason) => match reason {
            TypeCheckFailReason::TypeNotMatchWithReason(reason) => {
                context.add_diagnostic(DiagnosticCode::ParamTypeNotMatch, range, reason, None);
            }
            TypeCheckFailReason::TypeNotMatch => {
                context.add_diagnostic(
                    DiagnosticCode::ParamTypeNotMatch,
                    range,
                    t!(
                        "expected `%{source}` but found `%{found}`",
                        source = humanize_type(db, &param_type, RenderLevel::Simple),
                        found = humanize_type(db, &expr_type, RenderLevel::Simple)
                    )
                    .to_string(),
                    None,
                );
            }
            TypeCheckFailReason::TypeRecursion => {
                context.add_diagnostic(
                    DiagnosticCode::ParamTypeNotMatch,
                    range,
                    "type recursion".into(),
                    None,
                );
            }
        },
    }
}

fn get_call_source_type(
    semantic_model: &SemanticModel,
    call_expr: &LuaCallExpr,
) -> Option<LuaType> {
    if let Some(LuaExpr::IndexExpr(index_expr)) = call_expr.get_prefix_expr() {
        return if let Some(prefix_expr) = index_expr.get_prefix_expr() {
            let expr_type = semantic_model
                .infer_expr(prefix_expr.clone())
                .unwrap_or(LuaType::SelfInfer);
            Some(expr_type)
        } else {
            None
        };
    }
    None
}

/// Check if colon call is possible. This check can only be performed
/// when it's a colon call but not a colon definition.
#[allow(unused)]
fn check_first_param_colon_call(
    semantic_model: &SemanticModel,
    call_expr: LuaCallExpr,
    self_type: &LuaType,
) -> TypeCheckResult {
    if !matches!(self_type, LuaType::SelfInfer | LuaType::Any) {
        if let Some(LuaExpr::IndexExpr(index_expr)) = call_expr.get_prefix_expr() {
            // We need to narrow `SelfInfer` to the actual type
            return if let Some(prefix_expr) = index_expr.get_prefix_expr() {
                let expr_type = semantic_model
                    .infer_expr(prefix_expr.clone())
                    .unwrap_or(LuaType::SelfInfer);
                semantic_model.type_check(self_type, &expr_type)
            } else {
                Err(TypeCheckFailReason::TypeNotMatch)
            };
        }
    }
    Ok(())
}
