use crate::{adapters, diagnostics::LspError, session::Session};
use pgls_workspace::features::semantic_tokens::SemanticTokensParams;
use tower_lsp::lsp_types::{self, SemanticToken, SemanticTokens, SemanticTokensRangeResult};

/// Handles a full semantic tokens request.
/// Returns all semantic tokens for the entire document.
#[tracing::instrument(level = "debug", skip(session), err)]
pub fn semantic_tokens_full(
    session: &Session,
    params: lsp_types::SemanticTokensParams,
) -> Result<Option<lsp_types::SemanticTokensResult>, LspError> {
    let url = &params.text_document.uri;
    let path = session.file_path(url)?;
    let doc = session.document(url)?;
    let encoding = adapters::negotiated_encoding(session.client_capabilities().unwrap());

    let workspace_result = session
        .workspace
        .get_semantic_tokens(SemanticTokensParams { path, range: None })?;

    let lsp_tokens = encode_tokens(&workspace_result.tokens, &doc.line_index, encoding)?;

    Ok(Some(lsp_types::SemanticTokensResult::Tokens(
        SemanticTokens {
            result_id: None,
            data: lsp_tokens,
        },
    )))
}

/// Handles a range semantic tokens request.
/// Returns semantic tokens for the specified range only.
#[tracing::instrument(level = "debug", skip(session), err)]
pub fn semantic_tokens_range(
    session: &Session,
    params: lsp_types::SemanticTokensRangeParams,
) -> Result<Option<SemanticTokensRangeResult>, LspError> {
    let url = &params.text_document.uri;
    let path = session.file_path(url)?;
    let doc = session.document(url)?;
    let encoding = adapters::negotiated_encoding(session.client_capabilities().unwrap());

    // Convert LSP range to TextRange
    let start_offset =
        adapters::from_lsp::offset(&doc.line_index, params.range.start, encoding)?;
    let end_offset = adapters::from_lsp::offset(&doc.line_index, params.range.end, encoding)?;
    let range = pgls_text_size::TextRange::new(start_offset, end_offset);

    let workspace_result = session
        .workspace
        .get_semantic_tokens(SemanticTokensParams {
            path,
            range: Some(range),
        })?;

    let lsp_tokens = encode_tokens(&workspace_result.tokens, &doc.line_index, encoding)?;

    Ok(Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
        result_id: None,
        data: lsp_tokens,
    })))
}

/// Encodes workspace semantic tokens into the LSP delta-encoded format.
///
/// LSP semantic tokens are encoded as a flat array of integers with 5 values per token:
/// - deltaLine: line difference from previous token
/// - deltaStart: character offset from start of line (or from previous token if same line)
/// - length: the length of the token in characters
/// - tokenType: the token type index
/// - tokenModifiers: bit flags for token modifiers
///
/// Multi-line tokens (like block comments) are split into multiple LSP tokens,
/// one per line, since LSP semantic tokens cannot span lines.
fn encode_tokens(
    tokens: &[pgls_workspace::features::semantic_tokens::SemanticToken],
    line_index: &adapters::line_index::LineIndex,
    encoding: adapters::PositionEncoding,
) -> Result<Vec<SemanticToken>, LspError> {
    let mut result = Vec::with_capacity(tokens.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for token in tokens {
        // Convert token range to LSP positions
        let start_pos = adapters::to_lsp::position(line_index, token.range.start(), encoding)
            .map_err(|e| LspError::from(anyhow::anyhow!("Failed to convert position: {}", e)))?;

        let end_pos = adapters::to_lsp::position(line_index, token.range.end(), encoding)
            .map_err(|e| LspError::from(anyhow::anyhow!("Failed to convert position: {}", e)))?;

        if start_pos.line == end_pos.line {
            // Single-line token - emit one LSP token
            let length = end_pos.character - start_pos.character;
            let delta_line = start_pos.line - prev_line;
            let delta_start = if delta_line == 0 {
                start_pos.character - prev_start
            } else {
                start_pos.character
            };

            result.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type: token.token_type,
                token_modifiers_bitset: token.token_modifiers,
            });

            prev_line = start_pos.line;
            prev_start = start_pos.character;
        } else {
            // Multi-line token - emit one LSP token per line
            for line in start_pos.line..=end_pos.line {
                let (line_start, line_length) = if line == start_pos.line {
                    // First line: from token start to end of line
                    let line_len = get_line_length(line_index, line, encoding);
                    (start_pos.character, line_len.saturating_sub(start_pos.character))
                } else if line == end_pos.line {
                    // Last line: from start of line to token end
                    (0, end_pos.character)
                } else {
                    // Middle lines: entire line
                    (0, get_line_length(line_index, line, encoding))
                };

                // Skip empty segments
                if line_length == 0 {
                    continue;
                }

                let delta_line = line - prev_line;
                let delta_start = if delta_line == 0 {
                    line_start - prev_start
                } else {
                    line_start
                };

                result.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: line_length,
                    token_type: token.token_type,
                    token_modifiers_bitset: token.token_modifiers,
                });

                prev_line = line;
                prev_start = line_start;
            }
        }
    }

    Ok(result)
}

/// Gets the length of a line in the appropriate encoding (excluding the newline character).
fn get_line_length(
    line_index: &adapters::line_index::LineIndex,
    line: u32,
    encoding: adapters::PositionEncoding,
) -> u32 {
    let line_usize = line as usize;

    // Get the start offset of this line and the next line
    let line_start = line_index.newlines.get(line_usize).copied();
    let next_line_start = line_index.newlines.get(line_usize + 1).copied();

    let (Some(start), Some(end)) = (line_start, next_line_start) else {
        // Last line or invalid line - estimate from offset
        // For the last line, we don't have a next newline offset
        // Return 0 as a safe fallback (the token end position should handle this)
        return 0;
    };

    // Line length in bytes (excluding newline)
    let byte_length = end - start - pgls_text_size::TextSize::from(1u32);

    // Convert to the appropriate encoding
    match encoding {
        adapters::PositionEncoding::Utf8 => byte_length.into(),
        adapters::PositionEncoding::Wide(enc) => {
            let line_col = adapters::LineCol {
                line,
                col: byte_length.into(),
            };
            line_index
                .to_wide(enc, line_col)
                .map(|wlc| wlc.col)
                .unwrap_or(byte_length.into())
        }
    }
}
