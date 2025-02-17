use emmylua_parser::{LuaAstNode, LuaAstToken, LuaLocalName};

use crate::{EmmyLuaAnalysis, FileId, LuaType, VirtualUrlGenerator};

/// A virtual workspace for testing.
#[allow(unused)]
#[derive(Debug)]
pub struct VirtualWorkspace {
    pub virtual_url_generator: VirtualUrlGenerator,
    pub analysis: EmmyLuaAnalysis,
    id_counter: u32,
}

#[allow(unused)]
impl VirtualWorkspace {
    pub fn new() -> Self {
        let mut analysis = EmmyLuaAnalysis::new();
        VirtualWorkspace {
            virtual_url_generator: VirtualUrlGenerator::new(),
            analysis,
            id_counter: 0,
        }
    }

    pub fn new_with_init_std_lib() -> Self {
        let mut analysis = EmmyLuaAnalysis::new();
        analysis.init_std_lib(false);
        VirtualWorkspace {
            virtual_url_generator: VirtualUrlGenerator::new(),
            analysis,
            id_counter: 0,
        }
    }

    pub fn def(&mut self, content: &str) -> FileId {
        let id = self.id_counter;
        self.id_counter += 1;
        let uri = self
            .virtual_url_generator
            .new_uri(&format!("virtual_{}.lua", id));
        let file_id = self
            .analysis
            .update_file_by_uri(&uri, Some(content.to_string()))
            .unwrap();
        file_id
    }

    pub fn get_node<Ast: LuaAstNode>(&self, file_id: FileId) -> Ast {
        let tree = self
            .analysis
            .compilation
            .get_db()
            .get_vfs()
            .get_syntax_tree(&file_id)
            .unwrap();
        tree.get_chunk_node().descendants::<Ast>().next().unwrap()
    }

    pub fn ty(&mut self, type_repr: &str) -> LuaType {
        let virtual_content = format!("---@type {}\nlocal t", type_repr);
        let file_id = self.def(&virtual_content);
        let local_name = self.get_node::<LuaLocalName>(file_id);
        let semantic_model = self
            .analysis
            .compilation
            .get_semantic_model(file_id)
            .unwrap();
        let token = local_name.get_name_token().unwrap();
        let info = semantic_model
            .get_semantic_info(token.syntax().clone().into())
            .unwrap();
        info.typ
    }

    pub fn expr_ty(&mut self, expr: &str) -> LuaType {
        let virtual_content = format!("local t = {}", expr);
        let file_id = self.def(&virtual_content);
        let local_name = self.get_node::<LuaLocalName>(file_id);
        let semantic_model = self
            .analysis
            .compilation
            .get_semantic_model(file_id)
            .unwrap();
        let token = local_name.get_name_token().unwrap();
        let info = semantic_model
            .get_semantic_info(token.syntax().clone().into())
            .unwrap();
        info.typ
    }
}

#[cfg(test)]
mod tests {
    use crate::LuaType;

    use super::VirtualWorkspace;

    #[test]
    fn test_basic() {
        let mut ws = VirtualWorkspace::new();

        ws.def(
            r#"
        ---@class a
        "#,
        );

        let ty = ws.ty("a");
        match ty {
            LuaType::Ref(i) => {
                assert_eq!(i.get_name(), "a");
            }
            _ => assert!(false),
        }
    }
}
