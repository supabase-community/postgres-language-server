use crate::{
    CompletionItem, builder::CompletionBuilder, context::CompletionContext,
    relevance::CompletionRelevanceData,
};

pub fn complete_schemas(ctx: &CompletionContext, builder: &mut CompletionBuilder) {
    let available_schemas = &ctx.schema_cache.schemas;

    for schema in available_schemas {
        let relevance = CompletionRelevanceData::Schema(&schema);

        let item = CompletionItem {
            label: schema.name.clone(),
            description: "Schema".into(),
            preselected: false,
            kind: crate::CompletionItemKind::Schema,
            score: relevance.get_score(ctx),
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        CompletionItemKind, complete,
        test_helper::{CURSOR_POS, get_test_deps, get_test_params},
    };

    #[tokio::test]
    async fn autocompletes_schemas() {
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

        let query = format!("select * from {}", CURSOR_POS);

        let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let items = complete(params);

        assert!(!items.is_empty());

        assert_eq!(
            items
                .into_iter()
                .take(5)
                .map(|i| (i.label, i.kind))
                .collect::<Vec<(String, CompletionItemKind)>>(),
            vec![
                ("public".to_string(), CompletionItemKind::Schema),
                ("auth".to_string(), CompletionItemKind::Schema),
                ("internal".to_string(), CompletionItemKind::Schema),
                ("private".to_string(), CompletionItemKind::Schema),
                ("users".to_string(), CompletionItemKind::Table),
            ]
        );
    }
}
