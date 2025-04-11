use pgt_text_size::TextSize;

use crate::{
    builder::CompletionBuilder,
    context::CompletionContext,
    item::CompletionItem,
    providers::{complete_columns, complete_functions, complete_tables},
    sanitization::SanitizedCompletionParams,
};

pub const LIMIT: usize = 50;

#[derive(Debug)]
pub struct CompletionParams<'a> {
    pub position: TextSize,
    pub schema: &'a pgt_schema_cache::SchemaCache,
    pub text: String,
    pub tree: Option<&'a tree_sitter::Tree>,
}

#[tracing::instrument(level = "debug", skip_all, fields(
    text = params.text,
    position = params.position.to_string()
))]
pub fn complete(params: CompletionParams) -> Vec<CompletionItem> {
    let sanitized_params = match SanitizedCompletionParams::try_from(params) {
        Ok(p) => p,
        Err(err) => {
            tracing::warn!("Not possible to get completions: {}", err);
            return vec![];
        }
    };

    let ctx = CompletionContext::new(&sanitized_params);

    let mut builder = CompletionBuilder::new();

    complete_tables(&ctx, &mut builder);
    complete_functions(&ctx, &mut builder);
    complete_columns(&ctx, &mut builder);

    builder.finish()
}
