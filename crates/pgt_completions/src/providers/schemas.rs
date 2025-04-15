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
        CompletionItem, CompletionItemKind, complete,
        test_helper::{CURSOR_POS, get_test_deps, get_test_params},
    };

    #[tokio::test]
    async fn autocompletes_schemas() {
        let setup = r#"
            create schema private;
            create schema auth;
            create schema internal;
        "#;

        let query = format!("select * from {}", CURSOR_POS);

        let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let items = complete(params);

        assert!(!items.is_empty());

        for item in items.iter().take(10) {
            println!(
                r#""{}", score: {}, kind: {:?}"#,
                item.label, item.score, item.kind
            );
        }

        assert_eq!(
            items
                .into_iter()
                .take(4)
                .map(|i| i.label)
                .collect::<Vec<String>>(),
            vec![
                "public".to_string(), // public always preferred
                "auth".to_string(),
                "internal".to_string(),
                "private".to_string()
            ]
        );
    }
}
