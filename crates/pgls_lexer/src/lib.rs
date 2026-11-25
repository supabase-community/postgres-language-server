mod codegen;
mod lexed;
mod lexer;
mod params;

pub use crate::codegen::syntax_kind::{KeywordCategory, SyntaxKind};
pub use crate::lexed::{LexDiagnostic, Lexed};
pub use crate::lexer::Lexer;
pub use crate::params::{
    NamedParameterConversion, convert_to_positional_params,
    convert_to_positional_params_with_metadata,
};

/// The category of the keyword `word`, compared case-insensitively. `None` if it's no keyword.
pub fn keyword_category(word: &str) -> Option<KeywordCategory> {
    SyntaxKind::from_keyword(word)?.keyword_category()
}

/// Lex the input string into tokens and diagnostics
pub fn lex(input: &str) -> Lexed<'_> {
    Lexer::new(input).lex()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgls_text_size::TextRange;

    #[test]
    fn test_basic_lexing() {
        let input = "SELECT * FROM users WHERE id = 1;";
        let lexed = lex(input);

        // Check we have tokens
        assert!(!lexed.is_empty());

        // Iterate over tokens and collect identifiers
        let mut identifiers = Vec::new();
        for (idx, kind) in lexed.tokens().enumerate() {
            if kind == SyntaxKind::IDENT {
                identifiers.push((lexed.text(idx), lexed.range(idx)));
            }
        }

        // Should find at least "users" and "id" as identifiers
        assert!(identifiers.len() >= 2);
    }

    #[test]
    fn test_lexing_with_errors() {
        let input = "SELECT 'unterminated string";
        let lexed = lex(input);

        // Should have tokens
        assert!(!lexed.is_empty());

        // Should have an error for unterminated string
        let errors = lexed.errors();
        assert!(!errors.is_empty());
        // Check the error message exists
        assert!(!errors[0].message.to_string().is_empty());
    }

    #[test]
    fn test_lexing_string_params_with_errors() {
        let input = "SELECT :'unterminated string";
        let lexed = lex(input);

        // Should have tokens
        assert!(!lexed.is_empty());

        // Should have an error for unterminated string
        let errors = lexed.errors();
        assert!(!errors.is_empty());
        // Check the error message exists
        assert!(!errors[0].message.to_string().is_empty());
    }

    #[test]
    fn test_lexing_identifier_params_with_errors() {
        let input = "SELECT :\"unterminated string";
        let lexed = lex(input);

        // Should have tokens
        assert!(!lexed.is_empty());

        // Should have an error for unterminated string
        let errors = lexed.errors();
        assert!(!errors.is_empty());
        // Check the error message exists
        assert!(!errors[0].message.to_string().is_empty());
    }

    #[test]
    fn test_token_ranges() {
        let input = "SELECT id";
        let lexed = lex(input);

        // First token should be a keyword (SELECT gets parsed as a keyword)
        let _first_kind = lexed.kind(0);
        assert_eq!(u32::from(lexed.range(0).start()), 0);
        assert_eq!(u32::from(lexed.range(0).end()), 6);
        assert_eq!(lexed.text(0), "SELECT");

        // Find the id token
        for (idx, kind) in lexed.tokens().enumerate() {
            if kind == SyntaxKind::IDENT && lexed.text(idx) == "id" {
                assert_eq!(u32::from(lexed.range(idx).start()), 7);
                assert_eq!(u32::from(lexed.range(idx).end()), 9);
            }
        }
    }

    #[test]
    fn eof_and_past_the_last_token() {
        let lexed = lex("SELECT");
        let eof = lexed.len() - 1;
        assert_eq!(lexed.kind(eof), SyntaxKind::EOF);
        assert_eq!(lexed.text(eof), "");
        assert_eq!(lexed.range(eof), TextRange::empty(6.into()));
        assert_eq!(lexed.kind(eof + 1), SyntaxKind::EOF);
        assert_eq!(lexed.text(eof + 1), "");
        assert_eq!(lexed.range(eof + 1), TextRange::empty(6.into()));
    }

    #[test]
    fn descriptions_name_tokens_for_messages() {
        for (kind, description) in [
            (SyntaxKind::AS_KW, "\"AS\""),
            (SyntaxKind::L_PAREN, "\"(\""),
            (SyntaxKind::BACKSLASH, "\"\\\""),
            (SyntaxKind::IDENT, "an identifier"),
            (SyntaxKind::EOF, "the end of the input"),
        ] {
            assert_eq!(kind.description(), description, "{kind:?}");
        }
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let lexed = lex(input);
        assert_eq!(lexed.len(), 1);
        assert_eq!(lexed.kind(0), SyntaxKind::EOF);
        assert!(!lexed.has_blank_line());
    }

    #[test]
    fn test_blank_line_detection() {
        assert!(!lex("SELECT 1\nSELECT 2").has_blank_line());
        assert!(lex("SELECT 1\n\nSELECT 2").has_blank_line());
    }

    #[test]
    fn test_whitespace_handling() {
        let input = "  SELECT  \n  id  ";
        let lexed = lex(input);

        // Collect non-whitespace tokens
        let mut non_whitespace = Vec::new();
        for (idx, kind) in lexed.tokens().enumerate() {
            if !matches!(
                kind,
                SyntaxKind::SPACE | SyntaxKind::TAB | SyntaxKind::LINE_ENDING | SyntaxKind::EOF
            ) {
                non_whitespace.push(lexed.text(idx));
            }
        }

        assert_eq!(non_whitespace.len(), 2); // SELECT and id
    }

    /// The lexer cuts what the fuzz target `lexer_vs_scan` found into the tokens of Postgres'
    /// scanner: non-ASCII characters in identifiers, string constants continued across a line
    /// break, and `''` in bit and hex strings.
    #[test]
    fn tokens_are_the_scanners() {
        for sql in [
            "SELECT\u{6dd}1",
            "ΑΣ",
            "SELECT ΑΣ, aΑ1, $ΑΣ$x$ΑΣ$",
            "'foo'\n'bar'",
            "SELECT 'a' -- c\n\n  -- d\n\t'b'\r\n'c' AS x",
            "E'a'\n'b', U&'a'\n'b', b'1'\n'0', x'a'\n'b'",
            "'a' 'b', 'a'\n-- c",
            "b''''",
            "x'ab'''",
            "'it''s'",
        ] {
            let lexed = lex(sql);
            let lexer: Vec<_> = (0..lexed.len())
                .map(|idx| lexed.range(idx))
                .filter(|&range| !sql[range].trim().is_empty())
                .map(|range| (u32::from(range.start()), u32::from(range.end())))
                .collect();
            let scanner: Vec<_> = pgls_query::scan(sql)
                .unwrap()
                .tokens
                .iter()
                .map(|token| (token.start as u32, token.end as u32))
                .collect();
            assert_eq!(lexer, scanner, "{sql:?}");
        }
    }

    /// The data rows after `COPY … FROM STDIN` are one token, up to a line that is exactly `\.`,
    /// and aren't lexed as SQL. Without a `\.`, they stay SQL.
    #[test]
    fn copy_data() {
        for (sql, data) in [
            // pg_dump: tabs, `\N` and backslashes, and a row that looks like SQL
            (
                "COPY public.actor (actor_id, name) FROM stdin;\n1\tPENELOPE\t\\N\ta\\\\b\n2\tselect 1;\tx\n\\.\n\nSELECT 1;",
                vec!["1\tPENELOPE\t\\N\ta\\\\b\n2\tselect 1;\tx\n\\."],
            ),
            // no `\.`, or one at the end of the text
            ("copy t from STDIN;\n1\n'2\n", vec![]),
            (
                "copy t from stdin;\nselect 1;\ncopy u from stdin;\n1\n",
                vec![],
            ),
            ("copy t from STDIN;\n1\n\\.", vec!["1\n\\."]),
            // two blocks in a row, with options, and `\.` inside a row
            (
                "copy a from stdin;\n1\n\\.\ncopy b (x) from stdin with (format csv); -- c\n2 \\.\n\\.\r\nselect 1;",
                vec!["1\n\\.", "2 \\.\n\\."],
            ),
            ("copy t from stdin;\r\n1\r\n\\.\r\n", vec!["1\r\n\\."]),
            ("\\copy t (a) from stdin with csv\n1\n\\.\n", vec!["1\n\\."]),
            ("copy t from stdin; select 1;\n1\n\\.\n", vec!["1\n\\."]),
            ("copy t from stdin;\n", vec![]),
            // no data
            ("copy t to stdout;\n1\n", vec![]),
            ("copy (select 1 from stdin) to stdout;\n1\n", vec![]),
            ("copy t from 'f';\n1\n", vec![]),
            ("copy \"t\" from \"stdin\";\n1\n", vec![]),
            ("\\copy t from pstdin\n1\n", vec![]),
            ("\\copyright\nselect 1 from stdin;\n1\n", vec![]),
            ("select 1; copy t from stdin\n1\n", vec![]),
        ] {
            let lexed = lex(sql);
            let found: Vec<_> = (0..lexed.len())
                .filter(|&idx| lexed.kind(idx) == SyntaxKind::COPY_DATA)
                .map(|idx| lexed.text(idx))
                .collect();
            assert_eq!(found, data, "{sql:?}");
        }
    }

    #[test]
    fn finds_lex_errors() {
        // Test with unterminated block comment
        let input = "/* unterminated comment";
        let lexed = lex(input);
        let errors = lexed.errors();

        // Should have error for unterminated block comment
        assert!(!errors.is_empty());
        assert!(errors[0].message.to_string().contains("Missing trailing"));
        assert!(errors[0].span.start() < errors[0].span.end());

        // Test with unterminated string
        let input2 = "SELECT 'unterminated string";
        let lexed2 = lex(input2);
        let errors2 = lexed2.errors();

        // Should have error for unterminated string
        assert!(!errors2.is_empty());
        assert!(errors2[0].message.to_string().contains("Missing trailing"));
    }
}
