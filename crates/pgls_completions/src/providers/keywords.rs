use pgls_treesitter::TreesitterContext;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    providers::helper::get_range_to_replace,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

const ALL_KEYWORDS: &[&str] = &[
    "action",
    "add",
    "admin",
    "after",
    "all",
    "alter",
    "always",
    "analyze",
    "and",
    "any",
    "array",
    "as",
    "asc",
    "atomic",
    "attribute",
    "authorization",
    "before",
    "begin",
    "between",
    "bigint",
    "bigserial",
    "binary",
    "bit",
    "boolean",
    "brin",
    "btree",
    "by",
    "bytea",
    "cache",
    "called",
    "cascade",
    "cascaded",
    "case",
    "cast",
    "char",
    "character",
    "characteristics",
    "check",
    "collate",
    "column",
    "columns",
    "comment",
    "commit",
    "committed",
    "compression",
    "concurrently",
    "conflict",
    "connection",
    "constraint",
    "constraints",
    "copy",
    "cost",
    "create",
    "cross",
    "csv",
    "current",
    "current_role",
    "current_timestamp",
    "current_user",
    "cycle",
    "data",
    "database",
    "date",
    "decimal",
    "declare",
    "default",
    "deferrable",
    "deferred",
    "definer",
    "delete",
    "delimiter",
    "desc",
    "disable_page_skipping",
    "distinct",
    "do",
    "double",
    "drop",
    "each",
    "else",
    "encoding",
    "encrypted",
    "end",
    "enum",
    "escape",
    "except",
    "exclude",
    "execute",
    "exists",
    "explain",
    "extended",
    "extension",
    "external",
    "false",
    "filter",
    "first",
    "float",
    "following",
    "for",
    "force",
    "force_not_null",
    "force_null",
    "force_quote",
    "foreign",
    "format",
    "freeze",
    "from",
    "full",
    "function",
    "functions",
    "generated",
    "gin",
    "gist",
    "grant",
    "granted",
    "group",
    "groups",
    "hash",
    "having",
    "header",
    "if",
    "immediate",
    "immutable",
    "in",
    "increment",
    "index",
    "index_cleanup",
    "inet",
    "inherit",
    "initially",
    "inner",
    "inout",
    "input",
    "insert",
    "instead",
    "int",
    "intersect",
    "interval",
    "into",
    "invoker",
    "is",
    "isolation",
    "join",
    "json",
    "jsonb",
    "key",
    "language",
    "last",
    "lateral",
    "leakproof",
    "left",
    "level",
    "like",
    "limit",
    "list",
    "local",
    "location",
    "logged",
    "main",
    "maintain",
    "match",
    "matched",
    "materialized",
    "maxvalue",
    "merge",
    "minvalue",
    "money",
    "name",
    "names",
    "natural",
    "new",
    "no",
    "none",
    "not",
    "nothing",
    "nowait",
    "null",
    "nulls",
    "numeric",
    "of",
    "off",
    "offset",
    "oid",
    "oids",
    "old",
    "on",
    "only",
    "option",
    "or",
    "order",
    "ordinality",
    "others",
    "out",
    "outer",
    "over",
    "overriding",
    "owned",
    "owner",
    "parallel",
    "partition",
    "partitioned",
    "password",
    "permissive",
    "plain",
    "policy",
    "precedes",
    "preceding",
    "precision",
    "primary",
    "privileges",
    "procedure",
    "procedures",
    "process_toast",
    "program",
    "public",
    "quote",
    "range",
    "read",
    "real",
    "recursive",
    "references",
    "referencing",
    "regclass",
    "regnamespace",
    "regproc",
    "regtype",
    "rename",
    "repeatable",
    "replace",
    "replication",
    "reset",
    "restart",
    "restrict",
    "restricted",
    "restrictive",
    "return",
    "returning",
    "returns",
    "revoke",
    "rewrite",
    "right",
    "role",
    "rollback",
    "routine",
    "routines",
    "row",
    "rows",
    "safe",
    "schema",
    "security",
    "select",
    "sequence",
    "serial",
    "serializable",
    "session",
    "session_user",
    "set",
    "setof",
    "show",
    "similar",
    "skip_locked",
    "smallint",
    "smallserial",
    "snapshot",
    "some",
    "spgist",
    "stable",
    "start",
    "statement",
    "statistics",
    "stdin",
    "storage",
    "stored",
    "strict",
    "support",
    "system",
    "table",
    "tables",
    "tablespace",
    "temp",
    "temporary",
    "text",
    "then",
    "ties",
    "time",
    "timestamp",
    "timestamptz",
    "to",
    "transaction",
    "trigger",
    "true",
    "truncate",
    "type",
    "unbounded",
    "uncommitted",
    "union",
    "unique",
    "unlogged",
    "unsafe",
    "until",
    "update",
    "user",
    "using",
    "uuid",
    "vacuum",
    "valid",
    "value",
    "values",
    "varchar",
    "variadic",
    "varying",
    "verbose",
    "version",
    "view",
    "volatile",
    "when",
    "where",
    "window",
    "with",
    "without",
    "write",
    "xml",
    "zone",
];

pub fn complete_keywords<'a>(
    ctx: &TreesitterContext<'a>,
    builder: &mut CompletionBuilder<'a>,
    use_upper_case: bool,
) {
    let keywords = ALL_KEYWORDS
        .iter()
        .filter(|kw| ctx.possible_keywords_at_position.contains(kw));

    for kw in keywords {
        let relevance = CompletionRelevanceData::Keyword(kw);

        let kw = if use_upper_case {
            kw.to_ascii_uppercase()
        } else {
            kw.to_string()
        };

        let item = PossibleCompletionItem {
            label: kw.clone(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: "".into(),
            kind: CompletionItemKind::Keyword,
            completion_text: Some(CompletionText {
                text: kw,
                range: get_range_to_replace(ctx),
                is_snippet: false,
            }),
            detail: None,
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {

    use pgls_test_utils::QueryWithCursorPosition;
    use sqlx::PgPool;

    use crate::test_helper::{
        CompletionAssertion, TestCompletionsCase, TestCompletionsSuite, assert_complete_results,
        assert_no_complete_results,
    };

    #[sqlx::test]
    async fn completes_keywords(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!("se{}", QueryWithCursorPosition::cursor_marker());

        assert_complete_results(
            query.as_str(),
            vec![
                CompletionAssertion::LabelAndKind(
                    "security".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "select".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "sequence".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "serial".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "serializable".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "session".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "session_user".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind("set".into(), crate::CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind(
                    "setof".into(),
                    crate::CompletionItemKind::Keyword,
                ),
            ],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn completes_after_from_clause(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("select email from users <sql>")
                    .type_sql("where id = 1;"),
            )
            .snapshot("after_from_clause")
            .await;
    }

    #[sqlx::test]
    async fn completes_join_kw(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!(
            "select * from public.users left {}",
            QueryWithCursorPosition::cursor_marker()
        );

        assert_complete_results(
            query.as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "join".into(),
                crate::CompletionItemKind::Keyword,
            )],
            Some(setup),
            &pool,
        )
        .await;
    }
}
