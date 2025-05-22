use crate::context::{CompletionContext, NodeUnderCursor, WrappingClause};

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
        self.check_mentioned_schema_or_alias(ctx)?;

        Some(())
    }

    fn completable_context(&self, ctx: &CompletionContext) -> Option<()> {
        let current_node_kind = ctx
            .node_under_cursor
            .as_ref()
            .map(|n| n.kind())
            .unwrap_or("");

        if current_node_kind.starts_with("keyword_")
            || current_node_kind == "="
            || current_node_kind == ","
            || current_node_kind == "literal"
            || current_node_kind == "ERROR"
        {
            return None;
        }

        // No autocompletions if there are two identifiers without a separator.
        if ctx.node_under_cursor.as_ref().is_some_and(|n| match n {
            NodeUnderCursor::TsNode(node) => node.prev_sibling().is_some_and(|p| {
                (p.kind() == "identifier" || p.kind() == "object_reference")
                    && n.kind() == "identifier"
            }),
            NodeUnderCursor::CustomNode { .. } => false,
        }) {
            return None;
        }

        // no completions if we're right after an asterisk:
        // `select * {}`
        if ctx.node_under_cursor.as_ref().is_some_and(|n| match n {
            NodeUnderCursor::TsNode(node) => node
                .prev_sibling()
                .is_some_and(|p| (p.kind() == "all_fields") && n.kind() == "identifier"),
            NodeUnderCursor::CustomNode { .. } => false,
        }) {
            return None;
        }

        Some(())
    }

    fn check_clause(&self, ctx: &CompletionContext) -> Option<()> {
        let clause = ctx.wrapping_clause_type.as_ref();

        let in_clause = |compare: WrappingClause| clause.is_some_and(|c| c == &compare);

        match self.data {
            CompletionRelevanceData::Table(_) => {
                if in_clause(WrappingClause::Select)
                    || in_clause(WrappingClause::Where)
                    || in_clause(WrappingClause::PolicyName)
                {
                    return None;
                };
            }
            CompletionRelevanceData::Column(_) => {
                if in_clause(WrappingClause::From) || in_clause(WrappingClause::PolicyName) {
                    return None;
                }

                // We can complete columns in JOIN cluases, but only if we are after the
                // ON node in the "ON u.id = posts.user_id" part.
                let in_join_clause_before_on_node = clause.is_some_and(|c| match c {
                    // we are in a JOIN, but definitely not after an ON
                    WrappingClause::Join { on_node: None } => true,

                    WrappingClause::Join { on_node: Some(on) } => ctx
                        .node_under_cursor
                        .as_ref()
                        .is_some_and(|n| n.end_byte() < on.start_byte()),

                    _ => false,
                });

                if in_join_clause_before_on_node {
                    return None;
                }
            }
            CompletionRelevanceData::Policy(_) => {
                if clause.is_none_or(|c| c != &WrappingClause::PolicyName) {
                    return None;
                }
            }
            _ => {
                if in_clause(WrappingClause::PolicyName) {
                    return None;
                }
            }
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

    fn check_mentioned_schema_or_alias(&self, ctx: &CompletionContext) -> Option<()> {
        if ctx.schema_or_alias_name.is_none() {
            return Some(());
        }

        let schema_or_alias = ctx.schema_or_alias_name.as_ref().unwrap();

        let matches = match self.data {
            CompletionRelevanceData::Table(table) => &table.schema == schema_or_alias,
            CompletionRelevanceData::Function(f) => &f.schema == schema_or_alias,
            CompletionRelevanceData::Column(col) => ctx
                .mentioned_table_aliases
                .get(schema_or_alias)
                .is_some_and(|t| t == &col.table_name),

            // we should never allow schema suggestions if there already was one.
            CompletionRelevanceData::Schema(_) => false,
            // no policy comletion if user typed a schema node first.
            CompletionRelevanceData::Policy(_) => false,
        };

        if !matches {
            return None;
        }

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::{
        CURSOR_POS, CompletionAssertion, assert_complete_results, assert_no_complete_results,
    };

    #[tokio::test]
    async fn completion_after_asterisk() {
        let setup = r#"
            create table users (
                id serial primary key,
                email text,
                address text
            );
        "#;

        assert_no_complete_results(format!("select * {}", CURSOR_POS).as_str(), setup).await;

        // if there s a COMMA after the asterisk, we're good
        assert_complete_results(
            format!("select *, {}", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::Label("address".into()),
                CompletionAssertion::Label("email".into()),
                CompletionAssertion::Label("id".into()),
            ],
            setup,
        )
        .await;
    }
}
