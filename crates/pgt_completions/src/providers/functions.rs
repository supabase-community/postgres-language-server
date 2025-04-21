use crate::{
    CompletionItem, CompletionItemKind, builder::CompletionBuilder, context::CompletionContext,
    relevance::CompletionRelevanceData,
};

pub fn complete_functions(ctx: &CompletionContext, builder: &mut CompletionBuilder) {
    let available_functions = &ctx.schema_cache.functions;

    for func in available_functions {
        let item = CompletionItem {
            label: func.name.clone(),
            score: CompletionRelevanceData::Function(func).get_score(ctx),
            description: format!("Schema: {}", func.schema),
            preselected: false,
            kind: CompletionItemKind::Function,
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
    async fn completes_fn() {
        let setup = r#"
          create or replace function cool()
          returns trigger
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;
        "#;

        let query = format!("select coo{}", CURSOR_POS);

        let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let results = complete(params);

        let CompletionItem { label, .. } = results
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "cool");
    }

    #[tokio::test]
    async fn prefers_fn_if_invocation() {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );

          create or replace function cool()
          returns trigger
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;
        "#;

        let query = format!(r#"select * from coo{}()"#, CURSOR_POS);

        let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let results = complete(params);

        let CompletionItem { label, kind, .. } = results
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "cool");
        assert_eq!(kind, CompletionItemKind::Function);
    }

    #[tokio::test]
    async fn prefers_fn_in_select_clause() {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );

          create or replace function cool()
          returns trigger
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;
        "#;

        let query = format!(r#"select coo{}"#, CURSOR_POS);

        let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let results = complete(params);

        let CompletionItem { label, kind, .. } = results
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "cool");
        assert_eq!(kind, CompletionItemKind::Function);
    }

    #[tokio::test]
    async fn prefers_function_in_from_clause_if_invocation() {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );

          create or replace function cool()
          returns trigger
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;
        "#;

        let query = format!(r#"select * from coo{}()"#, CURSOR_POS);

        let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let results = complete(params);

        let CompletionItem { label, kind, .. } = results
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "cool");
        assert_eq!(kind, CompletionItemKind::Function);
    }
}
