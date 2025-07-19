use std::fmt::Write;

use humansize::DECIMAL;

pub(crate) trait ToHoverMarkdown {
    fn to_hover_markdown<W: Write>(&self, writer: &mut W) -> Result<(), std::fmt::Error>;
}

impl ToHoverMarkdown for pgt_schema_cache::Table {
    fn to_hover_markdown<W: Write>(&self, writer: &mut W) -> Result<(), std::fmt::Error> {
        HeadlineWriter::for_table(writer, &self)?;
        BodyWriter::for_table(writer, &self)?;
        FooterWriter::for_table(writer, &self)?;

        Ok(())
    }
}

struct HeadlineWriter;

impl HeadlineWriter {
    fn for_table<W: Write>(
        writer: &mut W,
        table: &pgt_schema_cache::Table,
    ) -> Result<(), std::fmt::Error> {
        let table_kind = match table.table_kind {
            pgt_schema_cache::TableKind::View => " (View)",
            pgt_schema_cache::TableKind::MaterializedView => " (M.View)",
            pgt_schema_cache::TableKind::Partitioned => " (Partitioned)",
            pgt_schema_cache::TableKind::Ordinary => "",
        };

        let locked_txt = if table.rls_enabled {
            " - 🔒 RLS enabled"
        } else {
            " - 🔓 RLS disabled"
        };

        write!(
            writer,
            "### {}.{}{}{}",
            table.schema, table.name, table_kind, locked_txt
        )?;

        writeln!(writer)?;

        Ok(())
    }
}

struct BodyWriter;

impl BodyWriter {
    fn for_table<W: Write>(
        writer: &mut W,
        table: &pgt_schema_cache::Table,
    ) -> Result<(), std::fmt::Error> {
        if let Some(c) = table.comment.as_ref() {
            write!(writer, "{}", c)?;
            writeln!(writer)?;
        }

        Ok(())
    }
}

struct FooterWriter;

impl FooterWriter {
    fn for_table<W: Write>(
        writer: &mut W,
        table: &pgt_schema_cache::Table,
    ) -> Result<(), std::fmt::Error> {
        write!(
            writer,
            "~{} rows, ~{} dead rows, {}",
            table.live_rows_estimate,
            table.dead_rows_estimate,
            humansize::format_size(table.bytes as u64, DECIMAL)
        )?;

        Ok(())
    }
}
