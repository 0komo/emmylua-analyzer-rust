mod infer_binary;
mod infer_call;
mod infer_cache;
mod infer_index;
mod infer_name;
mod infer_table;
mod infer_unary;
mod infer_call_func;
mod test;

use emmylua_parser::{LuaAstNode, LuaClosureExpr, LuaExpr, LuaLiteralExpr, LuaLiteralToken};
use infer_binary::infer_binary_expr;
use infer_call::infer_call_expr;
use infer_cache::ExprCache;
pub use infer_cache::LuaInferCache;
use infer_index::infer_index_expr;
use infer_name::{infer_name_expr, infer_param};
use infer_table::infer_table_expr;
pub use infer_table::infer_table_should_be;
pub use infer_call_func::infer_call_expr_func;
use infer_unary::infer_unary_expr;
use smol_str::SmolStr;

use crate::{
    db_index::{DbIndex, LuaOperator, LuaOperatorMetaMethod, LuaSignatureId, LuaType},
    InFiled, LuaMultiReturn,
};

pub type InferResult = Option<LuaType>;

pub fn infer_expr(db: &DbIndex, cache: &mut LuaInferCache, expr: LuaExpr) -> InferResult {
    let syntax_id = expr.get_syntax_id();
    match cache.get_cache_expr_type(&syntax_id) {
        Some(ExprCache::Cache(ty)) => return Some(ty.clone()),
        Some(ExprCache::ReadyCache) => return Some(LuaType::Unknown),
        None => {}
    }

    // for @as
    let file_id = cache.get_file_id();
    let in_filed_syntax_id = InFiled::new(file_id, syntax_id);
    if let Some(force_type) = db.get_type_index().get_as_force_type(&in_filed_syntax_id) {
        cache.cache_expr_type(syntax_id, force_type.clone());
        return Some(force_type.clone());
    }

    cache.mark_expr_ready_cache(syntax_id);
    let result_type = match expr {
        LuaExpr::CallExpr(call_expr) => infer_call_expr(db, cache, call_expr),
        LuaExpr::TableExpr(table_expr) => infer_table_expr(db, cache, table_expr),
        LuaExpr::LiteralExpr(literal_expr) => infer_literal_expr(db, cache, literal_expr),
        LuaExpr::BinaryExpr(binary_expr) => infer_binary_expr(db, cache, binary_expr),
        LuaExpr::UnaryExpr(unary_expr) => infer_unary_expr(db, cache, unary_expr),
        LuaExpr::ClosureExpr(closure_expr) => infer_closure_expr(db, cache, closure_expr),
        LuaExpr::ParenExpr(paren_expr) => infer_expr(db, cache, paren_expr.get_expr()?),
        LuaExpr::NameExpr(name_expr) => infer_name_expr(db, cache, name_expr),
        LuaExpr::IndexExpr(index_expr) => infer_index_expr(db, cache, index_expr),
    };

    if let Some(result_type) = &result_type {
        cache.cache_expr_type(syntax_id, result_type.clone());
    } else {
        cache.clear_expr_cache(&syntax_id);
    }

    result_type
}

fn infer_literal_expr(db: &DbIndex, config: &LuaInferCache, expr: LuaLiteralExpr) -> InferResult {
    match expr.get_literal()? {
        LuaLiteralToken::Nil(_) => Some(LuaType::Nil),
        LuaLiteralToken::Bool(bool) => Some(LuaType::BooleanConst(bool.is_true())),
        LuaLiteralToken::Number(num) => {
            if num.is_int() {
                Some(LuaType::IntegerConst(num.get_int_value()))
            } else if num.is_float() {
                Some(LuaType::FloatConst(num.get_float_value()))
            } else {
                Some(LuaType::Number)
            }
        }
        LuaLiteralToken::String(str) => {
            Some(LuaType::StringConst(SmolStr::new(str.get_value()).into()))
        }
        LuaLiteralToken::Dots(_) => {
            let file_id = config.get_file_id();
            let range = expr.get_range();

            let decl_id = db
                .get_reference_index()
                .get_local_reference(&file_id)
                .and_then(|file_ref| file_ref.get_decl_id(&range));

            let decl_type = match decl_id.and_then(|id| db.get_decl_index().get_decl(&id)) {
                Some(decl) if decl.is_global() => LuaType::Any,
                Some(decl) if decl.is_param() => {
                    let base = infer_param(db, decl).unwrap_or(LuaType::Unknown);
                    LuaType::MuliReturn(LuaMultiReturn::Base(base).into())
                }
                _ => LuaType::Any, // 默认返回 Any
            };

            Some(decl_type)
        }
        _ => None,
    }
}

fn infer_closure_expr(
    _: &DbIndex,
    config: &LuaInferCache,
    closure: LuaClosureExpr,
) -> InferResult {
    let signature_id = LuaSignatureId::from_closure(config.get_file_id(), &closure);
    Some(LuaType::Signature(signature_id))
}

fn get_custom_type_operator(
    db: &DbIndex,
    operand_type: LuaType,
    op: LuaOperatorMetaMethod,
) -> Option<Vec<&LuaOperator>> {
    if operand_type.is_custom_type() {
        let type_id = match operand_type {
            LuaType::Ref(type_id) => type_id,
            LuaType::Def(type_id) => type_id,
            _ => return None,
        };
        let ops = db.get_operator_index().get_operators_by_type(&type_id)?;
        let op_ids = ops.get(&op)?;
        let operators = op_ids
            .iter()
            .filter_map(|id| db.get_operator_index().get_operator(id))
            .collect();

        Some(operators)
    } else {
        None
    }
}
