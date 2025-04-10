use emmylua_parser::{
    BinaryOperator, LuaAssignStat, LuaAst, LuaAstNode, LuaBinaryExpr, LuaBlock, LuaCallArgList,
    LuaCallExpr, LuaCallExprStat, LuaCommentOwner, LuaDocTag, LuaExpr, LuaLiteralToken, LuaStat,
    LuaVarExpr, UnaryOperator,
};
use rowan::TextRange;
use smol_str::SmolStr;

use crate::{
    db_index::{LuaType, TypeAssertion},
    DbIndex, FileId, LuaDeclId, LuaFlowChain, LuaMemberId, LuaTypeDeclId, LuaTypeOwner, VarRefId,
};


pub fn analyze_ref_expr(
    db: &mut DbIndex,
    flow_chain: &mut LuaFlowChain,
    var_expr: &LuaVarExpr,
    var_ref_id: &VarRefId,
) -> Option<()> {
    let parent = var_expr.get_parent::<LuaAst>()?;
    broadcast_up(
        db,
        flow_chain,
        &var_ref_id,
        parent,
        LuaAst::cast(var_expr.syntax().clone())?,
        TypeAssertion::Exist,
    );

    Some(())
}

pub fn analyze_ref_assign(
    db: &mut DbIndex,
    flow_chain: &mut LuaFlowChain,
    var_expr: &LuaVarExpr,
    var_ref_id: &VarRefId,
    file_id: FileId,
) -> Option<()> {
    let assign_stat = var_expr.get_parent::<LuaAssignStat>()?;
    if is_decl_assign_stat(assign_stat.clone()).unwrap_or(false) {
        let type_owner = match var_expr {
            LuaVarExpr::IndexExpr(index_expr) => {
                let member_id = LuaMemberId::new(index_expr.get_syntax_id(), file_id);
                LuaTypeOwner::Member(member_id)
            }
            LuaVarExpr::NameExpr(name_expr) => {
                let decl_id = LuaDeclId::new(file_id, name_expr.get_position());
                LuaTypeOwner::Decl(decl_id)
            }
        };
        if let Some(type_cache) = db.get_type_index().get_type_cache(&type_owner) {
            let type_assert = TypeAssertion::Narrow(type_cache.as_type().clone());
            broadcast_down(
                db,
                flow_chain,
                var_ref_id,
                LuaAst::LuaAssignStat(assign_stat),
                type_assert,
                true,
            );
        }

        return None;
    }

    let (var_exprs, value_exprs) = assign_stat.get_var_and_expr_list();
    let var_index = var_exprs
        .iter()
        .position(|it| it.get_position() == var_expr.get_position())?;

    if value_exprs.len() == 0 {
        return None;
    }

    let (value_expr, idx) = if let Some(expr) = value_exprs.get(var_index) {
        (expr.clone(), 0)
    } else {
        (
            value_exprs.last()?.clone(),
            (var_index - (value_exprs.len() - 1)) as i32,
        )
    };

    let type_assert = TypeAssertion::Reassign((value_expr.get_syntax_id(), idx));
    broadcast_down(
        db,
        flow_chain,
        var_ref_id,
        LuaAst::LuaAssignStat(assign_stat),
        type_assert,
        true,
    );

    Some(())
}

fn is_decl_assign_stat(assign_stat: LuaAssignStat) -> Option<bool> {
    for comment in assign_stat.get_comments() {
        for tag in comment.get_doc_tags() {
            match tag {
                LuaDocTag::Type(_)
                | LuaDocTag::Class(_)
                | LuaDocTag::Module(_)
                | LuaDocTag::Enum(_) => {
                    return Some(true);
                }
                _ => {}
            }
        }
    }
    Some(false)
}

fn broadcast_up(
    db: &mut DbIndex,
    flow_chain: &mut LuaFlowChain,
    var_ref_id: &VarRefId,
    parent: LuaAst,
    origin: LuaAst,
    type_assert: TypeAssertion,
) -> Option<()> {
    let actual_range = origin.get_range();
    match parent {
        LuaAst::LuaIfStat(if_stat) => {
            // this mean the name_expr is a condition and the name_expr is not nil and is not false
            if let Some(block) = if_stat.get_block() {
                flow_chain.add_type_assert(
                    var_ref_id,
                    type_assert.clone(),
                    block.get_range(),
                    actual_range,
                );
            }

            if let Some(ne_type_assert) = type_assert.get_negation() {
                if let Some(else_stat) = if_stat.get_else_clause() {
                    let block_range = else_stat.get_range();
                    flow_chain.add_type_assert(
                        var_ref_id,
                        ne_type_assert.clone(),
                        block_range,
                        actual_range,
                    );
                } else if is_block_has_return(if_stat.get_block()).unwrap_or(false) {
                    let parent_block = if_stat.get_parent::<LuaBlock>()?;
                    let parent_range = parent_block.get_range();
                    let if_range = if_stat.get_range();
                    if if_range.end() < parent_range.end() {
                        let range = TextRange::new(if_range.end(), parent_range.end());
                        flow_chain.add_type_assert(
                            var_ref_id,
                            ne_type_assert.clone(),
                            range,
                            actual_range,
                        );
                    }
                }
                for else_if_clause in if_stat.get_else_if_clause_list() {
                    let block_range = else_if_clause.get_range();
                    flow_chain.add_type_assert(
                        var_ref_id,
                        ne_type_assert.clone(),
                        block_range,
                        actual_range,
                    );
                }
            }
        }
        LuaAst::LuaWhileStat(while_stat) => {
            // this mean the name_expr is a condition and the name_expr is not nil and is not false
            let block = while_stat.get_block()?;
            flow_chain.add_type_assert(var_ref_id, type_assert, block.get_range(), actual_range);
        }
        LuaAst::LuaElseIfClauseStat(else_if_clause_stat) => {
            // this mean the name_expr is a condition and the name_expr is not nil and is not false
            let block = else_if_clause_stat.get_block()?;
            flow_chain.add_type_assert(var_ref_id, type_assert, block.get_range(), actual_range);
        }
        LuaAst::LuaParenExpr(paren_expr) => {
            broadcast_up(
                db,
                flow_chain,
                var_ref_id,
                paren_expr.get_parent::<LuaAst>()?,
                LuaAst::LuaParenExpr(paren_expr),
                type_assert,
            );
        }
        LuaAst::LuaBinaryExpr(binary_expr) => {
            let op = binary_expr.get_op_token()?;
            match op.get_op() {
                BinaryOperator::OpAnd => {
                    let (left, right) = binary_expr.get_exprs()?;
                    if left.get_position() == origin.get_position() {
                        flow_chain.add_type_assert(
                            var_ref_id,
                            type_assert.clone(),
                            right.get_range(),
                            actual_range,
                        );
                    }

                    broadcast_up(
                        db,
                        flow_chain,
                        var_ref_id,
                        binary_expr.get_parent::<LuaAst>()?,
                        LuaAst::LuaBinaryExpr(binary_expr),
                        type_assert,
                    );
                }
                BinaryOperator::OpOr => {
                    let (left, right) = binary_expr.get_exprs()?;
                    if left.get_position() == origin.get_position() {
                        if let Some(ne) = type_assert.get_negation() {
                            flow_chain.add_type_assert(var_ref_id, ne, right.get_range(), actual_range);
                        }
                    }
                    broadcast_up(
                        db,
                        flow_chain,
                        var_ref_id,
                        binary_expr.get_parent::<LuaAst>()?,
                        LuaAst::LuaBinaryExpr(binary_expr),
                        type_assert,
                    );
                }
                BinaryOperator::OpEq => {
                    let (left, right) = binary_expr.get_exprs()?;
                    let expr = if left.get_position() == origin.get_position() {
                        right
                    } else {
                        left
                    };

                    if let LuaExpr::LiteralExpr(literal) = expr {
                        let type_assert = match literal.get_literal()? {
                            LuaLiteralToken::Nil(_) => TypeAssertion::NotExist,
                            LuaLiteralToken::Bool(b) => {
                                if b.is_true() {
                                    TypeAssertion::Exist
                                } else {
                                    TypeAssertion::NotExist
                                }
                            }
                            LuaLiteralToken::Number(i) => {
                                if i.is_int() {
                                    TypeAssertion::Narrow(LuaType::IntegerConst(i.get_int_value()))
                                } else {
                                    TypeAssertion::Narrow(LuaType::Number)
                                }
                            }
                            LuaLiteralToken::String(s) => TypeAssertion::Narrow(
                                LuaType::StringConst(SmolStr::new(s.get_value()).into()),
                            ),
                            _ => return None,
                        };

                        broadcast_up(
                            db,
                            flow_chain,
                            var_ref_id,
                            binary_expr.get_parent::<LuaAst>()?,
                            LuaAst::LuaBinaryExpr(binary_expr),
                            type_assert,
                        );
                    }
                }
                BinaryOperator::OpNe => {
                    let (left, right) = binary_expr.get_exprs()?;
                    let expr = if left.get_position() == origin.get_position() {
                        right
                    } else {
                        left
                    };

                    if let LuaExpr::LiteralExpr(literal) = expr {
                        let type_assert = match literal.get_literal()? {
                            LuaLiteralToken::Nil(_) => TypeAssertion::Exist,
                            LuaLiteralToken::Bool(b) => {
                                if b.is_true() {
                                    TypeAssertion::NotExist
                                } else {
                                    TypeAssertion::Exist
                                }
                            }
                            LuaLiteralToken::Number(i) => {
                                if i.is_int() {
                                    TypeAssertion::Remove(LuaType::IntegerConst(i.get_int_value()))
                                } else {
                                    TypeAssertion::Remove(LuaType::Number)
                                }
                            }
                            LuaLiteralToken::String(s) => TypeAssertion::Remove(
                                LuaType::StringConst(SmolStr::new(s.get_value()).into()),
                            ),
                            _ => return None,
                        };

                        broadcast_up(
                            db,
                            flow_chain,
                            var_ref_id,
                            binary_expr.get_parent::<LuaAst>()?,
                            LuaAst::LuaBinaryExpr(binary_expr),
                            type_assert,
                        );
                    }
                }

                _ => {}
            }
        }
        LuaAst::LuaCallArgList(call_args_list) => {
            infer_call_arg_list(db, flow_chain, type_assert, var_ref_id, call_args_list)?;
        }
        LuaAst::LuaUnaryExpr(unary_expr) => {
            let op = unary_expr.get_op_token()?;
            match op.get_op() {
                UnaryOperator::OpNot => {
                    if let Some(ne_type_assert) = type_assert.get_negation() {
                        broadcast_up(
                            db,
                            flow_chain,
                            var_ref_id,
                            unary_expr.get_parent::<LuaAst>()?,
                            LuaAst::LuaUnaryExpr(unary_expr),
                            ne_type_assert,
                        );
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Some(())
}

fn broadcast_down(
    db: &mut DbIndex,
    flow_chain: &mut LuaFlowChain,
    var_ref_id: &VarRefId,
    node: LuaAst,
    type_assert: TypeAssertion,
    continue_broadcast_outside: bool,
) -> Option<()> {
    let parent_block = node.get_parent::<LuaBlock>()?;
    let parent_range = parent_block.get_range();
    let range = node.get_range();
    if range.end() < parent_range.end() {
        let range = TextRange::new(range.end(), parent_range.end());
        flow_chain.add_type_assert(var_ref_id, type_assert.clone(), range, range);
    }

    if continue_broadcast_outside {
        broadcast_outside(db, flow_chain, var_ref_id, parent_block, type_assert);
    }

    Some(())
}

fn broadcast_outside(
    db: &mut DbIndex,
    flow_chain: &mut LuaFlowChain,
    var_ref_id: &VarRefId,
    node: LuaBlock,
    type_assert: TypeAssertion,
) -> Option<()> {
    let parent = node.get_parent::<LuaAst>()?;
    match &parent {
        LuaAst::LuaIfStat(_)
        | LuaAst::LuaDoStat(_)
        | LuaAst::LuaWhileStat(_)
        | LuaAst::LuaForStat(_)
        | LuaAst::LuaForRangeStat(_)
        | LuaAst::LuaRepeatStat(_) => {
            broadcast_down(db, flow_chain, var_ref_id, parent, type_assert, false);
        }
        LuaAst::LuaElseIfClauseStat(_) | LuaAst::LuaElseClauseStat(_) => {
            broadcast_down(
                db,
                flow_chain,
                var_ref_id,
                parent.get_parent::<LuaAst>()?,
                type_assert,
                false,
            );
        }
        _ => {}
    }

    Some(())
}

fn infer_call_arg_list(
    db: &mut DbIndex,
    flow_chain: &mut LuaFlowChain,
    type_assert: TypeAssertion,
    var_ref_id: &VarRefId,
    call_arg: LuaCallArgList,
) -> Option<()> {
    let parent = call_arg.get_parent::<LuaAst>()?;
    match parent {
        LuaAst::LuaCallExpr(call_expr) => {
            if call_expr.is_type() {
                infer_lua_type_assert(db, flow_chain, var_ref_id, call_expr);
            } else if call_expr.is_assert() {
                infer_lua_assert(db, flow_chain, type_assert, var_ref_id, call_expr);
            }
        }
        _ => {}
    }

    Some(())
}

fn infer_lua_type_assert(
    db: &mut DbIndex,
    flow_chain: &mut LuaFlowChain,
    var_ref_id: &VarRefId,
    call_expr: LuaCallExpr,
) -> Option<()> {
    let binary_expr = call_expr.get_parent::<LuaBinaryExpr>()?;
    let op = binary_expr.get_op_token()?;
    let mut is_eq = true;
    match op.get_op() {
        BinaryOperator::OpEq => {}
        BinaryOperator::OpNe => {
            is_eq = false;
        }
        _ => return None,
    };

    let operands = binary_expr.get_exprs()?;
    let literal_expr = if let LuaExpr::LiteralExpr(literal) = operands.0 {
        literal
    } else if let LuaExpr::LiteralExpr(literal) = operands.1 {
        literal
    } else {
        return None;
    };

    let type_literal = match literal_expr.get_literal()? {
        LuaLiteralToken::String(string) => string.get_value(),
        _ => return None,
    };

    let mut type_assert = match type_literal.as_str() {
        "number" => TypeAssertion::Narrow(LuaType::Number),
        "string" => TypeAssertion::Narrow(LuaType::String),
        "boolean" => TypeAssertion::Narrow(LuaType::Boolean),
        "table" => TypeAssertion::Narrow(LuaType::Table),
        "function" => TypeAssertion::Narrow(LuaType::Function),
        "thread" => TypeAssertion::Narrow(LuaType::Thread),
        "userdata" => TypeAssertion::Narrow(LuaType::Userdata),
        "nil" => TypeAssertion::Narrow(LuaType::Nil),
        // extend usage
        str => TypeAssertion::Narrow(LuaType::Ref(LuaTypeDeclId::new(str))),
    };

    if !is_eq {
        type_assert = type_assert.get_negation()?;
    }

    broadcast_up(
        db,
        flow_chain,
        var_ref_id,
        binary_expr.get_parent::<LuaAst>()?,
        LuaAst::LuaBinaryExpr(binary_expr),
        type_assert,
    );

    Some(())
}

fn is_block_has_return(block: Option<LuaBlock>) -> Option<bool> {
    if let Some(block) = block {
        for stat in block.get_stats() {
            if is_stat_change_flow(stat.clone()).unwrap_or(false) {
                return Some(true);
            }
        }
    }

    Some(false)
}

fn is_stat_change_flow(stat: LuaStat) -> Option<bool> {
    match stat {
        LuaStat::CallExprStat(call_stat) => {
            let call_expr = call_stat.get_call_expr()?;
            let prefix_expr = call_expr.get_prefix_expr()?;
            if let LuaExpr::NameExpr(name_expr) = prefix_expr {
                let name = name_expr.get_name_text()?;
                if name == "error" {
                    return Some(true);
                }
            }
            Some(false)
        }
        LuaStat::ReturnStat(_) => Some(true),
        LuaStat::DoStat(do_stat) => Some(is_block_has_return(do_stat.get_block()).unwrap_or(false)),
        _ => Some(false),
    }
}

fn infer_lua_assert(
    db: &mut DbIndex,
    flow_chain: &mut LuaFlowChain,
    type_assert: TypeAssertion,
    var_ref_id: &VarRefId,
    call_expr: LuaCallExpr,
) -> Option<()> {
    broadcast_down(
        db,
        flow_chain,
        var_ref_id,
        LuaAst::LuaCallExprStat(call_expr.get_parent::<LuaCallExprStat>()?),
        type_assert.clone(),
        true,
    );
    Some(())
}
