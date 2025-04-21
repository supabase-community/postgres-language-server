use crate::adapters::{PositionEncoding, WideEncoding, negotiated_encoding};
use pgt_workspace::features::code_actions::CommandActionCategory;
use strum::IntoEnumIterator;
use tower_lsp::lsp_types::{
    ClientCapabilities, CompletionOptions, ExecuteCommandOptions, PositionEncodingKind,
    SaveOptions, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, WorkDoneProgressOptions,
};

use crate::handlers::code_actions::command_id;

/// The capabilities to send from server as part of [`InitializeResult`]
///
/// [`InitializeResult`]: lspower::lsp::InitializeResult
pub(crate) fn server_capabilities(capabilities: &ClientCapabilities) -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(match negotiated_encoding(capabilities) {
            PositionEncoding::Utf8 => PositionEncodingKind::UTF8,
            PositionEncoding::Wide(wide) => match wide {
                WideEncoding::Utf16 => PositionEncodingKind::UTF16,
                WideEncoding::Utf32 => PositionEncodingKind::UTF32,
            },
        }),
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::INCREMENTAL),
                will_save: None,
                will_save_wait_until: None,
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                    include_text: Some(false),
                })),
            },
        )),
        completion_provider: Some(CompletionOptions {
            // currently not supporting the completionItem/resolve request.
            // The request is used to get more information about a simple CompletionItem.
            resolve_provider: None,

            trigger_characters: Some(vec![".".to_owned(), ",".to_owned(), " ".to_owned()]),

            // No character will lead to automatically inserting the selected completion-item
            all_commit_characters: None,

            // No special support for completionItem/resolve requests
            completion_item: None,

            // We do not report the progress of the completion process
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        }),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: CommandActionCategory::iter()
                .map(|c| command_id(&c))
                .collect::<Vec<String>>(),

            ..Default::default()
        }),
        document_formatting_provider: None,
        document_range_formatting_provider: None,
        document_on_type_formatting_provider: None,
        code_action_provider: Some(tower_lsp::lsp_types::CodeActionProviderCapability::Simple(
            true,
        )),
        rename_provider: None,
        ..Default::default()
    }
}
