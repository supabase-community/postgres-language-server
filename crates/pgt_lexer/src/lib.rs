mod codegen;
pub mod diagnostics;

use diagnostics::ScanError;
use pg_query::protobuf::{KeywordKind, ScanToken};
use pgt_text_size::{TextLen, TextRange, TextSize};
use regex::Regex;
use std::{collections::VecDeque, sync::LazyLock};

pub use crate::codegen::SyntaxKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Whitespace,
    NoKeyword,
    UnreservedKeyword,
    ColNameKeyword,
    TypeFuncNameKeyword,
    ReservedKeyword,
}

impl From<&ScanToken> for TokenType {
    fn from(token: &ScanToken) -> TokenType {
        match token.token {
            // SqlComment | CComment
            275 | 276 => TokenType::Whitespace,
            _ => match token.keyword_kind() {
                KeywordKind::NoKeyword => TokenType::NoKeyword,
                KeywordKind::UnreservedKeyword => TokenType::UnreservedKeyword,
                KeywordKind::ColNameKeyword => TokenType::ColNameKeyword,
                KeywordKind::TypeFuncNameKeyword => TokenType::TypeFuncNameKeyword,
                KeywordKind::ReservedKeyword => TokenType::ReservedKeyword,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub text: String,
    pub span: TextRange,
    pub token_type: TokenType,
}

impl Token {
    pub fn eof(pos: usize) -> Token {
        Token {
            kind: SyntaxKind::Eof,
            text: "".to_string(),
            span: TextRange::at(TextSize::try_from(pos).unwrap(), TextSize::from(0)),
            token_type: TokenType::Whitespace,
        }
    }
}

pub static WHITESPACE_TOKENS: &[SyntaxKind] = &[
    SyntaxKind::Whitespace,
    SyntaxKind::Tab,
    SyntaxKind::Newline,
    SyntaxKind::SqlComment,
    SyntaxKind::CComment,
];

static PATTERN_LEXER: LazyLock<Regex> = LazyLock::new(|| {
    #[cfg(windows)]
    {
        // On Windows, treat \r\n as a single newline token
        // and treat \r as a whitespace token
        Regex::new(r"(?P<whitespace> (+|\r))|(?P<newline>(\r\n|\n)+)|(?P<tab>\t+)").unwrap()
    }
    #[cfg(not(windows))]
    {
        // On other platforms, just check for \n
        Regex::new(r"(?P<whitespace> +)|(?P<newline>\n+)|(?P<tab>\t+)").unwrap()
    }
});

fn whitespace_tokens(input: &str) -> VecDeque<Token> {
    let mut tokens = VecDeque::new();

    for cap in PATTERN_LEXER.captures_iter(input) {
        if let Some(whitespace) = cap.name("whitespace") {
            tokens.push_back(Token {
                token_type: TokenType::Whitespace,
                kind: SyntaxKind::Whitespace,
                text: whitespace.as_str().to_string(),
                span: TextRange::new(
                    TextSize::from(u32::try_from(whitespace.start()).unwrap()),
                    TextSize::from(u32::try_from(whitespace.end()).unwrap()),
                ),
            });
        } else if let Some(newline) = cap.name("newline") {
            tokens.push_back(Token {
                token_type: TokenType::Whitespace,
                kind: SyntaxKind::Newline,
                text: newline.as_str().to_string(),
                span: TextRange::new(
                    TextSize::from(u32::try_from(newline.start()).unwrap()),
                    TextSize::from(u32::try_from(newline.end()).unwrap()),
                ),
            });
        } else if let Some(tab) = cap.name("tab") {
            tokens.push_back(Token {
                token_type: TokenType::Whitespace,
                kind: SyntaxKind::Tab,
                text: tab.as_str().to_string(),
                span: TextRange::new(
                    TextSize::from(u32::try_from(tab.start()).unwrap()),
                    TextSize::from(u32::try_from(tab.end()).unwrap()),
                ),
            });
        } else {
            panic!("No match");
        };
    }

    tokens
}

/// Turn a string of potentially valid sql code into a list of tokens, including their range in the source text.
///
/// The implementation is primarily using libpg_querys `scan` method, and fills in the gaps with tokens that are not parsed by the library, e.g. whitespace.
pub fn lex(text: &str) -> Result<Vec<Token>, Vec<ScanError>> {
    let mut whitespace_tokens = whitespace_tokens(text);

    // tokens from pg_query.rs
    let mut pgt_query_tokens = match pg_query::scan(text) {
        Ok(r) => r.tokens.into_iter().collect::<VecDeque<_>>(),
        Err(err) => return Err(ScanError::from_pg_query_err(err, text)),
    };

    // merge the two token lists
    let mut tokens: Vec<Token> = Vec::new();
    let mut pos = TextSize::from(0);

    while pos < text.text_len() {
        if !pgt_query_tokens.is_empty()
            && TextSize::from(u32::try_from(pgt_query_tokens[0].start).unwrap()) == pos
        {
            let pgt_query_token = pgt_query_tokens.pop_front().unwrap();

            // the lexer returns byte indices, so we need to slice
            let token_text = &text[usize::try_from(pgt_query_token.start).unwrap()
                ..usize::try_from(pgt_query_token.end).unwrap()];

            let len = token_text.text_len();
            let has_whitespace = token_text.contains(" ") || token_text.contains("\n");
            tokens.push(Token {
                token_type: TokenType::from(&pgt_query_token),
                kind: SyntaxKind::from(&pgt_query_token),
                text: token_text.to_string(),
                span: TextRange::new(
                    TextSize::from(u32::try_from(pgt_query_token.start).unwrap()),
                    TextSize::from(u32::try_from(pgt_query_token.end).unwrap()),
                ),
            });
            pos += len;

            if has_whitespace {
                while !whitespace_tokens.is_empty()
                    && whitespace_tokens[0].span.start() < TextSize::from(u32::from(pos))
                {
                    whitespace_tokens.pop_front();
                }
            }

            continue;
        }

        if !whitespace_tokens.is_empty()
            && whitespace_tokens[0].span.start() == TextSize::from(u32::from(pos))
        {
            let whitespace_token = whitespace_tokens.pop_front().unwrap();
            let len = whitespace_token.text.text_len();
            tokens.push(whitespace_token);
            pos += len;
            continue;
        }

        let usize_pos = usize::from(pos);
        panic!(
            "No token found at position {:?}: '{:?}'",
            pos,
            text.get(usize_pos..usize_pos + 1)
        );
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_chars() {
        let input = "insert into c (name, full_name) values ('Å', 1);";
        let tokens = lex(input).unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_tab_tokens() {
        let input = "select\t1";
        let tokens = lex(input).unwrap();
        assert_eq!(tokens[1].kind, SyntaxKind::Tab);
    }

    #[test]
    #[cfg(windows)]
    fn test_carriage_return() {
        let input = "select\r\n\r1";
        let tokens = lex(input).unwrap();
        assert_eq!(tokens[1].kind, SyntaxKind::Newline);
        assert_eq!(tokens[2].kind, SyntaxKind::Whitespace);
    }

    #[test]
    fn test_newline_tokens() {
        let input = "select\n1";
        let tokens = lex(input).unwrap();
        assert_eq!(tokens[1].kind, SyntaxKind::Newline);
    }

    #[test]
    fn test_consecutive_newlines() {
        // Test with multiple consecutive newlines
        #[cfg(windows)]
        let input = "select\r\n\r\n\r1";
        #[cfg(not(windows))]
        let input = "select\n\n1";

        let tokens = lex(input).unwrap();

        // Check that we have exactly one newline token between "select" and "1"
        assert_eq!(tokens[0].kind, SyntaxKind::Select);
        assert_eq!(tokens[1].kind, SyntaxKind::Newline);
        assert_eq!(tokens[1].kind, SyntaxKind::Whitespace);
        assert_eq!(tokens[2].kind, SyntaxKind::Iconst);
    }

    #[test]
    fn test_whitespace_tokens() {
        let input = "select 1";
        let tokens = lex(input).unwrap();
        assert_eq!(tokens[1].kind, SyntaxKind::Whitespace);
    }

    #[test]
    fn test_lexer() {
        let input = "select 1; \n -- some comment \n select 2\t";

        let tokens = lex(input).unwrap();
        let mut tokens_iter = tokens.iter();

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Select);
        assert_eq!(token.text, "select");

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Whitespace);

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Iconst);
        assert_eq!(token.text, "1");

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Ascii59);

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Whitespace);

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Newline);

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Whitespace);

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::SqlComment);
        assert_eq!(token.text, "-- some comment ");

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Newline);

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Whitespace);

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Select);
        assert_eq!(token.text, "select");

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Whitespace);

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Iconst);
        assert_eq!(token.text, "2");

        let token = tokens_iter.next().unwrap();
        assert_eq!(token.kind, SyntaxKind::Tab);
    }
}
