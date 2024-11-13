use rowan::{TextRange, TextSize};

use crate::{db_index::LuaType, FileId};

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum LuaDecl {
    Local {
        name: String,
        file_id: FileId,
        range: TextRange,
        attrib: Option<LocalAttribute>,
        decl_type: Option<LuaType>,
    },
    Global {
        name: String,
        file_id: FileId,
        range: TextRange,
        decl_type: Option<LuaType>,
    },
}

impl LuaDecl {
    pub fn get_file_id(&self) -> FileId {
        match self {
            LuaDecl::Local { file_id, .. } => *file_id,
            LuaDecl::Global { file_id, .. } => *file_id,
        }
    }

    pub fn get_id(&self) -> LuaDeclId {
        match self {
            LuaDecl::Local { file_id, .. } => LuaDeclId::new(*file_id, self.get_position()),
            LuaDecl::Global { file_id, .. } => LuaDeclId::new(*file_id, self.get_position()),
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            LuaDecl::Local { name, .. } => name,
            LuaDecl::Global { name, .. } => name,
        }
    }

    pub fn get_position(&self) -> TextSize {
        match self {
            LuaDecl::Local { range, .. } => range.start(),
            LuaDecl::Global { range, .. } => range.start(),
        }
    }
    #[allow(unused)]
    pub fn get_range(&self) -> TextRange {
        match self {
            LuaDecl::Local { range, .. } => *range,
            LuaDecl::Global { range, .. } => *range,
        }
    }

    pub fn get_type(&self) -> Option<&LuaType> {
        match self {
            LuaDecl::Local { decl_type, .. } => decl_type.as_ref(),
            LuaDecl::Global { decl_type, .. } => decl_type.as_ref(),
        }
    }

    pub (crate) fn set_decl_type(&mut self, decl_type: LuaType) {
        match self {
            LuaDecl::Local { decl_type: dt, .. } => *dt = Some(decl_type),
            LuaDecl::Global { decl_type: dt, .. } => *dt = Some(decl_type),
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, LuaDecl::Local { .. })
    }

    pub fn is_global(&self) -> bool {
        matches!(self, LuaDecl::Global { .. })
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct LuaDeclId {
    pub file_id: FileId,
    pub position: TextSize,
}

impl LuaDeclId {
    pub fn new(file_id: FileId, position: TextSize) -> Self {
        Self { file_id, position }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum LocalAttribute {
    Const,
    Close,
    IterConst,
}
