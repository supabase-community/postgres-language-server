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
    "ESC_STRING",
    "FLOAT_NUMBER",
    "INT_NUMBER",
    "NULL",
    "STRING",
    "IDENT",
];

const VARIANT_DATA: &[(&str, &str)] = &[
    ("String", "String"),
    ("EscString", "String"),          // E'hello\nworld'
    ("DollarQuotedString", "String"), // $$hello world$$
    ("IntNumber", "i64"),             // 123, -456
    ("FloatNumber", "f64"),           // 123.45, 1.2e-3
    ("BitString", "String"),          // B'1010', X'FF'
    ("ByteString", "String"),         // Similar to bit string
    ("Ident", "String"),              // user_id, table_name
    ("PositionalParam", "u32"),       // $1, $2, $3 (the number matters!)
    ("Comment", "String"),            // /* comment text */
];

pub fn token_kind_mod() -> proc_macro2::TokenStream {
    let keywords = keyword_kinds().expect("Failed to get keyword kinds");

    let KeywordKinds { all_keywords, .. } = keywords;

    let mut enum_variants: Vec<TokenStream> = Vec::new();
    let mut from_kw_match_arms: Vec<TokenStream> = Vec::new();

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
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
    }
}
