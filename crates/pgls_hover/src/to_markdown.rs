use std::fmt::Write;

use pgls_schema_cache::SchemaCache;

pub(crate) trait ToHoverMarkdown {
    fn body_markdown_type(&self) -> &'static str {
        "plain"
    }

    fn footer_markdown_type(&self) -> &'static str {
        "plain"
    }

    fn hover_headline<W: Write>(
        &self,
        writer: &mut W,
        schema_cache: &SchemaCache,
    ) -> Result<(), std::fmt::Error>;

    fn hover_body<W: Write>(
        &self,
        writer: &mut W,
        schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error>; // returns true if something was written

    fn hover_footer<W: Write>(
        &self,
        writer: &mut W,
        schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error>; // returns true if something was written
}

pub(crate) fn format_hover_markdown<T: ToHoverMarkdown>(
    item: &T,
    schema_cache: &SchemaCache,
) -> Result<String, std::fmt::Error> {
    let mut markdown = String::new();

    write!(markdown, "### ")?;
    item.hover_headline(&mut markdown, schema_cache)?;
    markdown_newline(&mut markdown)?;

    write!(markdown, "```{}", item.body_markdown_type())?;
    markdown_newline(&mut markdown)?;
    item.hover_body(&mut markdown, schema_cache)?;
    markdown_newline(&mut markdown)?;
    write!(markdown, "```")?;

    markdown_newline(&mut markdown)?;
    write!(markdown, "---  ")?;
    markdown_newline(&mut markdown)?;

    write!(markdown, "```{}", item.footer_markdown_type())?;
    markdown_newline(&mut markdown)?;
    item.hover_footer(&mut markdown, schema_cache)?;
    markdown_newline(&mut markdown)?;
    write!(markdown, "```")?;

    Ok(markdown)
}

pub(crate) fn markdown_newline<W: Write>(writer: &mut W) -> Result<(), std::fmt::Error> {
    write!(writer, "  ")?;
    writeln!(writer)?;
    Ok(())
}
