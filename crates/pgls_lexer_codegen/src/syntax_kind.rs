use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::keywords::{KeywordKinds, keyword_kinds};

const WHITESPACE: &[&str] = &[
    "SPACE",        // " "
    "TAB",          // "\t"
    "VERTICAL_TAB", // "\x0B"
    "FORM_FEED",    // "\x0C"
    "LINE_ENDING",  // "\n" or "\r" in any combination
];

const PUNCT: &[(&str, &str)] = &[
    ("$", "DOLLAR"),
    (";", "SEMICOLON"),
    (",", "COMMA"),
    ("(", "L_PAREN"),
    (")", "R_PAREN"),
    ("[", "L_BRACK"),
    ("]", "R_BRACK"),
    ("<", "L_ANGLE"),
    (">", "R_ANGLE"),
    ("@", "AT"),
    ("#", "POUND"),
    ("~", "TILDE"),
    ("?", "QUESTION"),
    ("&", "AMP"),
    ("|", "PIPE"),
    ("+", "PLUS"),
    ("*", "STAR"),
    ("/", "SLASH"),
    ("\\", "BACKSLASH"),
    ("^", "CARET"),
    ("%", "PERCENT"),
    ("_", "UNDERSCORE"),
    (".", "DOT"),
    (":", "COLON"),
    ("::", "DOUBLE_COLON"),
    ("=", "EQ"),
    ("!", "BANG"),
    ("-", "MINUS"),
    ("`", "BACKTICK"),
];

const EXTRA: &[&str] = &["POSITIONAL_PARAM", "NAMED_PARAM", "ERROR", "COMMENT", "EOF"];

const LITERALS: &[&str] = &[
    "BIT_STRING",
    "BYTE_STRING",
    "DOLLAR_QUOTED_STRING",
    "ESC_STRING",
    "FLOAT_NUMBER",
    "INT_NUMBER",
    "NULL",
    "STRING",
    "IDENT",
];

pub fn syntax_kind_mod() -> proc_macro2::TokenStream {
    let keywords = keyword_kinds().expect("Failed to get keyword kinds");

    let KeywordKinds { all_keywords, .. } = keywords;

    let mut enum_variants: Vec<TokenStream> = Vec::new();
    let mut from_kw_match_arms: Vec<TokenStream> = Vec::new();
    let mut is_kw_match_arms: Vec<TokenStream> = Vec::new();
    let mut keyword_category_match_arms: Vec<TokenStream> = Vec::new();

    let mut is_trivia_match_arms: Vec<TokenStream> = Vec::new();

    // collect keywords
    for kw in &all_keywords {
        let name = &kw.name;
        if name.to_uppercase().contains("WHITESPACE") {
            continue; // Skip whitespace as it is handled separately
        }

        let kind_ident = format_ident!("{}_KW", name.to_case(Case::UpperSnake));
        let category = match kw.category.as_str() {
            "UNRESERVED_KEYWORD" => format_ident!("Unreserved"),
            "COL_NAME_KEYWORD" => format_ident!("ColName"),
            "TYPE_FUNC_NAME_KEYWORD" => format_ident!("TypeFuncName"),
            "RESERVED_KEYWORD" => format_ident!("Reserved"),
            other => panic!("Unknown keyword category {other} for keyword {name}"),
        };

        enum_variants.push(quote! { #kind_ident });
        from_kw_match_arms.push(quote! {
            #name => Some(SyntaxKind::#kind_ident)
        });
        is_kw_match_arms.push(quote! {
            SyntaxKind::#kind_ident => true
        });
        keyword_category_match_arms.push(quote! {
            SyntaxKind::#kind_ident => Some(KeywordCategory::#category)
        });
    }

    // collect extra keywords
    EXTRA.iter().for_each(|&name| {
        let variant_name = format_ident!("{}", name);
        enum_variants.push(quote! { #variant_name });

        if name == "COMMENT" {
            is_trivia_match_arms.push(quote! {
                SyntaxKind::#variant_name => true
            });
        }
    });

    // collect whitespace variants
    WHITESPACE.iter().for_each(|&name| {
        let variant_name = format_ident!("{}", name);
        enum_variants.push(quote! { #variant_name });
        is_trivia_match_arms.push(quote! {
            SyntaxKind::#variant_name => true
        });
    });

    // collect punctuations
    PUNCT.iter().for_each(|&(_ascii_name, variant)| {
        let variant_name = format_ident!("{}", variant);
        enum_variants.push(quote! { #variant_name });
    });

    // collect literals
    LITERALS.iter().for_each(|&name| {
        let variant_name = format_ident!("{}", name);
        enum_variants.push(quote! { #variant_name });
    });

    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        #[repr(u16)]
        pub enum SyntaxKind {
            #(#enum_variants),*,
        }

        /// How reserved a keyword is, i.e. where Postgres accepts it as a name.
        ///
        /// See the categories in https://github.com/postgres/postgres/blob/REL_18_6/src/include/parser/kwlist.h
        /// and the name classification hierarchy in https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L17619
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum KeywordCategory {
            /// Available for use as any kind of name.
            Unreserved,
            /// Can be a column or table name, but not a function or type name.
            ColName,
            /// Can be a function or type name, but not a column or table name.
            TypeFuncName,
            /// Can only be a column label after `AS`.
            Reserved,
        }

        impl SyntaxKind {
            pub(crate) fn from_keyword(ident: &str) -> Option<SyntaxKind> {
                let lower_ident = ident.to_ascii_lowercase();
                match lower_ident.as_str() {
                    #(#from_kw_match_arms),*,
                    _ => None
                }
            }

            pub fn is_keyword(&self) -> bool {
                match self {
                    #(#is_kw_match_arms),*,
                    _ => false
                }
            }

            /// The category of a keyword, `None` for any other token.
            pub fn keyword_category(&self) -> Option<KeywordCategory> {
                match self {
                    #(#keyword_category_match_arms),*,
                    _ => None
                }
            }

            pub fn is_trivia(&self) -> bool {
                match self {
                    #(#is_trivia_match_arms),*,
                    _ => false
                }
            }
        }
    }
}
