use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

mod column;
mod function;
mod role;
mod schema;
mod table;

mod test_helper;

/// Mapper type that will be used for filtering and turning data to markdown.
#[derive(Debug)]
pub enum Hoverable<'a> {
    Table(&'a pgt_schema_cache::Table),
    Column(&'a pgt_schema_cache::Column),
    Function(&'a pgt_schema_cache::Function),
    Role(&'a pgt_schema_cache::Role),
    Schema(&'a pgt_schema_cache::Schema),
}

impl<'a> From<&'a pgt_schema_cache::Schema> for Hoverable<'a> {
    fn from(value: &'a pgt_schema_cache::Schema) -> Self {
        Hoverable::Schema(value)
    }
}

impl<'a> From<&'a pgt_schema_cache::Table> for Hoverable<'a> {
    fn from(value: &'a pgt_schema_cache::Table) -> Self {
        Hoverable::Table(value)
    }
}

impl<'a> From<&'a pgt_schema_cache::Column> for Hoverable<'a> {
    fn from(value: &'a pgt_schema_cache::Column) -> Self {
        Hoverable::Column(value)
    }
}

impl<'a> From<&'a pgt_schema_cache::Function> for Hoverable<'a> {
    fn from(value: &'a pgt_schema_cache::Function) -> Self {
        Hoverable::Function(value)
    }
}

impl<'a> From<&'a pgt_schema_cache::Role> for Hoverable<'a> {
    fn from(value: &'a pgt_schema_cache::Role) -> Self {
        Hoverable::Role(value)
    }
}

impl ContextualPriority for Hoverable<'_> {
    fn relevance_score(&self, ctx: &pgt_treesitter::TreesitterContext) -> f32 {
        match self {
            Hoverable::Table(table) => table.relevance_score(ctx),
            Hoverable::Column(column) => column.relevance_score(ctx),
            Hoverable::Function(function) => function.relevance_score(ctx),
            Hoverable::Role(role) => role.relevance_score(ctx),
            Hoverable::Schema(schema) => schema.relevance_score(ctx),
        }
    }
}

impl ToHoverMarkdown for Hoverable<'_> {
    fn hover_headline<W: std::fmt::Write>(&self, writer: &mut W) -> Result<(), std::fmt::Error> {
        match self {
            Hoverable::Table(table) => ToHoverMarkdown::hover_headline(*table, writer),
            Hoverable::Column(column) => ToHoverMarkdown::hover_headline(*column, writer),
            Hoverable::Function(function) => ToHoverMarkdown::hover_headline(*function, writer),
            Hoverable::Role(role) => ToHoverMarkdown::hover_headline(*role, writer),
            Hoverable::Schema(schema) => ToHoverMarkdown::hover_headline(*schema, writer),
        }
    }

    fn hover_body<W: std::fmt::Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        match self {
            Hoverable::Table(table) => ToHoverMarkdown::hover_body(*table, writer),
            Hoverable::Column(column) => ToHoverMarkdown::hover_body(*column, writer),
            Hoverable::Function(function) => ToHoverMarkdown::hover_body(*function, writer),
            Hoverable::Role(role) => ToHoverMarkdown::hover_body(*role, writer),
            Hoverable::Schema(schema) => ToHoverMarkdown::hover_body(*schema, writer),
        }
    }

    fn hover_footer<W: std::fmt::Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        match self {
            Hoverable::Table(table) => ToHoverMarkdown::hover_footer(*table, writer),
            Hoverable::Column(column) => ToHoverMarkdown::hover_footer(*column, writer),
            Hoverable::Function(function) => ToHoverMarkdown::hover_footer(*function, writer),
            Hoverable::Role(role) => ToHoverMarkdown::hover_footer(*role, writer),
            Hoverable::Schema(schema) => ToHoverMarkdown::hover_footer(*schema, writer),
        }
    }

    fn body_markdown_type(&self) -> &'static str {
        match self {
            Hoverable::Table(table) => table.body_markdown_type(),
            Hoverable::Column(column) => column.body_markdown_type(),
            Hoverable::Function(function) => function.body_markdown_type(),
            Hoverable::Role(role) => role.body_markdown_type(),
            Hoverable::Schema(schema) => schema.body_markdown_type(),
        }
    }

    fn footer_markdown_type(&self) -> &'static str {
        match self {
            Hoverable::Table(table) => table.footer_markdown_type(),
            Hoverable::Column(column) => column.footer_markdown_type(),
            Hoverable::Function(function) => function.footer_markdown_type(),
            Hoverable::Role(role) => role.footer_markdown_type(),
            Hoverable::Schema(schema) => schema.footer_markdown_type(),
        }
    }
}
