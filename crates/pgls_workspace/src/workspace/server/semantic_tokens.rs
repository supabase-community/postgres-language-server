//! Semantic token computation using pgls_query::scan()
//!
//! This module provides semantic token types and modifiers for SQL syntax highlighting.
//! The LSP layer converts these to protocol-specific types.

use std::num::NonZeroUsize;
use std::sync::Mutex;

use lru::LruCache;
use pgls_query::protobuf::{KeywordKind, Token};
use pgls_text_size::{TextRange, TextSize};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

use super::statement_identifier::StatementId;

const DEFAULT_CACHE_SIZE: usize = 1000;

/// Semantic token types for SQL syntax highlighting.
/// The discriminant values define the indices sent to the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "camelCase")]
#[repr(u32)]
pub enum TokenType {
    Keyword = 0,
    Type = 1,
    Function = 2,
    Parameter = 3,
    String = 4,
    Number = 5,
    Operator = 6,
    Comment = 7,
    Property = 8,
}

impl TokenType {
    /// Returns the legend as a list of token type names in order.
    pub fn legend() -> Vec<&'static str> {
        Self::iter().map(|t| t.into()).collect()
    }
}

/// Semantic token modifier bit flags.
/// Modifiers can be combined using bitwise OR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "camelCase")]
#[repr(u32)]
pub enum TokenModifier {
    Declaration = 0,
    Definition = 1,
    Readonly = 2,
    DefaultLibrary = 3,
}

impl TokenModifier {
    /// Returns the bit flag value for this modifier.
    pub const fn bit(self) -> u32 {
        1 << (self as u32)
    }

    /// Returns the legend as a list of modifier names in order.
    pub fn legend() -> Vec<&'static str> {
        Self::iter().map(|m| m.into()).collect()
    }
}

/// No modifiers applied.
pub const NO_MODIFIERS: u32 = 0;

/// A semantic token with absolute position
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SemanticToken {
    /// The text range of this token
    pub range: TextRange,
    /// The semantic token type index
    pub token_type: u32,
    /// The semantic token modifiers as a bit flag
    pub token_modifiers: u32,
}

impl SemanticToken {
    pub fn new(start: TextSize, end: TextSize, token_type: TokenType, token_modifiers: u32) -> Self {
        Self {
            range: TextRange::new(start, end),
            token_type: token_type as u32,
            token_modifiers,
        }
    }
}

/// Cache for semantic tokens per statement
pub struct SemanticTokenStore {
    db: Mutex<LruCache<StatementId, Vec<SemanticToken>>>,
}

impl Default for SemanticTokenStore {
    fn default() -> Self {
        Self {
            db: Mutex::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap(),
            )),
        }
    }
}

impl SemanticTokenStore {
    /// Get cached tokens or compute and cache them for the given statement.
    /// Returns tokens with positions relative to the statement start.
    pub fn get_or_cache_tokens(&self, statement: &StatementId) -> Vec<SemanticToken> {
        let mut cache = self.db.lock().unwrap();

        if let Some(existing) = cache.get(statement) {
            return existing.clone();
        }

        let tokens = compute_tokens(statement.content());
        cache.put(statement.clone(), tokens.clone());
        tokens
    }
}

/// Maps a pgls_query token to semantic token type and modifiers
fn map_token(token: Token, keyword_kind: KeywordKind) -> Option<(TokenType, u32)> {
    match token {
        // Comments
        Token::SqlComment | Token::CComment => Some((TokenType::Comment, NO_MODIFIERS)),

        // String literals
        Token::Sconst | Token::Usconst => Some((TokenType::String, TokenModifier::Readonly.bit())),

        // Numeric literals
        Token::Iconst | Token::Fconst | Token::Bconst | Token::Xconst => {
            Some((TokenType::Number, TokenModifier::Readonly.bit()))
        }

        // Parameters ($1, $2, etc.)
        Token::Param => Some((TokenType::Parameter, NO_MODIFIERS)),

        // Operators
        Token::Op
        | Token::Typecast
        | Token::DotDot
        | Token::ColonEquals
        | Token::EqualsGreater
        | Token::LessEquals
        | Token::GreaterEquals
        | Token::NotEquals => Some((TokenType::Operator, NO_MODIFIERS)),

        // Single-character operators
        Token::Ascii37  // %
        | Token::Ascii42  // *
        | Token::Ascii43  // +
        | Token::Ascii45  // -
        | Token::Ascii47  // /
        | Token::Ascii60  // <
        | Token::Ascii61  // =
        | Token::Ascii62  // >
        | Token::Ascii94  // ^
        => Some((TokenType::Operator, NO_MODIFIERS)),

        // Identifiers
        Token::Ident | Token::Uident => Some((TokenType::Property, NO_MODIFIERS)),

        // Type keywords - these are known SQL type names
        Token::Bigint
        | Token::Bit
        | Token::BooleanP
        | Token::CharP
        | Token::Character
        | Token::Dec
        | Token::DecimalP
        | Token::DoubleP
        | Token::FloatP
        | Token::IntP
        | Token::Integer
        | Token::Interval
        | Token::National
        | Token::Nchar
        | Token::None
        | Token::Numeric
        | Token::Real
        | Token::Setof
        | Token::Smallint
        | Token::Time
        | Token::Timestamp
        | Token::Varchar
        | Token::Varying
        | Token::Xmlattributes
        | Token::Xmlconcat
        | Token::Xmlelement
        | Token::Xmlexists
        | Token::Xmlforest
        | Token::Xmlnamespaces
        | Token::Xmlparse
        | Token::Xmlpi
        | Token::Xmlroot
        | Token::Xmlserialize
        | Token::Xmltable => Some((TokenType::Type, NO_MODIFIERS)),

        // Keywords (other tokens with keyword_kind)
        _ => match keyword_kind {
            KeywordKind::TypeFuncNameKeyword => Some((TokenType::Type, NO_MODIFIERS)),
            KeywordKind::ReservedKeyword => {
                Some((TokenType::Keyword, TokenModifier::DefaultLibrary.bit()))
            }
            KeywordKind::UnreservedKeyword | KeywordKind::ColNameKeyword => {
                Some((TokenType::Keyword, NO_MODIFIERS))
            }
            KeywordKind::NoKeyword => None,
        },
    }
}

/// Computes semantic tokens for the given SQL string
pub fn compute_tokens(sql: &str) -> Vec<SemanticToken> {
    let scan_result = match pgls_query::scan(sql) {
        Ok(result) => result,
        Err(_) => return Vec::new(),
    };

    let mut tokens = Vec::with_capacity(scan_result.tokens.len());

    for scan_token in scan_result.tokens {
        let token = Token::try_from(scan_token.token).unwrap_or(Token::Nul);
        let keyword_kind =
            KeywordKind::try_from(scan_token.keyword_kind).unwrap_or(KeywordKind::NoKeyword);

        if let Some((token_type, token_modifiers)) = map_token(token, keyword_kind) {
            let start = TextSize::from(scan_token.start as u32);
            let end = TextSize::from(scan_token.end as u32);

            tokens.push(SemanticToken::new(start, end, token_type, token_modifiers));
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let sql = "SELECT * FROM users";
        let tokens = compute_tokens(sql);

        assert!(!tokens.is_empty());

        // First token should be SELECT (keyword)
        assert_eq!(tokens[0].token_type, TokenType::Keyword as u32);

        // Check that we have an identifier for 'users'
        let users_token = tokens
            .iter()
            .find(|t| {
                let start: usize = t.range.start().into();
                let end: usize = t.range.end().into();
                &sql[start..end] == "users"
            })
            .expect("Should find users token");
        assert_eq!(users_token.token_type, TokenType::Property as u32);
    }

    #[test]
    fn test_string_literal() {
        let sql = "SELECT 'hello'";
        let tokens = compute_tokens(sql);

        let string_token = tokens
            .iter()
            .find(|t| t.token_type == TokenType::String as u32)
            .expect("Should find string token");

        assert_eq!(string_token.token_modifiers, TokenModifier::Readonly.bit());
    }

    #[test]
    fn test_numeric_literal() {
        let sql = "SELECT 42, 3.14";
        let tokens = compute_tokens(sql);

        let number_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Number as u32)
            .collect();

        assert_eq!(number_tokens.len(), 2);
    }

    #[test]
    fn test_comment() {
        let sql = "SELECT 1 -- this is a comment";
        let tokens = compute_tokens(sql);

        let comment_token = tokens
            .iter()
            .find(|t| t.token_type == TokenType::Comment as u32)
            .expect("Should find comment token");

        assert!(comment_token.range.start() > TextSize::from(8u32));
    }

    #[test]
    fn test_parameter() {
        let sql = "SELECT $1, $2";
        let tokens = compute_tokens(sql);

        let param_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Parameter as u32)
            .collect();

        assert_eq!(param_tokens.len(), 2);
    }

    #[test]
    fn test_operators() {
        let sql = "SELECT 1 + 2 * 3";
        let tokens = compute_tokens(sql);

        let operator_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Operator as u32)
            .collect();

        assert_eq!(operator_tokens.len(), 2);
    }

    #[test]
    fn test_type_keyword() {
        let sql = "SELECT 1::integer";
        let tokens = compute_tokens(sql);

        // 'integer' should be a type token
        let type_token = tokens
            .iter()
            .find(|t| {
                let start: usize = t.range.start().into();
                let end: usize = t.range.end().into();
                &sql[start..end] == "integer"
            })
            .expect("Should find integer token");

        assert_eq!(type_token.token_type, TokenType::Type as u32);
    }

    #[test]
    fn test_multiline_comment() {
        let sql = "SELECT /* this is\na multiline\ncomment */ 1";
        let tokens = compute_tokens(sql);

        // Should find a comment token spanning multiple lines
        let comment_token = tokens
            .iter()
            .find(|t| t.token_type == TokenType::Comment as u32)
            .expect("Should find comment token");

        // The token should span from "/*" to "*/"
        let start: usize = comment_token.range.start().into();
        let end: usize = comment_token.range.end().into();
        let comment_text = &sql[start..end];

        assert!(comment_text.starts_with("/*"));
        assert!(comment_text.ends_with("*/"));
        assert!(comment_text.contains('\n'), "Comment should be multi-line");
    }

    #[test]
    fn test_multiline_string() {
        let sql = "SELECT 'hello\nworld'";
        let tokens = compute_tokens(sql);

        // Should find a string token spanning multiple lines
        let string_token = tokens
            .iter()
            .find(|t| t.token_type == TokenType::String as u32)
            .expect("Should find string token");

        let start: usize = string_token.range.start().into();
        let end: usize = string_token.range.end().into();
        let string_text = &sql[start..end];

        assert!(string_text.contains('\n'), "String should be multi-line");
    }

    #[test]
    fn test_token_type_legend() {
        let legend = TokenType::legend();
        assert_eq!(legend[0], "keyword");
        assert_eq!(legend[1], "type");
        assert_eq!(legend[2], "function");
        assert_eq!(legend[3], "parameter");
        assert_eq!(legend[4], "string");
        assert_eq!(legend[5], "number");
        assert_eq!(legend[6], "operator");
        assert_eq!(legend[7], "comment");
        assert_eq!(legend[8], "property");
    }

    #[test]
    fn test_token_modifier_legend() {
        let legend = TokenModifier::legend();
        assert_eq!(legend[0], "declaration");
        assert_eq!(legend[1], "definition");
        assert_eq!(legend[2], "readonly");
        assert_eq!(legend[3], "defaultLibrary");
    }

    #[test]
    fn test_token_modifier_bits() {
        assert_eq!(TokenModifier::Declaration.bit(), 1 << 0);
        assert_eq!(TokenModifier::Definition.bit(), 1 << 1);
        assert_eq!(TokenModifier::Readonly.bit(), 1 << 2);
        assert_eq!(TokenModifier::DefaultLibrary.bit(), 1 << 3);
    }
}
