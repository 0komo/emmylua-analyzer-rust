use emmylua_parser::{
    LuaAstToken, LuaAstTokenChildren, LuaDocTagAlias, LuaDocTagClass, LuaDocTagEnum, LuaDocTagMeta, LuaDocTagModule, LuaDocTagNamespace, LuaDocTagUsing, LuaNameToken
};
use flagset::FlagSet;

use crate::db_index::{LuaDeclTypeKind, LuaTypeAttribute};

use super::DeclAnalyzer;

pub fn analyze_doc_tag_class(analyzer: &mut DeclAnalyzer, class: LuaDocTagClass) {
    let (name, range) = if let Some(name_token) = class.get_name_token() {
        (
            name_token.get_name_text().to_string(),
            name_token.syntax().text_range(),
        )
    } else {
        return;
    };

    let attrib = if let Some(attrib) = class.get_attrib() {
        get_attrib_value(attrib.get_attrib_tokens())
    } else {
        None
    };

    let file_id = analyzer.get_file_id();
    analyzer.db.get_type_index().add_type_decl(file_id, range, name, LuaDeclTypeKind::Class, attrib);
}

fn get_attrib_value(
    attrib: LuaAstTokenChildren<LuaNameToken>,
) -> Option<FlagSet<LuaTypeAttribute>> {
    let mut attr: FlagSet<LuaTypeAttribute> = LuaTypeAttribute::None.into();

    for token in attrib {
        match token.get_name_text() {
            "partial" => {
                attr |= LuaTypeAttribute::Partial;
            }
            "key" => {
                attr |= LuaTypeAttribute::Key;
            }
            "global" => {
                attr |= LuaTypeAttribute::Global;
            }
            "exact" => {
                attr |= LuaTypeAttribute::Exact;
            }
            _ => {}
        }
    }
    Some(attr)
}

pub fn analyze_doc_tag_enum(analyzer: &mut DeclAnalyzer, enum_: LuaDocTagEnum) {
    let (name, range) = if let Some(name_token) = enum_.get_name_token() {
        (
            name_token.get_name_text().to_string(),
            name_token.syntax().text_range(),
        )
    } else {
        return;
    };

    let attrib = if let Some(attrib) = enum_.get_attrib() {
        get_attrib_value(attrib.get_attrib_tokens())
    } else {
        None
    };

    let file_id = analyzer.get_file_id();
    analyzer.db.get_type_index().add_type_decl(file_id, range, name, LuaDeclTypeKind::Enum, attrib);
}

pub fn analyze_doc_tag_alias(analyzer: &mut DeclAnalyzer, alias: LuaDocTagAlias) {
    let (name, range) = if let Some(name_token) = alias.get_name_token() {
        (
            name_token.get_name_text().to_string(),
            name_token.syntax().text_range(),
        )
    } else {
        return;
    };

    let file_id = analyzer.get_file_id();
    analyzer.db.get_type_index().add_type_decl(file_id, range, name, LuaDeclTypeKind::Alias, None);
}

pub fn analyze_doc_tag_namespace(analyzer: &mut DeclAnalyzer, namespace: LuaDocTagNamespace) {
    let name = if let Some(name_token) = namespace.get_name_token() {
        name_token.get_name_text().to_string()
    } else {
        return;
    };

    let file_id = analyzer.get_file_id();
    analyzer
        .db
        .get_type_index()
        .add_file_namespace(file_id, name);
}

pub fn analyze_doc_tag_using(analyzer: &mut DeclAnalyzer, using: LuaDocTagUsing) {
    let name = if let Some(name_token) = using.get_name_token() {
        name_token.get_name_text().to_string()
    } else {
        return;
    };

    let file_id = analyzer.get_file_id();
    analyzer
        .db
        .get_type_index()
        .add_file_using_namespace(file_id, name)
}

pub fn analyze_doc_tag_module(analyzer: &mut DeclAnalyzer, module: LuaDocTagModule) {
    let file_id = analyzer.get_file_id();
    if let Some(name_token) = module.get_name_token() {
        if name_token.get_name_text() == "no-require" {
            analyzer.db.get_module_index().set_module_visibility(file_id, false);
        }
        return;
    }

    let force_module_path = if let Some(string_token) = module.get_string_token() {
        string_token.get_value()
    } else {
        return;
    };

    analyzer
        .db
        .get_module_index()
        .add_module_by_module_path(file_id, force_module_path);
}

pub fn analyze_doc_tag_meta(analyzer: &mut DeclAnalyzer, _: LuaDocTagMeta) {
    let file_id = analyzer.get_file_id();
    analyzer
        .db
        .get_meta_file()
        .add_meta_file(file_id);
}