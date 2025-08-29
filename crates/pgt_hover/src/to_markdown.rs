use std::fmt::Write;

use humansize::DECIMAL;

pub(crate) trait ToHoverMarkdown {
    fn hover_headline<W: Write>(&self, writer: &mut W) -> Result<(), std::fmt::Error>;
    fn hover_body<W: Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error>; // returns true if something was written
    fn hover_footer<W: Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error>; // returns true if something was written
}

pub(crate) fn format_hover_markdown<T: ToHoverMarkdown>(
    item: &T,
) -> Result<String, std::fmt::Error> {
    let mut markdown = String::new();

    write!(markdown, "### ")?;
    item.hover_headline(&mut markdown)?;
    markdown_newline(&mut markdown)?;

    if item.hover_body(&mut markdown)? {
        markdown_newline(&mut markdown)?;
    }

    item.hover_footer(&mut markdown)?;

    Ok(markdown)
}

impl ToHoverMarkdown for pgt_schema_cache::Table {
    fn hover_headline<W: Write>(&self, writer: &mut W) -> Result<(), std::fmt::Error> {
        let table_kind = match self.table_kind {
            pgt_schema_cache::TableKind::View => " (View)",
            pgt_schema_cache::TableKind::MaterializedView => " (M.View)",
            pgt_schema_cache::TableKind::Partitioned => " (Partitioned)",
            pgt_schema_cache::TableKind::Ordinary => "",
        };

        let locked_txt = if self.rls_enabled {
            " - 🔒 RLS enabled"
        } else {
            " - 🔓 RLS disabled"
        };

        write!(
            writer,
            "{}.{}{}{}",
            self.schema, self.name, table_kind, locked_txt
        )
    }

    fn hover_body<W: Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        if let Some(comment) = &self.comment {
            write!(writer, "{}", comment)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn hover_footer<W: Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
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

impl ToHoverMarkdown for pgt_schema_cache::Column {
    fn hover_headline<W: Write>(&self, writer: &mut W) -> Result<(), std::fmt::Error> {
        write!(
            writer,
            "{}.{}.{}",
            self.schema_name, self.table_name, self.name
        )
    }

    fn hover_body<W: Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        if let Some(tname) = self.type_name.as_ref() {
            write!(writer, "{}", tname)?;
            if let Some(l) = self.varchar_length {
                write!(writer, "({})", l)?;
            }
        } else {
            write!(writer, "typeid: {}", self.type_id)?;
        }

        if self.is_primary_key {
            write!(writer, " - 🔑 primary key")?;
        } else if self.is_unique {
            write!(writer, " - unique")?;
        }

        if self.is_nullable {
            write!(writer, " - nullable")?;
        } else {
            write!(writer, " - not-nullable")?;
        }

        if let Some(comment) = &self.comment {
            write!(writer, "  \n{}", comment)?;
        }

        Ok(true)
    }

    fn hover_footer<W: Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        if let Some(default) = &self.default_expr {
            write!(writer, "Default: `{}`", default)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn markdown_newline<W: Write>(writer: &mut W) -> Result<(), std::fmt::Error> {
    write!(writer, "  ")?;
    writeln!(writer)?;
    Ok(())
}
