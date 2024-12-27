mod highlight_tokens;

use emmylua_parser::{LuaAstNode, LuaTokenKind};
use lsp_types::{DocumentHighlight, DocumentHighlightParams};
use rowan::TokenAtOffset;
use highlight_tokens::highlight_tokens;
use tokio_util::sync::CancellationToken;

use crate::context::ServerContextSnapshot;

pub async fn on_document_highlight_handler(
    context: ServerContextSnapshot,
    params: DocumentHighlightParams,
    _: CancellationToken,
) -> Option<Vec<DocumentHighlight>> {
    let uri = params.text_document_position_params.text_document.uri;
    let analysis = context.analysis.read().await;
    let file_id = analysis.get_file_id(&uri)?;
    let position = params.text_document_position_params.position;
    let mut semantic_model = analysis.compilation.get_semantic_model(file_id)?;
    let root = semantic_model.get_root();
    let position_offset = {
        let document = semantic_model.get_document();
        document.get_offset(position.line as usize, position.character as usize)?
    };

    let token = match root.syntax().token_at_offset(position_offset) {
        TokenAtOffset::Single(token) => token,
        TokenAtOffset::Between(left, right) => {
            if left.kind() == LuaTokenKind::TkName.into() {
                left
            } else {
                right
            }
        }
        TokenAtOffset::None => {
            return None;
        }
    };

    highlight_tokens(&mut semantic_model, token)
}
