use pgls_text_size::TextSize;

use pgls_treesitter::{TreeSitterContextParams, context::TreesitterContext};

use crate::{
    builder::CompletionBuilder,
    item::CompletionItem,
    providers::{
        complete_columns, complete_functions, complete_keywords, complete_policies, complete_roles,
        complete_schemas, complete_tables,
    },
    sanitization::SanitizedCompletionParams,
};

pub const LIMIT: usize = 50;

#[derive(Debug)]
pub struct CompletionParams<'a> {
    pub position: TextSize,
    pub schema: &'a pgls_schema_cache::SchemaCache,
    pub text: String,
    pub tree: &'a tree_sitter::Tree,
}

#[tracing::instrument(level = "debug", skip_all, fields(
    text = params.text,
    position = params.position.to_string()
))]
pub fn complete(params: CompletionParams) -> Vec<CompletionItem> {
    let use_upper_case = params
        .text
        .split_ascii_whitespace()
        // filter out special chars and numbers
        .filter(|word| word.chars().all(|c| c.is_alphabetic()))
        .any(|t| t == t.to_ascii_uppercase());

    let sanitized_params = SanitizedCompletionParams::from(params);

    let ctx = TreesitterContext::new(TreeSitterContextParams {
        position: sanitized_params.position,
        text: &sanitized_params.text,
        tree: &sanitized_params.tree,
    });

    let mut builder = CompletionBuilder::new(&ctx);

    complete_tables(&ctx, sanitized_params.schema, &mut builder);
    complete_functions(&ctx, sanitized_params.schema, &mut builder);
    complete_columns(&ctx, sanitized_params.schema, &mut builder);
    complete_schemas(&ctx, sanitized_params.schema, &mut builder);
    complete_policies(&ctx, sanitized_params.schema, &mut builder);
    complete_roles(&ctx, sanitized_params.schema, &mut builder);
    complete_keywords(&ctx, &mut builder, use_upper_case);

    builder.finish()
}
