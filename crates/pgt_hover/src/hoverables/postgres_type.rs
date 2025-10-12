use std::fmt::Write;

use pgt_schema_cache::PostgresType;
use pgt_treesitter::TreesitterContext;

use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

impl ToHoverMarkdown for PostgresType {
    fn hover_headline<W: Write>(&self, writer: &mut W) -> Result<(), std::fmt::Error> {
        write!(writer, "`{}.{}`", self.schema, self.name)?;
        Ok(())
    }

    fn hover_body<W: Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        if let Some(comment) = &self.comment {
            write!(writer, "Comment: '{}'", comment)?;
            writeln!(writer)?;
            writeln!(writer)?;
        }

        if !self.attributes.attrs.is_empty() {
            write!(writer, "Attributes:")?;
            writeln!(writer)?;

            for attribute in &self.attributes.attrs {
                // TODO: look up other types with schema cache.
                write!(writer, "- {}", attribute.name)?;
                writeln!(writer)?;
            }

            writeln!(writer)?;
        }

        if !self.enums.values.is_empty() {
            write!(writer, "Enum Permutations:")?;
            writeln!(writer)?;

            for kind in &self.enums.values {
                // TODO: look up other types with schema cache.
                write!(writer, "- {}", kind)?;
                writeln!(writer)?;
            }

            writeln!(writer)?;
        }

        Ok(true)
    }

    fn hover_footer<W: Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        writeln!(writer)?;
        Ok(true)
    }
}

impl ContextualPriority for PostgresType {
    // there are no schemas with duplicate names.
    fn relevance_score(&self, ctx: &TreesitterContext) -> f32 {
        let mut score = 0.0;

        if ctx
            .get_mentioned_relations(&Some(self.schema.clone()))
            .is_some()
        {
            score += 100.0;
        }

        if ctx.get_mentioned_relations(&None).is_some() && self.schema == "public" {
            score += 100.0;
        }

        if self.schema == "public" && score == 0.0 {
            score += 10.0;
        }

        score
    }
}
