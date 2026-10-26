use std::fmt::Write;

use pgls_catalog::Snapshot;
use pgls_catalog::resolve::ColumnSource;
use pgls_treesitter::TreesitterContext;

use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

/// A column of a CTE or a subquery in FROM, which isn't in the snapshot.
#[derive(Debug)]
pub struct DerivedColumn {
    pub source: ColumnSource,
    pub name: String,
    pub type_name: Option<String>,
}

impl ToHoverMarkdown for DerivedColumn {
    fn hover_headline<W: Write>(
        &self,
        writer: &mut W,
        _snapshot: &Snapshot,
    ) -> Result<(), std::fmt::Error> {
        match &self.source {
            ColumnSource::Cte { name } | ColumnSource::Relation { name, .. } => {
                write!(writer, "`{name}.{}`", self.name)
            }
            ColumnSource::Subquery { alias: Some(alias) } => {
                write!(writer, "`{alias}.{}`", self.name)
            }
            ColumnSource::Subquery { alias: None } => write!(writer, "`{}`", self.name),
        }
    }

    fn hover_body<W: Write>(
        &self,
        writer: &mut W,
        _snapshot: &Snapshot,
    ) -> Result<bool, std::fmt::Error> {
        write!(
            writer,
            "{}",
            self.type_name.as_deref().unwrap_or("unknown type")
        )?;
        match &self.source {
            ColumnSource::Cte { .. } => write!(writer, " - CTE column")?,
            ColumnSource::Subquery { .. } => write!(writer, " - subquery column")?,
            ColumnSource::Relation { .. } => {}
        }
        writeln!(writer)?;
        Ok(true)
    }

    fn hover_footer<W: Write>(
        &self,
        _writer: &mut W,
        _snapshot: &Snapshot,
    ) -> Result<bool, std::fmt::Error> {
        Ok(false)
    }
}

impl ContextualPriority for DerivedColumn {
    fn relevance_score(&self, _ctx: &TreesitterContext) -> f32 {
        0.0
    }
}
