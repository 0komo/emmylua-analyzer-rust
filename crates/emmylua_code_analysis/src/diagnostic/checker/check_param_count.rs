use emmylua_parser::{
    LuaAstNode, LuaAstToken, LuaCallExpr, LuaExpr, LuaGeneralToken, LuaLiteralToken,
};

use crate::{DiagnosticCode, LuaType, SemanticModel};

use super::DiagnosticContext;

pub const CODES: &[DiagnosticCode] = &[
    DiagnosticCode::MissingParameter,
    DiagnosticCode::RedundantParameter,
];

pub fn check(context: &mut DiagnosticContext, semantic_model: &SemanticModel) -> Option<()> {
    for call_expr in semantic_model.get_root().descendants::<LuaCallExpr>() {
        check_call_expr(context, semantic_model, call_expr);
    }

    Some(())
}

fn check_call_expr(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    call_expr: LuaCallExpr,
) -> Option<()> {
    let func = semantic_model.infer_call_expr_func(call_expr.clone(), None)?;
    let params = func.get_params();
    let call_args = call_expr.get_args_list()?.get_args().collect::<Vec<_>>();
    let mut call_args_count = call_args.len();
    // 根据冒号定义与冒号调用的情况来调整调用参数的数量
    let colon_call = call_expr.is_colon_call();
    let colon_define = func.is_colon_define();
    match (colon_call, colon_define) {
        (true, true) | (false, false) => {}
        (false, true) => {
            if call_args_count > 0 {
                call_args_count -= 1;
            }
        }
        (true, false) => {
            call_args_count += 1;
        }
    }

    // Check for missing parameters
    if call_args_count < params.len() {
        // 调用参数包含 `...`
        for arg in call_args.iter() {
            if let LuaExpr::LiteralExpr(literal_expr) = arg {
                if let Some(literal_token) = literal_expr.get_literal() {
                    if let LuaLiteralToken::Dots(_) = literal_token {
                        return Some(());
                    }
                }
            }
        }
        // 参数调用中最后一个参数是多返回值
        if let Some(last_arg) = call_args.last() {
            if let Some(LuaType::MuliReturn(types)) = semantic_model.infer_expr(last_arg.clone()) {
                let len = types.get_len().unwrap_or(0);
                call_args_count = call_args_count + len as usize - 1;
                if call_args_count >= params.len() {
                    return Some(());
                }
            }
        }

        let mut miss_parameter_info = Vec::new();

        for i in call_args_count..params.len() {
            let param_info = params.get(i)?;
            if param_info.0 == "..." {
                break;
            }

            let typ = param_info.1.clone();
            if let Some(typ) = typ {
                if !typ.is_any() && !typ.is_unknown() && !typ.is_optional() {
                    miss_parameter_info
                        .push(t!("missing parameter: %{name}", name = param_info.0,));
                }
            }
        }

        if !miss_parameter_info.is_empty() {
            let right_paren = call_expr
                .get_args_list()?
                .tokens::<LuaGeneralToken>()
                .last()?;
            context.add_diagnostic(
                DiagnosticCode::MissingParameter,
                right_paren.get_range(),
                t!(
                    "expected %{num} parameters but found %{found_num}. %{infos}",
                    num = params.len(),
                    found_num = call_args_count,
                    infos = miss_parameter_info.join(" \n ")
                )
                .to_string(),
                None,
            );
        }
    }
    // Check for redundant parameters
    else if call_args_count > params.len() {
        // 参数定义中最后一个参数是 `...`
        if params.last().map_or(false, |(name, _)| name == "...") {
            return Some(());
        }

        let mut adjusted_index = 0;
        if colon_call != colon_define {
            adjusted_index = if colon_define && !colon_call { -1 } else { 1 };
        }

        for (i, arg) in call_args.iter().enumerate() {
            let param_index = i as isize + adjusted_index;

            if param_index < 0 || param_index < params.len() as isize {
                continue;
            }

            context.add_diagnostic(
                DiagnosticCode::RedundantParameter,
                arg.get_range(),
                t!(
                    "expected %{num} parameters but found %{found_num}",
                    num = params.len(),
                    found_num = call_args_count,
                )
                .to_string(),
                None,
            );
        }
    }

    Some(())
}
