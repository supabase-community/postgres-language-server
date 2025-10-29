use pgls_schema_cache::{Function, SchemaCache};
use pgls_treesitter::TreesitterContext;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    providers::helper::{get_range_to_replace, node_text_surrounded_by_quotes, only_leading_quote},
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::with_schema_or_alias;

pub fn complete_functions<'a>(
    ctx: &'a TreesitterContext,
    schema_cache: &'a SchemaCache,
    builder: &mut CompletionBuilder<'a>,
) {
    let available_functions = &schema_cache.functions;

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

fn get_completion_text(ctx: &TreesitterContext, func: &Function) -> CompletionText {
    let mut text = with_schema_or_alias(ctx, func.name.as_str(), Some(func.schema.as_str()));

    let mut range = get_range_to_replace(ctx);

    if ctx.is_invocation {
        CompletionText {
            text,
            range,
            is_snippet: false,
        }
    } else {
        if node_text_surrounded_by_quotes(ctx) && !only_leading_quote(ctx) {
            text.push('"');
            range = range.checked_expand_end(1.into()).unwrap_or(range);
        }

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
    use pgls_text_size::TextRange;
    use sqlx::{Executor, PgPool};

    use crate::{
        CompletionItem, CompletionItemKind, complete,
        test_helper::{
            CompletionAssertion, assert_complete_results, get_test_deps, get_test_params,
        },
    };

    use pgls_test_utils::QueryWithCursorPosition;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_fn(pool: PgPool) {
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

        let query = format!("select coo{}", QueryWithCursorPosition::cursor_marker());

        let (tree, cache) = get_test_deps(Some(setup), query.as_str().into(), &pool).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let results = complete(params);

        let CompletionItem { label, .. } = results
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "cool");
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn prefers_fn_if_invocation(pool: PgPool) {
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

        let query = format!(
            r#"select * from coo{}()"#,
            QueryWithCursorPosition::cursor_marker()
        );

        let (tree, cache) = get_test_deps(Some(setup), query.as_str().into(), &pool).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let results = complete(params);

        let CompletionItem { label, kind, .. } = results
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "cool");
        assert_eq!(kind, CompletionItemKind::Function);
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn prefers_fn_in_select_clause(pool: PgPool) {
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

        let query = format!(r#"select coo{}"#, QueryWithCursorPosition::cursor_marker());

        let (tree, cache) = get_test_deps(Some(setup), query.as_str().into(), &pool).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let results = complete(params);

        let CompletionItem { label, kind, .. } = results
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "cool");
        assert_eq!(kind, CompletionItemKind::Function);
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn prefers_function_in_from_clause_if_invocation(pool: PgPool) {
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

        let query = format!(
            r#"select * from coo{}()"#,
            QueryWithCursorPosition::cursor_marker()
        );

        let (tree, cache) = get_test_deps(Some(setup), query.as_str().into(), &pool).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let results = complete(params);

        let CompletionItem { label, kind, .. } = results
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "cool");
        assert_eq!(kind, CompletionItemKind::Function);
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn only_allows_functions_and_procedures_in_policy_checks(pool: PgPool) {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );

          create or replace function my_cool_foo()
          returns trigger
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;

          create or replace procedure my_cool_proc()
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;

          create or replace function string_concat_state(
            state text, 
            value text, 
          separator text)
          returns text
          language plpgsql
          as $$
          begin
              if state is null then
                  return value;
              else
                  return state || separator || value;
              end if;
          end;
          $$;

          create aggregate string_concat(text, text) (
            sfunc = string_concat_state,
            stype = text,
            initcond = ''
          );
        "#;

        pool.execute(setup).await.unwrap();

        let query = format!(
            r#"create policy "my_pol" on public.coos for insert with check (id = {})"#,
            QueryWithCursorPosition::cursor_marker()
        );

        assert_complete_results(
            query.as_str(),
            vec![
                CompletionAssertion::LabelNotExists("string_concat".into()),
                CompletionAssertion::LabelAndKind("id".into(), CompletionItemKind::Column),
                CompletionAssertion::LabelAndKind("name".into(), CompletionItemKind::Column),
                CompletionAssertion::LabelAndKind(
                    "my_cool_foo".into(),
                    CompletionItemKind::Function,
                ),
                CompletionAssertion::LabelAndKind(
                    "my_cool_proc".into(),
                    CompletionItemKind::Function,
                ),
                CompletionAssertion::LabelAndKind(
                    "string_concat_state".into(),
                    CompletionItemKind::Function,
                ),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn autocompletes_after_schema_in_quotes(pool: PgPool) {
        let setup = r#"
          create schema auth;

          create or replace function auth.my_cool_foo()
          returns trigger
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;
        "#;

        pool.execute(setup).await.unwrap();

        assert_complete_results(
            format!(
                r#"select "auth".{}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::CompletionTextAndRange(
                "my_cool_foo()".into(),
                TextRange::new(14.into(), 14.into()),
            )],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                r#"select "auth"."{}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::CompletionTextAndRange(
                r#"my_cool_foo"()"#.into(),
                TextRange::new(15.into(), 15.into()),
            )],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                r#"select "auth"."{}""#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::CompletionTextAndRange(
                r#"my_cool_foo"()"#.into(),
                TextRange::new(15.into(), 16.into()),
            )],
            None,
            &pool,
        )
        .await;
    }
}
