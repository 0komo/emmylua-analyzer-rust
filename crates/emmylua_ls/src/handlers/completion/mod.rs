mod completion_context;
mod providers;
mod data;

use completion_context::CompletionContext;
use emmylua_parser::LuaAstNode;
use lsp_types::{CompletionItem, CompletionParams, CompletionResponse};
use providers::add_completions;
use rowan::TokenAtOffset;
use tokio_util::sync::CancellationToken;

use crate::context::ServerContextSnapshot;

pub async fn on_completion_handler(
    context: ServerContextSnapshot,
    params: CompletionParams,
    cancel_token: CancellationToken,
) -> Option<CompletionResponse> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    let analysis = context.analysis.read().await;
    let file_id = analysis.get_file_id(&uri)?;
    let semantic_model = analysis.compilation.get_semantic_model(file_id)?;
    let root = semantic_model.get_root();
    let position_offset = {
        let document = semantic_model.get_document();
        document.get_offset(position.line as usize, position.character as usize)?
    };

    let token = match root.syntax().token_at_offset(position_offset) {
        TokenAtOffset::Single(token) => token,
        TokenAtOffset::Between(left, _) => left,
        TokenAtOffset::None => {
            return None;
        }
    };

    let mut completion_context = CompletionContext::new(token, semantic_model, cancel_token);
    add_completions(&mut completion_context);
    Some(CompletionResponse::Array(
        completion_context.get_completion_items(),
    ))
}

pub async fn on_completion_resolve_handler(
    context: ServerContextSnapshot,
    params: CompletionItem,
    cancel_token: CancellationToken,
) -> CompletionItem {
    params
}
