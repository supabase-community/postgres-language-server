use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

/// Mapper type that will be used for filtering and turning data to markdown.
#[derive(Debug)]
pub(crate) enum HoverItem<'a> {
    Table(&'a pgt_schema_cache::Table),
    Column(&'a pgt_schema_cache::Column),
}

impl<'a> From<&'a pgt_schema_cache::Table> for HoverItem<'a> {
    fn from(value: &'a pgt_schema_cache::Table) -> Self {
        HoverItem::Table(value)
    }
}

impl<'a> From<&'a pgt_schema_cache::Column> for HoverItem<'a> {
    fn from(value: &'a pgt_schema_cache::Column) -> Self {
        HoverItem::Column(value)
    }
}

impl ContextualPriority for HoverItem<'_> {
    fn relevance_score(&self, ctx: &pgt_treesitter::TreesitterContext) -> f32 {
        match self {
            HoverItem::Table(table) => table.relevance_score(ctx),
            HoverItem::Column(column) => column.relevance_score(ctx),
        }
    }
}

impl ToHoverMarkdown for HoverItem<'_> {
    fn hover_headline<W: std::fmt::Write>(&self, writer: &mut W) -> Result<(), std::fmt::Error> {
        match self {
            HoverItem::Table(table) => ToHoverMarkdown::hover_headline(*table, writer),
            HoverItem::Column(column) => ToHoverMarkdown::hover_headline(*column, writer),
        }
    }

    fn hover_body<W: std::fmt::Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        match self {
            HoverItem::Table(table) => ToHoverMarkdown::hover_body(*table, writer),
            HoverItem::Column(column) => ToHoverMarkdown::hover_body(*column, writer),
        }
    }

    fn hover_footer<W: std::fmt::Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        match self {
            HoverItem::Table(table) => ToHoverMarkdown::hover_footer(*table, writer),
            HoverItem::Column(column) => ToHoverMarkdown::hover_footer(*column, writer),
        }
    }
}
