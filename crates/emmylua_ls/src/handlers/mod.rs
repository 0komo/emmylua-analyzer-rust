mod code_lens;
mod command;
mod completion;
mod defination;
mod document_color;
mod document_highlight;
mod document_link;
mod document_selection_range;
mod document_symbol;
mod emmy_annotator;
mod fold_range;
mod hover;
mod initialized;
mod inlay_hint;
mod notification_handler;
mod references;
mod rename;
mod request_handler;
mod response_handler;
mod semantic_token;
mod signature_helper;
mod text_document;

use command::COMMANDS;
pub use initialized::initialized_handler;
pub use initialized::{init_analysis, ClientConfig};
use lsp_types::{
    CodeLensOptions, ColorProviderCapability, CompletionOptions, CompletionOptionsCompletionItem,
    DocumentLinkOptions, DocumentSymbolOptions, ExecuteCommandOptions,
    FoldingRangeProviderCapability, HoverProviderCapability, InlayHintOptions,
    InlayHintServerCapabilities, OneOf, RenameOptions, SaveOptions,
    SelectionRangeProviderCapability, SemanticTokensFullOptions, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities,
    SignatureHelpOptions, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncSaveOptions,
};
pub use notification_handler::on_notification_handler;
pub use request_handler::on_req_handler;
pub use response_handler::on_response_handler;
use semantic_token::{SEMANTIC_TOKEN_MODIFIERS, SEMANTIC_TOKEN_TYPES};

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            lsp_types::TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                will_save: None,
                will_save_wait_until: None,
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                    include_text: Some(false),
                })),
            },
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        document_symbol_provider: Some(OneOf::Right(DocumentSymbolOptions {
            label: Some("EmmyLua".into()),
            work_done_progress_options: Default::default(),
        })),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        color_provider: Some(ColorProviderCapability::Simple(true)),
        document_link_provider: Some(DocumentLinkOptions {
            resolve_provider: Some(false),
            work_done_progress_options: Default::default(),
        }),
        selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: Some(
                vec![".", ":", "(", "[", "\"", "\'", ",", "@", "\\", "/"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            work_done_progress_options: Default::default(),
            completion_item: Some(CompletionOptionsCompletionItem {
                label_details_support: Some(true),
            }),
            all_commit_characters: Default::default(),
        }),
        inlay_hint_provider: Some(OneOf::Right(InlayHintServerCapabilities::Options(
            InlayHintOptions {
                resolve_provider: Some(false),
                work_done_progress_options: Default::default(),
            },
        ))),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: Default::default(),
        })),
        code_lens_provider: Some(CodeLensOptions {
            resolve_provider: Some(true),
        }),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(", ","].iter().map(|s| s.to_string()).collect()),
            retrigger_characters: Some(vec!["(", ","].iter().map(|s| s.to_string()).collect()),
            ..Default::default()
        }),
        document_highlight_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_modifiers: SEMANTIC_TOKEN_MODIFIERS.iter().cloned().collect(),
                    token_types: SEMANTIC_TOKEN_TYPES.iter().cloned().collect(),
                },
                full: Some(SemanticTokensFullOptions::Bool(true)),
                ..Default::default()
            },
        )),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: COMMANDS.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }),
        ..Default::default()
    }
}
