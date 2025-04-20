use crate::context::{ClauseType, CompletionContext};

use super::CompletionRelevanceData;

#[derive(Debug)]
pub(crate) struct CompletionFilter<'a> {
    data: CompletionRelevanceData<'a>,
}

impl<'a> From<CompletionRelevanceData<'a>> for CompletionFilter<'a> {
    fn from(value: CompletionRelevanceData<'a>) -> Self {
        Self { data: value }
    }
}

impl CompletionFilter<'_> {
    pub fn is_relevant(&self, ctx: &CompletionContext) -> Option<()> {
        self.completable_context(ctx)?;
        self.check_clause(ctx)?;
        self.check_invocation(ctx)?;
        self.check_mentioned_schema(ctx)?;

        Some(())
    }

    fn completable_context(&self, ctx: &CompletionContext) -> Option<()> {
        let current_node_kind = ctx.node_under_cursor.map(|n| n.kind()).unwrap_or("");

        if current_node_kind.starts_with("keyword_")
            || current_node_kind == "="
            || current_node_kind == ","
            || current_node_kind == "literal"
        {
            return None;
        }

        Some(())
    }

    fn check_clause(&self, ctx: &CompletionContext) -> Option<()> {
        let clause = ctx.wrapping_clause_type.as_ref();

        match self.data {
            CompletionRelevanceData::Table(_) => {
                let in_select_clause = clause.is_some_and(|c| c == &ClauseType::Select);
                let in_where_clause = clause.is_some_and(|c| c == &ClauseType::Where);

                if in_select_clause || in_where_clause {
                    return None;
                };
            }
            CompletionRelevanceData::Column(_) => {
                let in_from_clause = clause.is_some_and(|c| c == &ClauseType::From);

                if in_from_clause {
                    return None;
                }
            }
            _ => {}
        }

        Some(())
    }

    fn check_invocation(&self, ctx: &CompletionContext) -> Option<()> {
        if !ctx.is_invocation {
            return Some(());
        }

        match self.data {
            CompletionRelevanceData::Table(_) | CompletionRelevanceData::Column(_) => return None,
            _ => {}
        }

        Some(())
    }

    fn check_mentioned_schema(&self, ctx: &CompletionContext) -> Option<()> {
        if ctx.schema_name.is_none() {
            return Some(());
        }

        let name = ctx.schema_name.as_ref().unwrap();

        let does_not_match = match self.data {
            CompletionRelevanceData::Table(table) => &table.schema != name,
            CompletionRelevanceData::Function(foo) => &foo.schema != name,
            CompletionRelevanceData::Column(_) => {
                // columns belong to tables, not schemas
                true
            }
            CompletionRelevanceData::Schema(_) => {
                // we should never allow schema suggestions if there already was one.
                true
            }
        };

        if does_not_match {
            return None;
        }

        Some(())
    }
}
