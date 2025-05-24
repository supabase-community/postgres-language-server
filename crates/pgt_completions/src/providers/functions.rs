use pgt_schema_cache::Function;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    context::CompletionContext,
    providers::helper::get_range_to_replace,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::get_completion_text_with_schema_or_alias;

pub fn complete_functions<'a>(ctx: &'a CompletionContext, builder: &mut CompletionBuilder<'a>) {
    let available_functions = &ctx.schema_cache.functions;

    for func in available_functions {
        let relevance = CompletionRelevanceData::Function(func);

        let item = PossibleCompletionItem {
            label: func.name.clone(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: format!("Schema: {}", func.schema),
            kind: CompletionItemKind::Function,
            detail: None,
            completion_text: Some(get_completion_text(ctx, func)),
        };

        builder.add_item(item);
    }
}

fn get_completion_text(ctx: &CompletionContext, func: &Function) -> CompletionText {
    let range = get_range_to_replace(ctx);
    let mut text = get_completion_text_with_schema_or_alias(ctx, &func.name, &func.schema)
        .map(|ct| ct.text)
        .unwrap_or(func.name.to_string());

    if ctx.is_invocation {
        CompletionText {
            text,
            range,
            is_snippet: false,
        }
    } else {
        text.push('(');

        let num_args = func.args.args.len();
        for (idx, arg) in func.args.args.iter().enumerate() {
            text.push_str(format!(r#"${{{}:{}}}"#, idx + 1, arg.name).as_str());
            if idx < num_args - 1 {
                text.push_str(", ");
            }
        }

        text.push(')');

        CompletionText {
            text,
            range,
            is_snippet: num_args > 0,
        }
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
