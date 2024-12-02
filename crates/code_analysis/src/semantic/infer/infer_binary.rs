use emmylua_parser::{BinaryOperator, LuaBinaryExpr};

use crate::db_index::{DbIndex, LuaOperatorMetaMethod, LuaType, TypeAssertion};

use super::{get_custom_type_operator, infer_config::LuaInferConfig, infer_expr, InferResult};

pub fn infer_binary_expr(
    db: &DbIndex,
    config: &mut LuaInferConfig,
    expr: LuaBinaryExpr,
) -> InferResult {
    let op = expr.get_op_token()?.get_op();
    let (left, right) = expr.get_exprs()?;
    let left_type = infer_expr(db, config, left)?;
    let right_type = infer_expr(db, config, right)?;

    match op {
        BinaryOperator::OpAdd => infer_binary_expr_add(db, left_type, right_type),
        BinaryOperator::OpSub => infer_binary_expr_sub(db, left_type, right_type),
        BinaryOperator::OpMul => infer_binary_expr_mul(db, left_type, right_type),
        BinaryOperator::OpDiv => infer_binary_expr_div(db, left_type, right_type),
        BinaryOperator::OpIDiv => infer_binary_expr_idiv(db, left_type, right_type),
        BinaryOperator::OpMod => infer_binary_expr_mod(db, left_type, right_type),
        BinaryOperator::OpPow => infer_binary_expr_pow(db, left_type, right_type),
        BinaryOperator::OpBAnd => infer_binary_expr_band(db, left_type, right_type),
        BinaryOperator::OpBOr => infer_binary_expr_bor(db, left_type, right_type),
        BinaryOperator::OpBXor => infer_binary_expr_bxor(db, left_type, right_type),
        BinaryOperator::OpShl => infer_binary_expr_shl(db, left_type, right_type),
        BinaryOperator::OpShr => infer_binary_expr_shr(db, left_type, right_type),
        BinaryOperator::OpConcat => infer_binary_expr_concat(db, left_type, right_type),
        BinaryOperator::OpLt
        | BinaryOperator::OpLe
        | BinaryOperator::OpGt
        | BinaryOperator::OpGe
        | BinaryOperator::OpEq
        | BinaryOperator::OpNe => Some(LuaType::Boolean),
        BinaryOperator::OpAnd => Some(right_type),
        BinaryOperator::OpOr => infer_binary_expr_or(left_type, right_type),
        BinaryOperator::OpNop => Some(left_type),
    }
}

fn infer_binary_custom_operator(
    db: &DbIndex,
    left: &LuaType,
    right: &LuaType,
    op: LuaOperatorMetaMethod,
) -> InferResult {
    let operators = get_custom_type_operator(db, left.clone(), op);
    if let Some(operators) = operators {
        for operator in operators {
            let first_param = operator.get_operands().get(0)?;
            if first_param == right {
                return Some(operator.get_result().clone());
            }
        }
    }

    let operators = get_custom_type_operator(db, right.clone(), op);
    if let Some(operators) = operators {
        for operator in operators {
            let first_param = operator.get_operands().get(0)?;
            if first_param == left {
                return Some(operator.get_result().clone());
            }
        }
    }

    match op {
        LuaOperatorMetaMethod::Add
        | LuaOperatorMetaMethod::Sub
        | LuaOperatorMetaMethod::Mul
        | LuaOperatorMetaMethod::Div
        | LuaOperatorMetaMethod::Mod
        | LuaOperatorMetaMethod::Pow => Some(LuaType::Number),
        LuaOperatorMetaMethod::IDiv
        | LuaOperatorMetaMethod::BAnd
        | LuaOperatorMetaMethod::BOr
        | LuaOperatorMetaMethod::BXor
        | LuaOperatorMetaMethod::Shl
        | LuaOperatorMetaMethod::Shr => Some(LuaType::Integer),

        LuaOperatorMetaMethod::Concat => Some(LuaType::String),
        _ => None,
    }
}

fn infer_binary_expr_add(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_number() && right.is_number() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 + int2))
            }
            (LuaType::FloatConst(num1), LuaType::FloatConst(num2)) => {
                Some(LuaType::FloatConst(num1 + num2))
            }
            (LuaType::IntegerConst(int1), LuaType::FloatConst(num2)) => {
                Some(LuaType::FloatConst((*int1 as f64 + *num2).into()))
            }
            (LuaType::FloatConst(num1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::FloatConst((*num1 + *int2 as f64).into()))
            }
            _ => {
                if left.is_integer() && right.is_integer() {
                    Some(LuaType::Integer)
                } else {
                    Some(LuaType::Number)
                }
            }
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::Add)
}

fn infer_binary_expr_sub(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_number() && right.is_number() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 - int2))
            }
            (LuaType::FloatConst(num1), LuaType::FloatConst(num2)) => {
                Some(LuaType::FloatConst(num1 - num2))
            }
            (LuaType::IntegerConst(int1), LuaType::FloatConst(num2)) => {
                Some(LuaType::FloatConst((*int1 as f64 - *num2).into()))
            }
            (LuaType::FloatConst(num1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::FloatConst((*num1 - *int2 as f64).into()))
            }
            _ => {
                if left.is_integer() && right.is_integer() {
                    Some(LuaType::Integer)
                } else {
                    Some(LuaType::Number)
                }
            }
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::Sub)
}

fn infer_binary_expr_mul(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_number() && right.is_number() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 * int2))
            }
            (LuaType::FloatConst(num1), LuaType::FloatConst(num2)) => {
                Some(LuaType::FloatConst(num1 * num2))
            }
            (LuaType::IntegerConst(int1), LuaType::FloatConst(num2)) => {
                Some(LuaType::FloatConst((*int1 as f64 * *num2).into()))
            }
            (LuaType::FloatConst(num1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::FloatConst((*num1 * *int2 as f64).into()))
            }
            _ => {
                if left.is_integer() && right.is_integer() {
                    Some(LuaType::Integer)
                } else {
                    Some(LuaType::Number)
                }
            }
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::Mul)
}

fn infer_binary_expr_div(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_number() && right.is_number() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::FloatConst((*int1 as f64 / *int2 as f64).into()))
            }
            (LuaType::FloatConst(num1), LuaType::FloatConst(num2)) => {
                Some(LuaType::FloatConst(num1 / num2))
            }
            (LuaType::IntegerConst(int1), LuaType::FloatConst(num2)) => {
                Some(LuaType::FloatConst((*int1 as f64 / *num2).into()))
            }
            (LuaType::FloatConst(num1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::FloatConst(*num1 / *int2 as f64))
            }
            _ => Some(LuaType::Number),
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::Div)
}

fn infer_binary_expr_idiv(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_integer() && right.is_integer() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 / int2))
            }
            _ => Some(LuaType::Integer),
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::IDiv)
}

fn infer_binary_expr_mod(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_integer() && right.is_integer() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 % int2))
            }
            _ => Some(LuaType::Integer),
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::Mod)
}

fn infer_binary_expr_pow(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_number() && right.is_number() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1.pow(*int2 as u32)))
            }
            (LuaType::FloatConst(num1), LuaType::IntegerConst(num2)) => {
                Some(LuaType::FloatConst(num1.powf(*num2 as f64).into()))
            }
            _ => Some(LuaType::Number),
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::Pow)
}

fn infer_binary_expr_band(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_integer() && right.is_integer() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 & int2))
            }
            _ => Some(LuaType::Integer),
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::BAnd)
}

fn infer_binary_expr_bor(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_integer() && right.is_integer() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 | int2))
            }
            _ => Some(LuaType::Integer),
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::BOr)
}

fn infer_binary_expr_bxor(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_integer() && right.is_integer() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 ^ int2))
            }
            _ => Some(LuaType::Integer),
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::BXor)
}

fn infer_binary_expr_shl(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_integer() && right.is_integer() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 << int2))
            }
            _ => Some(LuaType::Integer),
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::Shl)
}

fn infer_binary_expr_shr(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_integer() && right.is_integer() {
        return match (&left, &right) {
            (LuaType::IntegerConst(int1), LuaType::IntegerConst(int2)) => {
                Some(LuaType::IntegerConst(int1 >> int2))
            }
            _ => Some(LuaType::Integer),
        };
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::Shr)
}

fn infer_binary_expr_concat(db: &DbIndex, left: LuaType, right: LuaType) -> InferResult {
    if left.is_number() || left.is_string() || right.is_number() || right.is_string() {
        match (&left, &right) {
            (LuaType::StringConst(s1), LuaType::StringConst(s2)) => {
                return Some(LuaType::StringConst(format!("{}{}", *s1, *s2).into()));
            }
            (LuaType::StringConst(s1), LuaType::IntegerConst(i)) => {
                return Some(LuaType::StringConst(format!("{}{}", *s1, i).into()));
            }
            (LuaType::IntegerConst(i), LuaType::StringConst(s2)) => {
                return Some(LuaType::StringConst(format!("{}{}", i, *s2).into()));
            }
            _ => return Some(LuaType::String),
        }
    }

    infer_binary_custom_operator(db, &left, &right, LuaOperatorMetaMethod::Concat)
}

fn infer_binary_expr_or(left: LuaType, right: LuaType) -> InferResult {
    if !right.is_boolean() {
        if left.is_unknown() {
            return Some(right);
        }
        return Some(TypeAssertion::Exist.tighten_type(left));
    }

    Some(LuaType::Boolean)
}
