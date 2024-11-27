use std::collections::HashMap;

use emmylua_parser::{LuaAstNode, LuaClosureExpr};
use rowan::TextSize;

use crate::{db_index::LuaType, FileId};

#[derive(Debug)]
pub struct LuaSignature {
    pub generic_params: Vec<(String, Option<LuaType>)>,
    pub overloads: Vec<LuaType>,
    pub param_docs: HashMap<String, LuaDocParamInfo>,
    pub params: Vec<String>,
    pub return_docs: Vec<LuaDocReturnInfo>,
    pub(crate) resolve_return: bool,
}

impl LuaSignature {
    pub fn new() -> Self {
        Self {
            generic_params: Vec::new(),
            overloads: Vec::new(),
            param_docs: HashMap::new(),
            params: Vec::new(),
            return_docs: Vec::new(),
            resolve_return: false,
        }
    }
}

impl LuaSignature {
    pub fn is_generic(&self) -> bool {
        !self.generic_params.is_empty()
    }

    pub fn is_resolve_return(&self) -> bool {
        self.resolve_return || !self.return_docs.is_empty()
    }
}

#[derive(Debug)]
pub struct LuaDocParamInfo {
    pub name: String,
    pub type_ref: LuaType,
    pub nullable: bool,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct LuaDocReturnInfo {
    pub name: Option<String>,
    pub type_ref: LuaType,
    pub description: Option<String>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct LuaSignatureId {
    file_id: FileId,
    position: TextSize,
}

impl LuaSignatureId {
    pub fn new(file_id: FileId, closure: &LuaClosureExpr) -> Self {
        Self { file_id, position: closure.get_position() }
    }

    pub fn get_file_id(&self) -> FileId {
        self.file_id
    }

    pub fn get_position(&self) -> TextSize {
        self.position
    }
}