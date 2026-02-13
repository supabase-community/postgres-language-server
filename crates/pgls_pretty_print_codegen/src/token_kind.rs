use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::keywords::{KeywordKinds, keyword_kinds};

const STRUCTURAL_PUNCT: &[(&str, &str)] = &[
    (";", "SEMICOLON"), // Statement terminator - structural
    (",", "COMMA"),     // List separator - structural
    ("(", "L_PAREN"),   // Grouping - structural
    (")", "R_PAREN"),   // Grouping - structural
    ("[", "L_BRACK"),   // Array indexing - structural
    ("]", "R_BRACK"),   // Array indexing - structural
    (".", "DOT"),       // Qualified names (schema.table) - structural
];

const PUNCT: &[(&str, &str)] = &[
    ("$", "DOLLAR"),        // Positional parameters ($1, $2) - special parsing
    ("::", "DOUBLE_COLON"), // Type cast operator - special syntax
];

const EXTRA: &[&str] = &["POSITIONAL_PARAM", "COMMENT"];

const LITERALS: &[&str] = &[
    "BIT_STRING",
    "BYTE_STRING",
    "DOLLAR_QUOTED_STRING",
    "DOLLAR_QUOTE_DELIMITER", // $function$, $body$, $$, etc.
    "DOLLAR_QUOTE_BODY",      // The content inside dollar quotes (preserved verbatim)
    "ESC_STRING",
    "FLOAT_NUMBER",
    "INT_NUMBER",
    "NULL",
    "STRING",
    "IDENT",
    "BOOLEAN",
    "TYPE_IDENT", // Data type names (text, varchar, int, etc.)
];

const VARIANT_DATA: &[(&str, &str)] = &[
    ("STRING", "String"),
    ("ESC_STRING", "String"),             // E'hello\nworld'
    ("DOLLAR_QUOTED_STRING", "String"),   // $$hello world$$ (legacy, full string)
    ("DOLLAR_QUOTE_DELIMITER", "String"), // $function$, $body$, $$, etc.
    ("DOLLAR_QUOTE_BODY", "String"),      // The content inside dollar quotes
    ("INT_NUMBER", "i64"),                // 123, -456
    ("FLOAT_NUMBER", "f64"),              // 123.45, 1.2e-3
    ("BIT_STRING", "String"),             // B'1010', X'FF'
    ("BYTE_STRING", "String"),            // Similar to bit string
    ("IDENT", "String"),                  // user_id, table_name
    ("POSITIONAL_PARAM", "u32"),          // $1, $2, $3 (the number matters!)
    ("COMMENT", "String"),                // /* comment text */
    ("BOOLEAN", "bool"),                  // true, false
    ("TYPE_IDENT", "String"),             // text, varchar, int (data type names)
];

pub fn token_kind_mod() -> proc_macro2::TokenStream {
    let keywords = keyword_kinds().expect("Failed to get keyword kinds");

    let KeywordKinds { all_keywords, .. } = keywords;

    let mut enum_variants: Vec<TokenStream> = Vec::new();
    let mut from_kw_match_arms: Vec<TokenStream> = Vec::new();
    let mut render_kw_match_arms: Vec<TokenStream> = Vec::new();

    // helper function to create a variant quote for enum
    // used to handle variants with data types
    let variant_quote = |name: &str| {
        let variant_name = format_ident!("{}", name);

        if let Some((_, data_type)) = VARIANT_DATA.iter().find(|&&(n, _)| n == name) {
            let data_type = format_ident!("{}", data_type);
            quote! { #variant_name(#data_type) }
        } else {
            quote! { #variant_name }
        }
    };

    // collect keywords
    for kw in &all_keywords {
        if kw.to_uppercase().contains("WHITESPACE") {
            continue; // Skip whitespace as it is handled separately
        }

        let kind_ident = format_ident!("{}_KW", kw.to_case(Case::UpperSnake));

        enum_variants.push(quote! { #kind_ident });
        from_kw_match_arms.push(quote! {
            #kw => Some(TokenKind::#kind_ident)
        });
        let kw_lower = kw.to_lowercase();
        let kw_upper = kw.to_uppercase();
        render_kw_match_arms.push(quote! {
            TokenKind::#kind_ident => match config.keyword_case {
                crate::renderer::KeywordCase::Upper => #kw_upper.to_string(),
                crate::renderer::KeywordCase::Lower => #kw_lower.to_string(),
            }
        });
    }

    // collect extra keywords
    EXTRA.iter().for_each(|&name| {
        enum_variants.push(variant_quote(name));
    });

    // collect punctuations
    STRUCTURAL_PUNCT.iter().for_each(|&(_ascii_name, variant)| {
        let variant_name = format_ident!("{}", variant);
        enum_variants.push(quote! { #variant_name });
    });
    PUNCT.iter().for_each(|&(_ascii_name, variant)| {
        let variant_name = format_ident!("{}", variant);
        enum_variants.push(quote! { #variant_name });
    });

    // collect literals
    LITERALS.iter().for_each(|&name| {
        enum_variants.push(variant_quote(name));
    });

    quote! {
        #[derive(Clone, PartialEq, Debug)]
        pub enum TokenKind {
            #(#enum_variants),*,
        }

        impl TokenKind {
            pub(crate) fn from_keyword(ident: &str) -> Option<TokenKind> {
                let lower_ident = ident.to_ascii_lowercase();
                match lower_ident.as_str() {
                    #(#from_kw_match_arms),*,
                    _ => None
                }
            }
        }

        impl TokenKind {
            pub fn render(&self, config: &crate::renderer::RenderConfig) -> String {
                match self {
                    TokenKind::SEMICOLON => ";".to_string(),
                    TokenKind::COMMA => ",".to_string(),
                    TokenKind::L_PAREN => "(".to_string(),
                    TokenKind::R_PAREN => ")".to_string(),
                    TokenKind::L_BRACK => "[".to_string(),
                    TokenKind::R_BRACK => "]".to_string(),
                    TokenKind::DOT => ".".to_string(),
                    TokenKind::DOUBLE_COLON => "::".to_string(),
                    TokenKind::DOLLAR => "$".to_string(),
                    TokenKind::IDENT(ident) => ident.clone(),
                    TokenKind::TYPE_IDENT(type_name) => match config.type_case {
                        crate::renderer::KeywordCase::Upper => type_name.to_uppercase(),
                        crate::renderer::KeywordCase::Lower => type_name.to_lowercase(),
                    },
                    TokenKind::STRING(s) => s.clone(),
                    TokenKind::ESC_STRING(s) => s.clone(),
                    TokenKind::DOLLAR_QUOTED_STRING(s) => s.clone(),
                    TokenKind::DOLLAR_QUOTE_DELIMITER(s) => s.clone(),
                    TokenKind::DOLLAR_QUOTE_BODY(s) => s.clone(),
                    TokenKind::INT_NUMBER(n) => n.to_string(),
                    TokenKind::FLOAT_NUMBER(n) => n.to_string(),
                    TokenKind::BIT_STRING(s) => s.clone(),
                    TokenKind::BYTE_STRING(s) => s.clone(),
                    TokenKind::BOOLEAN(b) => match config.constant_case {
                        crate::renderer::KeywordCase::Upper => if *b { "TRUE" } else { "FALSE" }.to_string(),
                        crate::renderer::KeywordCase::Lower => if *b { "true" } else { "false" }.to_string(),
                    },
                    TokenKind::NULL => match config.constant_case {
                        crate::renderer::KeywordCase::Upper => "NULL".to_string(),
                        crate::renderer::KeywordCase::Lower => "null".to_string(),
                    },
                    #(#render_kw_match_arms),*,
                    _ => format!("{:?}", self), // Fallback for other variants
                }
            }
        }
    }
}
