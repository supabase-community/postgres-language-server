mod merge_derive;
mod partial_derive;
mod util;

use proc_macro::TokenStream;
use proc_macro_error::*;
use syn::{DeriveInput, parse_macro_input};

/// Derives the `pgls_configuration::Merge` trait for a custom enum or struct.
#[proc_macro_derive(Merge)]
#[proc_macro_error]
pub fn derive_mergeable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input = merge_derive::DeriveInput::parse(input);

    let tokens = merge_derive::generate_merge(input);

    TokenStream::from(tokens)
}

/// Generates a "partial" struct from another.
///
/// A partial struct has the same shape as the struct it is derived from (the
/// "full" struct), but with all its fields wrapped in `Option`. Fields that
/// were already wrapped in an `Option` don't get wrapped again.
///
/// The name of the generated partial struct is `Partial{FullStruct}`.
#[proc_macro_derive(Partial, attributes(partial))]
#[proc_macro_error]
pub fn derive_partial(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input = partial_derive::DeriveInput::parse(input);

    let tokens = partial_derive::generate_partial(input);

    TokenStream::from(tokens)
}
