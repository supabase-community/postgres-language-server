use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::keywords::{KeywordKinds, keyword_kinds};

/// The whitespace kinds and how a message names them.
const WHITESPACE: &[(&str, &str)] = &[
    ("SPACE", "a space"),
    ("TAB", "a tab"),
    ("VERTICAL_TAB", "a vertical tab"),
    ("FORM_FEED", "a form feed"),
    // "\n" or "\r" in any combination
    ("LINE_ENDING", "a line break"),
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
    ("..", "DOT_DOT"),
    (":", "COLON"),
    ("::", "DOUBLE_COLON"),
    (":=", "COLON_EQUALS"),
    ("=", "EQ"),
    ("!", "BANG"),
    ("-", "MINUS"),
    ("`", "BACKTICK"),
];

/// The other kinds of tokens and how a message names them.
const EXTRA: &[(&str, &str)] = &[
    ("POSITIONAL_PARAM", "a positional parameter"),
    ("NAMED_PARAM", "a named parameter"),
    ("OPERATOR", "an operator"),
    ("META_COMMAND", "a psql meta-command"),
    ("COPY_DATA", "COPY data"),
    ("ERROR", "an invalid token"),
    ("COMMENT", "a comment"),
    ("EOF", "the end of the input"),
];

/// The literal kinds and how a message names them.
const LITERALS: &[(&str, &str)] = &[
    ("BIT_STRING", "a bit string"),
    ("BYTE_STRING", "a hexadecimal string"),
    ("DOLLAR_QUOTED_STRING", "a dollar-quoted string"),
    ("ESC_STRING", "an escape string"),
    ("FLOAT_NUMBER", "a number"),
    ("INT_NUMBER", "an integer"),
    ("NULL", "\"NULL\""),
    ("STRING", "a string"),
    ("IDENT", "an identifier"),
];

pub(crate) fn syntax_kind_mod() -> proc_macro2::TokenStream {
    let keywords = match keyword_kinds() {
        Ok(keywords) => keywords,
        Err(error) => {
            let message = format!("failed to read the keywords of kwlist.h: {error}");
            return quote! { compile_error!(#message); };
        }
    };

    let KeywordKinds { all_keywords, .. } = keywords;

    let mut enum_variants: Vec<TokenStream> = Vec::new();
    let mut from_kw_match_arms: Vec<TokenStream> = Vec::new();
    let mut is_kw_match_arms: Vec<TokenStream> = Vec::new();
    let mut keyword_category_match_arms: Vec<TokenStream> = Vec::new();

    let mut is_trivia_match_arms: Vec<TokenStream> = Vec::new();
    let mut description_match_arms: Vec<TokenStream> = Vec::new();

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
        let description = format!("\"{}\"", name.to_uppercase());
        description_match_arms.push(quote! {
            SyntaxKind::#kind_ident => #description
        });
    }

    // collect extra keywords
    EXTRA.iter().for_each(|&(name, description)| {
        let variant_name = format_ident!("{}", name);
        enum_variants.push(quote! { #variant_name });
        description_match_arms.push(quote! {
            SyntaxKind::#variant_name => #description
        });

        if name == "COMMENT" {
            is_trivia_match_arms.push(quote! {
                SyntaxKind::#variant_name => true
            });
        }
    });

    // collect whitespace variants
    WHITESPACE.iter().for_each(|&(name, description)| {
        let variant_name = format_ident!("{}", name);
        enum_variants.push(quote! { #variant_name });
        description_match_arms.push(quote! {
            SyntaxKind::#variant_name => #description
        });
        is_trivia_match_arms.push(quote! {
            SyntaxKind::#variant_name => true
        });
    });

    // collect punctuations
    PUNCT.iter().for_each(|&(ascii_name, variant)| {
        let variant_name = format_ident!("{}", variant);
        enum_variants.push(quote! { #variant_name });
        let description = format!("\"{ascii_name}\"");
        description_match_arms.push(quote! {
            SyntaxKind::#variant_name => #description
        });
    });

    // collect literals
    LITERALS.iter().for_each(|&(name, description)| {
        let variant_name = format_ident!("{}", name);
        enum_variants.push(quote! { #variant_name });
        description_match_arms.push(quote! {
            SyntaxKind::#variant_name => #description
        });
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

            /// How a message names the token: a keyword or punctuation mark in quotes, like
            /// `"AS"` or `"("`, or what any other token is, like `an identifier`.
            pub fn description(&self) -> &'static str {
                match self {
                    #(#description_match_arms),*
                }
            }
        }
    }
}
