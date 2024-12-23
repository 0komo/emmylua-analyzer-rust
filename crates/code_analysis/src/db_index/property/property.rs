use emmylua_parser::VisibilityKind;
use serde::{Deserialize, Serialize};

use crate::db_index::{member::LuaMemberId, LuaDeclId, LuaSignatureId, LuaTypeDeclId};

use super::version::LuaVersionCond;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LuaProperty {
    pub id: LuaPropertyId,
    pub description: Option<Box<String>>,
    pub visibility: Option<VisibilityKind>,
    pub source: Option<Box<String>>,
    pub is_nodiscard: bool,
    pub is_deprecated: bool,
    pub deprecated_message: Option<Box<String>>,
    pub version_conds: Option<Box<Vec<LuaVersionCond>>>,
    pub is_async: bool,
}

impl LuaProperty {
    pub fn new(id: LuaPropertyId) -> Self {
        Self {
            id,
            description: None,
            visibility: None,
            source: None,
            is_nodiscard: false,
            is_deprecated: false,
            deprecated_message: None,
            version_conds: None,
            is_async: false,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct LuaPropertyId {
    id: u32,
}

impl LuaPropertyId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum LuaPropertyOwnerId {
    TypeDecl(LuaTypeDeclId),
    Member(LuaMemberId),
    LuaDecl(LuaDeclId),
    Signature(LuaSignatureId),
}

impl From<LuaDeclId> for LuaPropertyOwnerId {
    fn from(id: LuaDeclId) -> Self {
        LuaPropertyOwnerId::LuaDecl(id)
    }
}

impl From<LuaTypeDeclId> for LuaPropertyOwnerId {
    fn from(id: LuaTypeDeclId) -> Self {
        LuaPropertyOwnerId::TypeDecl(id)
    }
}

impl From<LuaMemberId> for LuaPropertyOwnerId {
    fn from(id: LuaMemberId) -> Self {
        LuaPropertyOwnerId::Member(id)
    }
}

impl From<LuaSignatureId> for LuaPropertyOwnerId {
    fn from(id: LuaSignatureId) -> Self {
        LuaPropertyOwnerId::Signature(id)
    }
}
