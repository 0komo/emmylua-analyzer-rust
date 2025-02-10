use std::collections::HashSet;

use emmylua_code_analysis::LuaType;
use emmylua_parser::{LuaAstNode, LuaNameExpr};

use crate::handlers::completion::{
    add_completions::add_decl_completion, completion_builder::CompletionBuilder,
};

pub fn add_completion(builder: &mut CompletionBuilder) -> Option<()> {
    if builder.is_cancelled() {
        return None;
    }

    let name_expr = LuaNameExpr::cast(builder.trigger_token.parent()?)?;

    let file_id = builder.semantic_model.get_file_id();
    let decl_tree = builder
        .semantic_model
        .get_db()
        .get_decl_index()
        .get_decl_tree(&file_id)?;

    let mut duplicated_name = HashSet::new();
    let local_env = decl_tree.get_env_decls(builder.trigger_token.text_range().start())?;
    for decl_id in local_env.iter() {
        let (name, mut typ) = {
            let decl = builder
                .semantic_model
                .get_db()
                .get_decl_index()
                .get_decl(&decl_id)?;
            (
                decl.get_name().to_string(),
                decl.get_type().cloned().unwrap_or(LuaType::Unknown),
            )
        };
        if duplicated_name.contains(&name) {
            continue;
        }

        if let Some(chain) = builder
            .semantic_model
            .get_db()
            .get_flow_index()
            .get_flow_chain(file_id, decl_id.clone())
        {
            for type_assert in chain.get_type_asserts(name_expr.get_position()) {
                typ = type_assert.simple_tighten_type(typ);
            }
        }

        duplicated_name.insert(name.clone());
        add_decl_completion(builder, decl_id.clone(), &name, &typ);
    }

    let global_env = builder
        .semantic_model
        .get_db()
        .get_decl_index()
        .get_global_decls();
    for decl_id in global_env.iter() {
        let (name, typ) = {
            let decl = builder
                .semantic_model
                .get_db()
                .get_decl_index()
                .get_decl(&decl_id)?;
            (
                decl.get_name().to_string(),
                decl.get_type().cloned().unwrap_or(LuaType::Unknown),
            )
        };
        if duplicated_name.contains(&name) {
            continue;
        }

        duplicated_name.insert(name.clone());
        add_decl_completion(builder, decl_id.clone(), &name, &typ);
    }

    builder.env_duplicate_name.extend(duplicated_name);

    Some(())
}
