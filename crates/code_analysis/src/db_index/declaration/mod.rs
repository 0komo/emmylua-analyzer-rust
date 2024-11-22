mod decl;
mod decl_tree;
mod scope;
mod test;

use std::collections::HashMap;

pub use decl::{LocalAttribute, LuaDecl, LuaDeclId};
pub use decl_tree::{LuaDeclOrMemberId, LuaDeclarationTree};
use internment::ArcIntern;
pub use scope::{LuaScopeId, LuaScopeKind};

use crate::FileId;

use super::{traits::LuaIndex, LuaMemberKey, LuaType};

#[derive(Debug)]
pub struct LuaDeclIndex {
    decl_trees: HashMap<FileId, LuaDeclarationTree>,
    global_decl: HashMap<LuaMemberKey, Vec<LuaDeclId>>,
}

impl LuaDeclIndex {
    pub fn new() -> Self {
        Self {
            decl_trees: HashMap::new(),
            global_decl: HashMap::new(),
        }
    }

    pub fn add_global_decl(&mut self, name: String, decl_id: LuaDeclId) {
        let key = ArcIntern::new(name);
        self.global_decl
            .entry(LuaMemberKey::Name(key.clone()))
            .or_insert_with(Vec::new)
            .push(decl_id);
    }

    pub fn add_decl_tree(&mut self, tree: LuaDeclarationTree) {
        self.decl_trees.insert(tree.file_id(), tree);
    }

    pub fn get_decl_tree(&self, file_id: &FileId) -> Option<&LuaDeclarationTree> {
        self.decl_trees.get(file_id)
    }

    pub fn get_decl(&self, decl_id: &LuaDeclId) -> Option<&LuaDecl> {
        let tree = self.decl_trees.get(&decl_id.file_id)?;
        tree.get_decl(decl_id)
    }

    pub fn get_decl_mut(&mut self, decl_id: &LuaDeclId) -> Option<&mut LuaDecl> {
        let tree = self.decl_trees.get_mut(&decl_id.file_id)?;
        tree.get_decl_mut(*decl_id)
    }

    pub fn get_global_decl_type(&self, key: &LuaMemberKey) -> Option<LuaType> {
        let decls = self.global_decl.get(key)?;
        if decls.len() == 1 {
            let decl = self.get_decl(&decls[0])?;
            return Some(decl.get_type()?.clone());
        }

        let mut valid_type = LuaType::Unknown;
        for decl_id in decls {
            let decl = self.get_decl(decl_id)?;
            let ty = decl.get_type();
            if let Some(ty) = ty {
                if ty.is_def() || ty.is_ref() {
                    return Some(ty.clone());
                }

                if valid_type == LuaType::Unknown {
                    valid_type = ty.clone();
                } else if ty.is_table() {
                    valid_type = ty.clone();
                }
            }
        }

        Some(valid_type)
    }

    pub fn get_global_decl_id(&self, key: &LuaMemberKey) -> Option<LuaDeclId> {
        let decls = self.global_decl.get(key)?;
        decls.first().cloned()
    }
}

impl LuaIndex for LuaDeclIndex {
    fn remove(&mut self, file_id: FileId) {
        self.decl_trees.remove(&file_id);
    }
}
