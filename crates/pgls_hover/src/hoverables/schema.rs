use std::fmt::Write;

use pgls_schema_cache::{Schema, SchemaCache};
use pgls_treesitter::TreesitterContext;

use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

impl ToHoverMarkdown for Schema {
    fn hover_headline<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<(), std::fmt::Error> {
        write!(writer, "`{}` - owned by {}", self.name, self.owner)?;

        Ok(())
    }

    fn hover_body<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        if let Some(comment) = &self.comment {
            write!(writer, "Comment: '{comment}'")?;
            writeln!(writer)?;
            writeln!(writer)?;
        }

        if !self.allowed_creators.is_empty() {
            write!(writer, "CREATE privileges:")?;
            writeln!(writer)?;

            for creator in &self.allowed_creators {
                write!(writer, "- {creator}")?;
                writeln!(writer)?;
            }

            writeln!(writer)?;
        }

        if !self.allowed_users.is_empty() {
            write!(writer, "USAGE privileges:")?;
            writeln!(writer)?;

            for user in &self.allowed_users {
                write!(writer, "- {user}")?;
                writeln!(writer)?;
            }

            writeln!(writer)?;
        }

        Ok(true)
    }

    fn hover_footer<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        writeln!(writer)?;
        write!(
            writer,
            "~{}, {} tables, {} views, {} functions",
            self.total_size, self.table_count, self.view_count, self.function_count,
        )?;
        Ok(true)
    }
}

impl ContextualPriority for Schema {
    // there are no schemas with duplicate names.
    fn relevance_score(&self, _ctx: &TreesitterContext) -> f32 {
        0.0
    }
}
