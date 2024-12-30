use code_analysis::{DbIndex, LuaCompilation, LuaPropertyOwnerId, LuaType};
use lsp_types::{OneOf, SymbolKind, SymbolTag, WorkspaceSymbol, WorkspaceSymbolResponse};
use tokio_util::sync::CancellationToken;

pub fn build_workspace_symbols(
    compilation: &LuaCompilation,
    query: String,
    cancel_token: CancellationToken,
) -> Option<WorkspaceSymbolResponse> {
    let mut symbols = Vec::new();
    add_global_variable_symbols(&mut symbols, compilation, &query, &cancel_token)?;
    add_type_symbols(&mut symbols, compilation, &query, &cancel_token)?;
    Some(WorkspaceSymbolResponse::Nested(symbols))
}

fn add_global_variable_symbols(
    symbols: &mut Vec<WorkspaceSymbol>,
    compilation: &LuaCompilation,
    query: &str,
    cancel_token: &CancellationToken,
) -> Option<()> {
    if cancel_token.is_cancelled() {
        return None;
    }

    let db = compilation.get_db();
    let decl_index = db.get_decl_index();
    let global_decl_id = decl_index.get_global_decls();
    for decl_id in global_decl_id {
        let decl = decl_index.get_decl(&decl_id)?;
        if cancel_token.is_cancelled() {
            return None;
        }

        if decl.get_name().contains(query) {
            let typ = decl.get_type().unwrap_or(&LuaType::Unknown);
            let property_owner_id = LuaPropertyOwnerId::LuaDecl(decl_id);
            let document = db.get_vfs().get_document(&decl.get_file_id())?;
            let location = document.to_lsp_location(decl.get_range())?;
            let symbol = WorkspaceSymbol {
                name: decl.get_name().to_string(),
                kind: get_symbol_kind(typ),
                tags: if is_deprecated(db, property_owner_id) {
                    Some(vec![SymbolTag::DEPRECATED])
                } else {
                    None
                },
                container_name: None,
                location: OneOf::Left(location),
                data: None,
            };
            symbols.push(symbol);
        }
    }

    Some(())
}

fn add_type_symbols(
    symbols: &mut Vec<WorkspaceSymbol>,
    compilation: &LuaCompilation,
    query: &str,
    cancel_token: &CancellationToken,
) -> Option<()> {
    if cancel_token.is_cancelled() {
        return None;
    }

    let db = compilation.get_db();
    let decl_index = db.get_type_index();
    let types = decl_index.get_all_types();
    for typ in types {
        if cancel_token.is_cancelled() {
            return None;
        }

        if typ.get_full_name().contains(query) {
            let property_owner_id = LuaPropertyOwnerId::TypeDecl(typ.get_id());
            let location = typ.get_locations().first()?;
            let document = db.get_vfs().get_document(&location.file_id)?;
            let location = document.to_lsp_location(location.range)?;
            let symbol = WorkspaceSymbol {
                name: typ.get_full_name().to_string(),
                kind: SymbolKind::CLASS,
                tags: if is_deprecated(db, property_owner_id) {
                    Some(vec![SymbolTag::DEPRECATED])
                } else {
                    None
                },
                container_name: None,
                location: OneOf::Left(location),
                data: None,
            };
            symbols.push(symbol);
        }
    }

    Some(())
}

fn get_symbol_kind(typ: &LuaType) -> SymbolKind {
    if typ.is_function() {
        return SymbolKind::FUNCTION;
    } else if typ.is_const() {
        return SymbolKind::CONSTANT;
    } else if typ.is_def() {
        return SymbolKind::CLASS;
    }

    SymbolKind::VARIABLE
}

fn is_deprecated(db: &DbIndex, id: LuaPropertyOwnerId) -> bool {
    let property = db.get_property_index().get_property(id);
    if property.is_none() {
        return false;
    }

    property.unwrap().is_deprecated
}
