use std::fmt::Write;

use humansize::DECIMAL;
use pgls_schema_cache::{SchemaCache, Table};
use pgls_treesitter::TreesitterContext;

use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

const MAX_COLUMNS_IN_HOVER: usize = 20;

impl ToHoverMarkdown for Table {
    fn hover_headline<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<(), std::fmt::Error> {
        write!(writer, "`{}.{}`", self.schema, self.name)?;

        let table_kind = match self.table_kind {
            pgls_schema_cache::TableKind::View => " (View)",
            pgls_schema_cache::TableKind::MaterializedView => " (M.View)",
            pgls_schema_cache::TableKind::Partitioned => " (Partitioned)",
            pgls_schema_cache::TableKind::Ordinary => "",
        };

        write!(writer, "{table_kind}")?;

        let locked_txt = if self.rls_enabled {
            " - 🔒 RLS enabled"
        } else {
            " - 🔓 RLS disabled"
        };

        write!(writer, "{locked_txt}")?;

        Ok(())
    }

    fn hover_body<W: Write>(
        &self,
        writer: &mut W,
        schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        if let Some(comment) = &self.comment {
            write!(writer, "Comment: '{comment}'")?;
            writeln!(writer)?;
        }

        let mut columns: Vec<_> = schema_cache
            .columns
            .iter()
            .filter(|column| column.schema_name == self.schema && column.table_name == self.name)
            .collect();
        columns.sort_by_key(|column| column.number);

        writeln!(writer, "Columns:")?;

        for column in columns.iter().take(MAX_COLUMNS_IN_HOVER) {
            write!(writer, "- {}: ", column.name)?;

            if let Some(type_name) = &column.type_name {
                write!(writer, "{type_name}")?;

                if let Some(varchar_length) = column.varchar_length {
                    write!(writer, "({varchar_length})")?;
                }
            } else {
                write!(writer, "typeid:{}", column.type_id)?;
            }

            writeln!(writer)?;
        }

        if columns.len() > MAX_COLUMNS_IN_HOVER {
            writeln!(
                writer,
                "... +{} more columns",
                columns.len() - MAX_COLUMNS_IN_HOVER
            )?;
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
            "~{} rows, ~{} dead rows, {}",
            self.live_rows_estimate,
            self.dead_rows_estimate,
            humansize::format_size(self.bytes as u64, DECIMAL)
        )?;
        Ok(true)
    }
}

impl ContextualPriority for Table {
    fn relevance_score(&self, ctx: &TreesitterContext) -> f32 {
        let mut score = 0.0;

        if ctx
            .get_mentioned_relations(&Some(self.schema.clone()))
            .is_some_and(|t| t.contains(&self.name))
        {
            score += 200.0;
        } else if ctx
            .get_mentioned_relations(&None)
            .is_some_and(|t| t.contains(&self.name))
        {
            score += 150.0;
        } else if ctx
            .get_mentioned_relations(&Some(self.schema.clone()))
            .is_some()
        {
            score += 50.0;
        }

        if self.schema == "public" && score == 0.0 {
            score += 10.0;
        }

        score
    }
}
