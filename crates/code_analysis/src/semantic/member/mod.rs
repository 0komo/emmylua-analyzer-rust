use crate::db_index::{LuaType, LuaTypeDeclId};

pub fn without_members(type_: &LuaType) -> bool {
    match type_ {
        LuaType::Nil
        | LuaType::Boolean
        | LuaType::BooleanConst(_)
        | LuaType::Integer
        | LuaType::IntegerConst(_)
        | LuaType::Number
        | LuaType::FloatConst(_)
        | LuaType::Function
        | LuaType::DocFunction(_)
        | LuaType::Table
        | LuaType::TableGeneric(_)
        | LuaType::Userdata
        | LuaType::Thread
        | LuaType::Unknown
        | LuaType::Any
        | LuaType::SelfInfer
        | LuaType::Extends(_)
        | LuaType::StrTplRef(_)
        | LuaType::TplRef(_)
        | LuaType::Array(_)
        | LuaType::MuliReturn(_) => true,
        _ => false,
    }
}

pub fn without_index_operator(type_: &LuaType) -> bool {
    match type_ {
        LuaType::Nil
        | LuaType::Boolean
        | LuaType::BooleanConst(_)
        | LuaType::Integer
        | LuaType::IntegerConst(_)
        | LuaType::Number
        | LuaType::FloatConst(_)
        | LuaType::Function
        | LuaType::DocFunction(_)
        | LuaType::Table
        | LuaType::Userdata
        | LuaType::Thread
        | LuaType::Unknown
        | LuaType::String
        | LuaType::StringConst(_)
        | LuaType::Io
        | LuaType::Any
        | LuaType::Extends(_)
        | LuaType::StrTplRef(_)
        | LuaType::TplRef(_)
        | LuaType::KeyOf(_)
        | LuaType::MuliReturn(_) => true,
        _ => false,
    }
}

pub fn get_buildin_type_map_type_id(type_: &LuaType) -> Option<LuaTypeDeclId> {
    match type_ {
        LuaType::String | LuaType::StringConst(_) => Some(LuaTypeDeclId::new("string")),
        LuaType::Io => Some(LuaTypeDeclId::new("io")),
        _ => None,
    }
}
