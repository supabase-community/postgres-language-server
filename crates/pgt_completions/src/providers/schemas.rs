use crate::{
    builder::{CompletionBuilder, PossibleCompletionItem},
    context::CompletionContext,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

pub fn complete_schemas<'a>(ctx: &'a CompletionContext, builder: &mut CompletionBuilder<'a>) {
    let available_schemas = &ctx.schema_cache.schemas;

    for schema in available_schemas {
        let relevance = CompletionRelevanceData::Schema(schema);

        let item = PossibleCompletionItem {
            label: schema.name.clone(),
            description: "Schema".into(),
            kind: crate::CompletionItemKind::Schema,
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            detail: None,
            completion_text: None,
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use crate::{
        CompletionItemKind,
        test_helper::{CURSOR_POS, CompletionAssertion, assert_complete_results},
    };

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
    async fn autocompletes_schemas(pool: PgPool) {
        let setup = r#"
            create schema private;
            create schema auth;
            create schema internal;

            -- add a table to compete against schemas
            create table users (
                id serial primary key,
                name text,
                password text
            );
        "#;

        assert_complete_results(
            format!("select * from {}", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".to_string(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("auth".to_string(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind(
                    "internal".to_string(),
                    CompletionItemKind::Schema,
                ),
                CompletionAssertion::LabelAndKind(
                    "private".to_string(),
                    CompletionItemKind::Schema,
                ),
                // users table still preferred over system schemas
                CompletionAssertion::LabelAndKind("users".to_string(), CompletionItemKind::Table),
                CompletionAssertion::LabelAndKind(
                    "information_schema".to_string(),
                    CompletionItemKind::Schema,
                ),
                CompletionAssertion::LabelAndKind(
                    "pg_catalog".to_string(),
                    CompletionItemKind::Schema,
                ),
                CompletionAssertion::LabelAndKind(
                    "pg_toast".to_string(),
                    CompletionItemKind::Schema,
                ),
            ],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
    async fn suggests_tables_and_schemas_with_matching_keys(pool: PgPool) {
        let setup = r#"
            create schema ultimate;

            -- add a table to compete against schemas
            create table users (
                id serial primary key,
                name text,
                password text
            );
        "#;

        assert_complete_results(
            format!("select * from u{}", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("users".into(), CompletionItemKind::Table),
                CompletionAssertion::LabelAndKind("ultimate".into(), CompletionItemKind::Schema),
            ],
            Some(setup),
            &pool,
        )
        .await;
    }
}
