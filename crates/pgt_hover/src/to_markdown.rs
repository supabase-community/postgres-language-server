use std::fmt::Write;

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

    write!(markdown, "#### ")?;
    item.hover_body(&mut markdown)?;
    markdown_newline(&mut markdown)?;

    write!(markdown, "---  ")?;
    markdown_newline(&mut markdown)?;
    item.hover_footer(&mut markdown)?;

    Ok(markdown)
}

pub(crate) fn markdown_newline<W: Write>(writer: &mut W) -> Result<(), std::fmt::Error> {
    write!(writer, "  ")?;
    writeln!(writer)?;
    Ok(())
}
