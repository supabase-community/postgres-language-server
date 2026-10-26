use pgls_workspace::{
    WorkspaceError,
    features::semantic_tokens::{PullSemanticTokensParams, SemanticTokenKind},
};
use tower_lsp_server::ls_types::{
    self, SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensLegend,
};

use crate::{
    adapters::{from_lsp, to_lsp},
    diagnostics::LspError,
    session::Session,
};

/// The token types, in the order of their indices in the legend.
const TOKEN_TYPES: [SemanticTokenType; 7] = [
    SemanticTokenType::NAMESPACE,
    SemanticTokenType::CLASS,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::TYPE,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::ENUM_MEMBER,
];

/// The bit of the `defaultLibrary` modifier, the only one in the legend.
const DEFAULT_LIBRARY: u32 = 1;

pub(crate) fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: vec![SemanticTokenModifier::DEFAULT_LIBRARY],
    }
}

fn token_type(kind: SemanticTokenKind) -> u32 {
    match kind {
        SemanticTokenKind::Namespace => 0,
        SemanticTokenKind::Class => 1,
        SemanticTokenKind::Property => 2,
        SemanticTokenKind::Function => 3,
        SemanticTokenKind::Type => 4,
        SemanticTokenKind::Parameter => 5,
        SemanticTokenKind::EnumMember => 6,
    }
}

#[tracing::instrument(level = "debug", skip(session), err)]
pub(crate) fn semantic_tokens_full(
    session: &Session,
    params: ls_types::SemanticTokensParams,
) -> Result<Option<ls_types::SemanticTokensResult>, LspError> {
    Ok(semantic_tokens(session, &params.text_document.uri, None)?
        .map(ls_types::SemanticTokensResult::Tokens))
}

#[tracing::instrument(level = "debug", skip(session), err)]
pub(crate) fn semantic_tokens_range(
    session: &Session,
    params: ls_types::SemanticTokensRangeParams,
) -> Result<Option<ls_types::SemanticTokensRangeResult>, LspError> {
    Ok(
        semantic_tokens(session, &params.text_document.uri, Some(params.range))?
            .map(ls_types::SemanticTokensRangeResult::Tokens),
    )
}

/// The tokens of the document, or of the statements that overlap `range`, encoded relative to
/// each other as the protocol requires.
fn semantic_tokens(
    session: &Session,
    url: &ls_types::Uri,
    range: Option<ls_types::Range>,
) -> Result<Option<SemanticTokens>, LspError> {
    let path = session.file_path(url)?;
    let project_key = session.project_key(&path);
    let doc = session.document(url)?;
    let encoding = session.position_encoding();
    // A range that ends past the end of the document, as some clients send, gets the tokens
    // of the whole document.
    let range = range.and_then(|range| from_lsp::text_range(&doc.line_index, range, encoding).ok());

    let result = match session
        .workspace
        .pull_semantic_tokens(PullSemanticTokensParams {
            project_key,
            path,
            range,
        }) {
        Ok(result) => result,
        Err(e) => match e {
            WorkspaceError::DatabaseConnectionError(_) => {
                session.editor_features_without_database();
                return Ok(None);
            }
            _ => return Err(e.into()),
        },
    };

    let mut data = Vec::with_capacity(result.tokens.len());
    let mut previous = ls_types::Position::default();
    for token in result.tokens {
        // A token whose position can't be encoded is left out rather than failing the response.
        let (Ok(start), Ok(end)) = (
            to_lsp::position(&doc.line_index, token.range.start(), encoding),
            to_lsp::position(&doc.line_index, token.range.end(), encoding),
        ) else {
            continue;
        };
        // Clients only support tokens on a single line by default, and the workspace splits
        // tokens that span several lines.
        if start.line != end.line {
            continue;
        }
        let delta_line = start.line - previous.line;
        data.push(ls_types::SemanticToken {
            delta_line,
            delta_start: if delta_line == 0 {
                start.character - previous.character
            } else {
                start.character
            },
            length: end.character - start.character,
            token_type: token_type(token.kind),
            token_modifiers_bitset: if token.default_library {
                DEFAULT_LIBRARY
            } else {
                0
            },
        });
        previous = start;
    }

    Ok(Some(SemanticTokens {
        result_id: None,
        data,
    }))
}
