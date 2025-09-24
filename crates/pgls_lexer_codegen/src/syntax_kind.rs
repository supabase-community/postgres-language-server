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

    let mut is_trivia_match_arms: Vec<TokenStream> = Vec::new();

    // collect keywords
    for kw in &all_keywords {
        if kw.to_uppercase().contains("WHITESPACE") {
            continue; // Skip whitespace as it is handled separately
        }

        let kind_ident = format_ident!("{}_KW", kw.to_case(Case::UpperSnake));

        enum_variants.push(quote! { #kind_ident });
        from_kw_match_arms.push(quote! {
            #kw => Some(SyntaxKind::#kind_ident)
        });
        is_kw_match_arms.push(quote! {
            SyntaxKind::#kind_ident => true
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

            pub fn is_trivia(&self) -> bool {
                match self {
                    #(#is_trivia_match_arms),*,
                    _ => false
                }
            }
        }
    }
}
