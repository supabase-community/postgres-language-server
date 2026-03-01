use pgls_text_size::TextSize;

use pgls_treesitter::{TreeSitterContextParams, context::TreesitterContext};

use crate::{
    builder::CompletionBuilder,
    item::CompletionItem,
    providers::{
        complete_columns, complete_functions, complete_keywords, complete_policies, complete_roles,
        complete_schemas, complete_tables,
    },
    sanitization::SanitizedCompletionParams,
};

pub const LIMIT: usize = 50;

#[derive(Debug)]
pub struct CompletionParams<'a> {
    pub position: TextSize,
    pub schema: &'a pgls_schema_cache::SchemaCache,
    pub text: String,
    pub tree: &'a tree_sitter::Tree,
}

#[tracing::instrument(level = "debug", skip_all, fields(
    text = params.text,
    position = params.position.to_string()
))]
pub fn complete(params: CompletionParams) -> Vec<CompletionItem> {
    let uses_upper_case = params
        .text
        .split_ascii_whitespace()
        // filter out special chars and numbers
        .filter(|word| word.chars().all(|c| c.is_alphabetic()))
        .any(|t| t == t.to_ascii_uppercase());

    let sanitized_params = SanitizedCompletionParams::from(params);

    let ctx = TreesitterContext::new(TreeSitterContextParams {
        position: sanitized_params.position,
        text: &sanitized_params.text,
        tree: &sanitized_params.tree,
    });

    let mut builder = CompletionBuilder::new(&ctx);

    complete_tables(&ctx, sanitized_params.schema, &mut builder);
    complete_functions(&ctx, sanitized_params.schema, &mut builder);
    complete_columns(&ctx, sanitized_params.schema, &mut builder);
    complete_schemas(&ctx, sanitized_params.schema, &mut builder);
    complete_policies(&ctx, sanitized_params.schema, &mut builder);
    complete_roles(&ctx, sanitized_params.schema, &mut builder);
    complete_keywords(&ctx, &mut builder, uses_upper_case);

    builder.finish()
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use crate::test_helper::{TestCompletionsCase, TestCompletionsSuite};

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completions_in_update_statements(pool: PgPool) {
        let setup = r#"
            create table instruments (
                id bigint primary key generated always as identity,
                name text not null,
                z text
            );

            create table others (
                id serial primary key,
                a text,
                b text
            );
        "#;

        TestCompletionsSuite::new(&pool, Some(setup))
            // basic SET + WHERE (full statement typed once)
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("update instruments set name = 'new' where id = 1;"),
            )
            // multi-col SET + alias
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("update instruments as i <sql> where i.id = 1;")
                    .type_sql("set name = 'x', z = 'y'"),
            )
            // RETURNING clause
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("update instruments set name = 'x' <sql>")
                    .type_sql("returning id, name;"),
            )
            // SET with DEFAULT
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("update instruments <sql> where id = 1;")
                    .type_sql("set name = default, z = 'y'"),
            )
            // FROM + join
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("update instruments set name = o.a <sql>")
                    .type_sql("from others o where instruments.id = o.id returning name, z;"),
            )
            // subquery in SET
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("update instruments set name = (<sql>) where id = 1;")
                    .type_sql("select a from others where id = 1"),
            )
            .snapshot("completions_in_update_statements")
            .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completions_in_insert_statements(pool: PgPool) {
        let setup = r#"
            create table instruments (
                id bigint primary key generated always as identity,
                name text not null,
                z text
            );

            create table others (
                id serial primary key,
                a text,
                b text
            );
        "#;

        TestCompletionsSuite::new(&pool, Some(setup))
            // basic VALUES (full statement typed once)
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("insert into instruments (id, name) values (1, 'bass');"),
            )
            // RETURNING clause
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        "insert into instruments (id, name) values (1, 'x') <sql>",
                    )
                    .type_sql("returning id, name;"),
            )
            // multi-row VALUES with DEFAULT
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("insert into instruments (id, name, z) <sql>")
                    .type_sql("values (1, 'a', 'b'), (2, 'c', default);"),
            )
            // INSERT SELECT
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("insert into instruments (id, name) <sql>")
                    .type_sql("select id, a from others;"),
            )
            // schema-qualified table
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("insert into <sql> (id, name) values (1, 'x');")
                    .type_sql("public.instruments"),
            )
            .snapshot("completions_in_insert_statements")
            .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completions_in_copy_statements(pool: PgPool) {
        let setup = r#"
            create table instruments (
                id bigint primary key generated always as identity,
                name text not null,
                z text
            );

            create table others (
                id serial primary key,
                a text,
                b text
            );
        "#;

        TestCompletionsSuite::new(&pool, Some(setup))
            // basic COPY TO (full statement typed once)
            .with_case(TestCompletionsCase::new().type_sql("copy instruments to stdout;"))
            // COPY FROM
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("copy instruments <sql>")
                    .type_sql("from stdin;"),
            )
            // column list
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("copy instruments (<sql>) to stdout;")
                    .type_sql("id, name"),
            )
            // WITH format options (TO)
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("copy instruments to stdout <sql>")
                    .type_sql("with (format csv, header);"),
            )
            // WITH format options (FROM)
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("copy instruments from stdin <sql>")
                    .type_sql("with (format csv, header);"),
            )
            // subquery
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("copy (<sql>) to stdout;")
                    .type_sql("select * from instruments where id > 0"),
            )
            .snapshot("completions_in_copy_statements")
            .await;
    }
}
