use std::collections::HashMap;

use code_analysis::{
    DbIndex, FileId, LuaDecl, LuaDeclId, LuaDeclarationTree, LuaDocument, LuaType,
};
use emmylua_parser::{LuaAstNode, LuaChunk, LuaSyntaxId, LuaSyntaxNode, LuaSyntaxToken};
use lsp_types::{DocumentSymbol, SymbolKind};
use rowan::TextRange;

pub struct DocumentSymbolBuilder<'a> {
    db: &'a DbIndex,
    decl_tree: &'a LuaDeclarationTree,
    document: &'a LuaDocument<'a>,
    document_symbols: HashMap<LuaSyntaxId, Box<LuaSymbol>>,
}

impl<'a> DocumentSymbolBuilder<'a> {
    pub fn new(
        db: &'a DbIndex,
        decl_tree: &'a LuaDeclarationTree,
        document: &'a LuaDocument,
    ) -> Self {
        Self {
            db,
            decl_tree,
            document,
            document_symbols: HashMap::new(),
        }
    }

    pub fn get_file_id(&self) -> FileId {
        self.document.get_file_id()
    }

    pub fn get_decl(&self, id: &LuaDeclId) -> Option<&LuaDecl> {
        self.decl_tree.get_decl(id)
    }

    pub fn add_node_symbol(&mut self, node: LuaSyntaxNode, symbol: LuaSymbol) {
        let syntax_id = LuaSyntaxId::new(node.kind().into(), node.text_range());
        self.document_symbols.insert(syntax_id, Box::new(symbol));
        let mut node = node;
        while let Some(parent) = node.parent() {
            let parent_syntax_id = LuaSyntaxId::new(parent.kind().into(), parent.text_range());
            if let Some(symbol) = self.document_symbols.get_mut(&parent_syntax_id) {
                symbol.add_child(syntax_id);
                break;
            }

            node = parent;
        }
    }

    pub fn add_token_symbol(&mut self, token: LuaSyntaxToken, symbol: LuaSymbol) {
        let syntax_id = LuaSyntaxId::new(token.kind().into(), token.text_range());
        self.document_symbols.insert(syntax_id, Box::new(symbol));

        let mut node = token.parent();
        while let Some(parent_node) = node {
            let parent_syntax_id =
                LuaSyntaxId::new(parent_node.kind().into(), parent_node.text_range());
            if let Some(symbol) = self.document_symbols.get_mut(&parent_syntax_id) {
                symbol.add_child(syntax_id);
                break;
            }

            node = parent_node.parent();
        }
    }

    #[allow(deprecated)]
    pub fn build(self, root: &LuaChunk) -> DocumentSymbol {
        let id = root.get_syntax_id();
        let lua_symbol = self.document_symbols.get(&id).unwrap();
        let lsp_range = self.document.to_lsp_range(lua_symbol.range).unwrap();
        let mut document_symbol = DocumentSymbol {
            name: lua_symbol.name.clone(),
            detail: lua_symbol.detail.clone(),
            kind: lua_symbol.kind,
            range: lsp_range.clone(),
            selection_range: lsp_range,
            children: None,
            tags: None,
            deprecated: None,
        };

        self.build_child_symbol(&mut document_symbol, lua_symbol);

        document_symbol
    }

    #[allow(deprecated)]
    fn build_child_symbol(
        &self,
        document_symbol: &mut DocumentSymbol,
        symbol: &LuaSymbol,
    ) -> Option<()> {
        for child in &symbol.children {
            let child_symbol = self.document_symbols.get(child)?;
            let lsp_range = self.document.to_lsp_range(child_symbol.range)?;
            let mut lsp_document_symbol = DocumentSymbol {
                name: child_symbol.name.clone(),
                detail: child_symbol.detail.clone(),
                kind: child_symbol.kind,
                range: lsp_range.clone(),
                selection_range: lsp_range,
                children: None,
                tags: None,
                deprecated: None,
            };

            self.build_child_symbol(&mut lsp_document_symbol, child_symbol);
            document_symbol
                .children
                .get_or_insert_with(Vec::new)
                .push(lsp_document_symbol);
        }

        Some(())
    }

    pub fn get_symbol_kind_and_detail(&self, ty: Option<&LuaType>) -> (SymbolKind, Option<String>) {
        if ty.is_none() {
            return (SymbolKind::VARIABLE, None);
        }

        let ty = ty.unwrap();

        if ty.is_def() {
            return (SymbolKind::CLASS, None);
        } else if ty.is_string() {
            return (SymbolKind::STRING, None);
        } else if ty.is_table() {
            return (SymbolKind::OBJECT, None);
        } else if ty.is_number() {
            return match ty {
                LuaType::IntegerConst(i) => (SymbolKind::NUMBER, Some(i.to_string())),
                LuaType::FloatConst(f) => (SymbolKind::NUMBER, Some(f.to_string())),
                _ => (SymbolKind::NUMBER, None),
            };
        } else if ty.is_function() {
            return match ty {
                LuaType::DocFunction(f) => {
                    let params = f.get_params();
                    let mut param_names = Vec::new();
                    for param in params {
                        param_names.push(param.0.to_string());
                    }

                    let detail = format!("({})", param_names.join(", "));
                    (SymbolKind::FUNCTION, Some(detail))
                }
                LuaType::Signature(s) => {
                    let signature = self.db.get_signature_index().get(s);
                    if let Some(signature) = signature {
                        let params = signature.get_type_params();
                        let mut param_names = Vec::new();
                        for param in params {
                            param_names.push(param.0.to_string());
                        }

                        let detail = format!("({})", param_names.join(", "));
                        return (SymbolKind::FUNCTION, Some(detail));
                    } else {
                        return (SymbolKind::FUNCTION, None);
                    }
                }
                _ => (SymbolKind::FUNCTION, None),
            };
        } else if ty.is_boolean() {
            return (SymbolKind::BOOLEAN, None);
        }

        (SymbolKind::VARIABLE, None)
    }
}

#[derive(Debug)]
pub struct LuaSymbol {
    name: String,
    detail: Option<String>,
    kind: SymbolKind,
    range: TextRange,
    children: Vec<LuaSyntaxId>,
}

impl LuaSymbol {
    pub fn new(name: String, detail: Option<String>, kind: SymbolKind, range: TextRange) -> Self {
        Self {
            name,
            detail,
            kind,
            range,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: LuaSyntaxId) {
        self.children.push(child);
    }
}
