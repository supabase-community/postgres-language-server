use pgls_resolve::FunctionContext;
use pgls_tokenizer::NamePosition;
use pgls_treesitter::TreesitterContext;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    providers::helper::get_range_to_replace,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::with_schema_or_alias;

/// The named parameters of the SQL function whose body is completed. Positional references
/// like `$1` have no name to complete.
pub(crate) fn complete_parameters<'a>(
    ctx: &TreesitterContext<'a>,
    function: Option<&'a FunctionContext>,
    builder: &mut CompletionBuilder<'a>,
) {
    let Some(function) = function else {
        return;
    };

    for param in &function.params {
        let Some(name) = param.name.as_deref() else {
            continue;
        };
        let relevance = CompletionRelevanceData::Parameter { function, param };

        builder.add_item(PossibleCompletionItem {
            label: name.to_owned(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: function.function_name.clone(),
            kind: CompletionItemKind::Parameter,
            completion_text: Some(CompletionText {
                is_snippet: false,
                range: get_range_to_replace(ctx),
                text: with_schema_or_alias(ctx, name, None, NamePosition::ColId),
            }),
            detail: Some(param.type_label()),
        });
    }
}

#[cfg(test)]
mod tests {
    use pgls_resolve::{FunctionContext, FunctionParam};
    use pgls_test_utils::QueryWithCursorPosition;
    use sqlx::{Executor, PgPool};

    use crate::{
        CompletionItemKind, complete,
        test_helper::{get_test_deps, get_test_params},
    };

    /// The body of `fn(user_id int4, names text[], int4)` in a database with a table `users`.
    struct Body<'a> {
        pool: &'a PgPool,
        function: FunctionContext,
    }

    impl<'a> Body<'a> {
        async fn new(pool: &'a PgPool) -> Self {
            pool.execute("create table users (id int4 primary key, name text);")
                .await
                .expect("setup");
            let param = |name: Option<&str>, type_name: &str, is_array| FunctionParam {
                name: name.map(str::to_owned),
                type_schema: Some("pg_catalog".to_owned()),
                type_name: type_name.to_owned(),
                is_array,
            };
            let function = FunctionContext {
                function_name: "fn".to_owned(),
                params: vec![
                    param(Some("user_id"), "int4", false),
                    param(Some("names"), "text", true),
                    param(None, "int4", false),
                ],
            };
            Self { pool, function }
        }

        /// The parameters offered at the cursor of `sql`, as `label: detail => text`.
        async fn parameters_at(&self, sql: &str) -> Vec<String> {
            let (tree, snapshot) = get_test_deps(None, sql.into(), self.pool).await;
            let mut params = get_test_params(&tree, &snapshot, sql.into());
            params.function = Some(&self.function);
            complete(params)
                .into_iter()
                .filter(|item| item.kind == CompletionItemKind::Parameter)
                .map(|item| {
                    format!(
                        "{}: {} => {}",
                        item.label,
                        item.detail.unwrap_or_default(),
                        item.completion_text.map(|t| t.text).unwrap_or_default()
                    )
                })
                .collect()
        }
    }

    #[pgls_test_utils::db_test]
    async fn completes_parameters_where_a_value_is_expected(pool: PgPool) {
        let body = Body::new(&pool).await;
        let cursor = QueryWithCursorPosition::cursor_marker();

        assert_eq!(
            body.parameters_at(&format!("select * from users where id = u{cursor}"))
                .await,
            ["user_id: int4 => user_id"]
        );
        assert_eq!(
            body.parameters_at(&format!("select na{cursor} from users"))
                .await,
            ["names: text[] => names"]
        );
        assert_eq!(
            body.parameters_at(&format!("select fn.{cursor}")).await,
            ["names: text[] => names", "user_id: int4 => user_id"]
        );
        assert_eq!(
            body.parameters_at(&format!(
                "update users set name = names[1] where id = {cursor}"
            ))
            .await,
            ["names: text[] => names", "user_id: int4 => user_id"]
        );
    }

    #[pgls_test_utils::db_test]
    async fn no_parameters_where_a_name_is_expected(pool: PgPool) {
        let body = Body::new(&pool).await;
        let cursor = QueryWithCursorPosition::cursor_marker();

        for sql in [
            format!("select * from u{cursor}"),
            format!("select users.{cursor} from users"),
            format!("update users set {cursor}"),
            format!("insert into users ({cursor}"),
            format!("select 1::u{cursor}"),
        ] {
            assert_eq!(
                body.parameters_at(&sql).await,
                Vec::<String>::new(),
                "{sql}"
            );
        }
    }
}
