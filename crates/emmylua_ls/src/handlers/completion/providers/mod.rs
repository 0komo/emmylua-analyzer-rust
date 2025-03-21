mod auto_require_provider;
mod doc_name_token_provider;
mod doc_tag_provider;
mod doc_type_provider;
mod env_provider;
mod file_path_provider;
mod keywords_provider;
mod member_provider;
mod module_path_provider;
mod postfix_provider;
mod table_field_provider;
mod type_special_provider;

use emmylua_parser::{LuaAst, LuaAstNode, LuaAstToken, LuaStringToken};
use lsp_types::CompletionTriggerKind;
use rowan::TextRange;

use super::completion_builder::CompletionBuilder;

pub fn add_completions(builder: &mut CompletionBuilder) -> Option<()> {
    // 空格补全只允许位于函数调用参数列表时非主动触发
    if is_space_completion(builder).is_some() {
        match LuaAst::cast(builder.trigger_token.parent()?)? {
            LuaAst::LuaCallArgList(_) => {
                type_special_provider::add_completion(builder);
                return Some(());
            }
            _ => return Some(()),
        }
    }

    postfix_provider::add_completion(builder);
    type_special_provider::add_completion(builder);
    // `env_provider` 在某些情况下是不需要的, 但有些补全功能依赖于他, 因在一些补全中可能会移除掉`env_provider` 的补全.
    // 目前可能移除掉他的补全为: `table_field_provider`
    env_provider::add_completion(builder);
    // 只有具有类型定义的表才会成功返回, 此时我们不需要处理其他补全
    if table_field_provider::add_completion(builder).is_some() {
        return Some(());
    }
    member_provider::add_completion(builder);
    keywords_provider::add_completion(builder);

    module_path_provider::add_completion(builder);
    file_path_provider::add_completion(builder);
    auto_require_provider::add_completion(builder);
    doc_tag_provider::add_completion(builder);
    doc_type_provider::add_completion(builder);
    doc_name_token_provider::add_completion(builder);

    for (index, item) in builder.get_completion_items_mut().iter_mut().enumerate() {
        if item.sort_text.is_none() {
            item.sort_text = Some(format!("{:04}", index + 1));
        }
    }
    // dbg!(&builder.get_completion_items_mut());

    Some(())
}

fn is_space_completion(builder: &CompletionBuilder) -> Option<()> {
    if builder.get_trigger_text().is_empty()
        && builder.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER
    {
        Some(())
    } else {
        None
    }
}

fn get_text_edit_range_in_string(
    builder: &mut CompletionBuilder,
    string_token: LuaStringToken,
) -> Option<lsp_types::Range> {
    let text = string_token.get_text();
    let range = string_token.get_range();
    if text.len() == 0 {
        return None;
    }

    let mut start_offset = u32::from(range.start());
    let mut end_offset = u32::from(range.end());
    if text.starts_with('"') || text.starts_with('\'') {
        start_offset += 1;
    }

    if text.ends_with('"') || text.ends_with('\'') {
        end_offset -= 1;
    }

    let new_text_range = TextRange::new(start_offset.into(), end_offset.into());
    let lsp_range = builder
        .semantic_model
        .get_document()
        .to_lsp_range(new_text_range);

    lsp_range
}
