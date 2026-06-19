use crate::{documents::Document, session::SessionHandle, utils::apply_document_changes};
use anyhow::Result;
use pgls_workspace::workspace::{
    ChangeFileParams, CloseFileParams, GetFileContentParams, OpenFileParams,
};
use tower_lsp::lsp_types;
use tracing::{error, field};

/// Handler for `textDocument/didOpen` LSP notification
#[tracing::instrument(level = "debug", skip(session), err)]
pub(crate) async fn did_open(
    session: &SessionHandle,
    params: lsp_types::DidOpenTextDocumentParams,
) -> Result<()> {
    let url = params.text_document.uri;
    let version = params.text_document.version;
    let content = params.text_document.text;

    let path = session.file_path(&url)?;
    let doc = Document::new(version, &content);

    session.workspace.open_file(OpenFileParams {
        path,
        version,
        content,
    })?;

    session.insert_document(url.clone(), doc);

    if let Err(err) = session.update_diagnostics(url).await {
        error!("Failed to update diagnostics: {}", err);
    }

    Ok(())
}

/// Handler for `textDocument/didChange` LSP notification
///
/// Document content is updated synchronously so other LSP features (hover,
/// completion) see the latest text immediately. The expensive diagnostic
/// analysis is scheduled via [`SessionHandle::schedule_diagnostics`], which
/// debounces rapid-fire changes so that only one analysis run fires at the
/// end of a burst rather than one per keystroke.
#[tracing::instrument(level = "debug", skip_all, fields(url = field::display(&params.text_document.uri), version = params.text_document.version), err)]
pub(crate) async fn did_change(
    session: &SessionHandle,
    params: lsp_types::DidChangeTextDocumentParams,
) -> Result<()> {
    let url = params.text_document.uri;
    let version = params.text_document.version;

    let pgls_path = session.file_path(&url)?;

    let old_text = session.workspace.get_file_content(GetFileContentParams {
        path: pgls_path.clone(),
    })?;
    tracing::trace!("old document: {:?}", old_text);
    tracing::trace!("content changes: {:?}", params.content_changes);

    let text = apply_document_changes(
        session.position_encoding(),
        old_text,
        params.content_changes,
    );

    tracing::trace!("new document: {:?}", text);

    session.insert_document(url.clone(), Document::new(version, &text));

    session.workspace.change_file(ChangeFileParams {
        path: pgls_path,
        version,
        content: text,
    })?;

    // Schedule debounced diagnostics instead of running immediately.
    // This prevents a keystroke burst from queuing one analysis per change.
    session.schedule_diagnostics(url);

    Ok(())
}

/// Handler for `textDocument/didClose` LSP notification
#[tracing::instrument(level = "debug", skip(session), err)]
pub(crate) async fn did_close(
    session: &SessionHandle,
    params: lsp_types::DidCloseTextDocumentParams,
) -> Result<()> {
    let url = params.text_document.uri;
    let pgls_path = session.file_path(&url)?;

    // Cancel any pending debounced diagnostic task before removing the document
    // so that no stale diagnostics are published after the file is closed.
    session.cancel_pending_diagnostics(&url);

    session
        .workspace
        .close_file(CloseFileParams { path: pgls_path })?;

    session.remove_document(&url);

    let diagnostics = vec![];
    let version = None;
    session
        .client
        .publish_diagnostics(url, diagnostics, version)
        .await;

    Ok(())
}
