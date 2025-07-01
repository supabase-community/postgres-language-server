use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::keywords::{KeywordKinds, keyword_kinds};

const WHITESPACE: &[&str] = &[
    "SPACE",           // " "
    "TAB",             // "\t"
    "NEWLINE",         // "\n"
    "CARRIAGE_RETURN", // "\r"
    "VERTICAL_TAB",    // "\x0B"
    "FORM_FEED",       // "\x0C"
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
    ("^", "CARET"),
    ("%", "PERCENT"),
    ("_", "UNDERSCORE"),
    (".", "DOT"),
    (":", "COLON"),
    ("=", "EQ"),
    ("!", "BANG"),
    ("-", "MINUS"),
    ("`", "BACKTICK"),
];

const EXTRA: &[&str] = &["POSITIONAL_PARAM", "ERROR", "COMMENT", "EOF"];

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
    }

    // collect extra keywords
    EXTRA.iter().for_each(|&name| {
        let variant_name = format_ident!("{}", name);
        enum_variants.push(quote! { #variant_name });
    });

    // collect whitespace variants
    WHITESPACE.iter().for_each(|&name| {
        let variant_name = format_ident!("{}", name);
        enum_variants.push(quote! { #variant_name });
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
        }
    }
}
