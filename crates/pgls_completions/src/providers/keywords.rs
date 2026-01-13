use pgls_treesitter::TreesitterContext;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    providers::helper::get_range_to_replace,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

#[derive(Debug, Clone, Copy)]
pub struct SqlKeyword {
    pub name: &'static str,
    pub require_prefix: bool,
    pub starts_statement: bool,
}

impl SqlKeyword {
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            require_prefix: false,
            starts_statement: false,
        }
    }

    const fn require_prefix(mut self) -> Self {
        self.require_prefix = true;
        self
    }

    const fn starts_statement(mut self) -> Self {
        self.starts_statement = true;
        self
    }
}

pub static ALL_KEYWORDS: &[SqlKeyword] = &[
    SqlKeyword::new("action"),
    SqlKeyword::new("add"),
    SqlKeyword::new("admin"),
    SqlKeyword::new("after"),
    SqlKeyword::new("all"),
    SqlKeyword::new("alter").starts_statement(),
    SqlKeyword::new("always"),
    SqlKeyword::new("analyze").starts_statement(),
    SqlKeyword::new("and").require_prefix(),
    SqlKeyword::new("any").require_prefix(),
    SqlKeyword::new("array").require_prefix(),
    SqlKeyword::new("as").require_prefix(),
    SqlKeyword::new("asc"),
    SqlKeyword::new("atomic"),
    SqlKeyword::new("attribute"),
    SqlKeyword::new("authorization"),
    SqlKeyword::new("before"),
    SqlKeyword::new("begin").starts_statement(),
    SqlKeyword::new("between").require_prefix(),
    SqlKeyword::new("bigint"),
    SqlKeyword::new("bigserial"),
    SqlKeyword::new("binary"),
    SqlKeyword::new("bit"),
    SqlKeyword::new("boolean"),
    SqlKeyword::new("brin"),
    SqlKeyword::new("btree"),
    SqlKeyword::new("by"),
    SqlKeyword::new("bytea"),
    SqlKeyword::new("cache"),
    SqlKeyword::new("called"),
    SqlKeyword::new("cascade"),
    SqlKeyword::new("cascaded"),
    SqlKeyword::new("case"),
    SqlKeyword::new("cast").require_prefix(),
    SqlKeyword::new("char"),
    SqlKeyword::new("character"),
    SqlKeyword::new("characteristics"),
    SqlKeyword::new("check"),
    SqlKeyword::new("collate"),
    SqlKeyword::new("column"),
    SqlKeyword::new("columns"),
    SqlKeyword::new("comment").starts_statement(),
    SqlKeyword::new("commit").starts_statement(),
    SqlKeyword::new("committed"),
    SqlKeyword::new("compression"),
    SqlKeyword::new("concurrently"),
    SqlKeyword::new("conflict"),
    SqlKeyword::new("connection"),
    SqlKeyword::new("constraint"),
    SqlKeyword::new("constraints"),
    SqlKeyword::new("copy").starts_statement(),
    SqlKeyword::new("cost"),
    SqlKeyword::new("create").starts_statement(),
    SqlKeyword::new("cross"),
    SqlKeyword::new("csv"),
    SqlKeyword::new("current"),
    SqlKeyword::new("current_role"),
    SqlKeyword::new("current_timestamp"),
    SqlKeyword::new("current_user"),
    SqlKeyword::new("cycle"),
    SqlKeyword::new("data"),
    SqlKeyword::new("database"),
    SqlKeyword::new("date"),
    SqlKeyword::new("decimal"),
    SqlKeyword::new("declare").starts_statement(),
    SqlKeyword::new("default"),
    SqlKeyword::new("deferrable"),
    SqlKeyword::new("deferred"),
    SqlKeyword::new("definer"),
    SqlKeyword::new("delete").starts_statement(),
    SqlKeyword::new("delimiter"),
    SqlKeyword::new("desc"),
    SqlKeyword::new("disable_page_skipping"),
    SqlKeyword::new("distinct"),
    SqlKeyword::new("do").starts_statement(),
    SqlKeyword::new("double"),
    SqlKeyword::new("drop").starts_statement(),
    SqlKeyword::new("each"),
    SqlKeyword::new("else"),
    SqlKeyword::new("encoding"),
    SqlKeyword::new("encrypted"),
    SqlKeyword::new("end"),
    SqlKeyword::new("enum"),
    SqlKeyword::new("escape"),
    SqlKeyword::new("except"),
    SqlKeyword::new("exclude"),
    SqlKeyword::new("execute"),
    SqlKeyword::new("exists").require_prefix(),
    SqlKeyword::new("explain").starts_statement(),
    SqlKeyword::new("extended"),
    SqlKeyword::new("extension"),
    SqlKeyword::new("external"),
    SqlKeyword::new("false").require_prefix(),
    SqlKeyword::new("filter"),
    SqlKeyword::new("first"),
    SqlKeyword::new("float"),
    SqlKeyword::new("following"),
    SqlKeyword::new("for"),
    SqlKeyword::new("force"),
    SqlKeyword::new("force_not_null"),
    SqlKeyword::new("force_null"),
    SqlKeyword::new("force_quote"),
    SqlKeyword::new("foreign"),
    SqlKeyword::new("format"),
    SqlKeyword::new("freeze"),
    SqlKeyword::new("from"),
    SqlKeyword::new("full"),
    SqlKeyword::new("function"),
    SqlKeyword::new("functions"),
    SqlKeyword::new("generated"),
    SqlKeyword::new("gin"),
    SqlKeyword::new("gist"),
    SqlKeyword::new("grant").starts_statement(),
    SqlKeyword::new("granted"),
    SqlKeyword::new("group"),
    SqlKeyword::new("groups"),
    SqlKeyword::new("hash"),
    SqlKeyword::new("having"),
    SqlKeyword::new("header"),
    SqlKeyword::new("if"),
    SqlKeyword::new("immediate"),
    SqlKeyword::new("immutable"),
    SqlKeyword::new("in").require_prefix(),
    SqlKeyword::new("increment"),
    SqlKeyword::new("index"),
    SqlKeyword::new("index_cleanup"),
    SqlKeyword::new("inet"),
    SqlKeyword::new("inherit"),
    SqlKeyword::new("initially"),
    SqlKeyword::new("inner"),
    SqlKeyword::new("inout"),
    SqlKeyword::new("input"),
    SqlKeyword::new("insert").starts_statement(),
    SqlKeyword::new("instead"),
    SqlKeyword::new("int"),
    SqlKeyword::new("intersect"),
    SqlKeyword::new("interval").require_prefix(),
    SqlKeyword::new("into"),
    SqlKeyword::new("invoker"),
    SqlKeyword::new("is"),
    SqlKeyword::new("isolation"),
    SqlKeyword::new("join"),
    SqlKeyword::new("json"),
    SqlKeyword::new("jsonb"),
    SqlKeyword::new("key"),
    SqlKeyword::new("language"),
    SqlKeyword::new("last"),
    SqlKeyword::new("lateral"),
    SqlKeyword::new("leakproof"),
    SqlKeyword::new("left"),
    SqlKeyword::new("level"),
    SqlKeyword::new("like").require_prefix(),
    SqlKeyword::new("limit"),
    SqlKeyword::new("list"),
    SqlKeyword::new("local"),
    SqlKeyword::new("location"),
    SqlKeyword::new("logged"),
    SqlKeyword::new("main"),
    SqlKeyword::new("maintain"),
    SqlKeyword::new("match"),
    SqlKeyword::new("matched"),
    SqlKeyword::new("materialized"),
    SqlKeyword::new("maxvalue"),
    SqlKeyword::new("merge").starts_statement(),
    SqlKeyword::new("minvalue"),
    SqlKeyword::new("money"),
    SqlKeyword::new("name"),
    SqlKeyword::new("names"),
    SqlKeyword::new("natural"),
    SqlKeyword::new("new"),
    SqlKeyword::new("no"),
    SqlKeyword::new("none"),
    SqlKeyword::new("not").require_prefix(),
    SqlKeyword::new("nothing"),
    SqlKeyword::new("nowait"),
    SqlKeyword::new("null").require_prefix(),
    SqlKeyword::new("nulls"),
    SqlKeyword::new("numeric"),
    SqlKeyword::new("of"),
    SqlKeyword::new("off"),
    SqlKeyword::new("offset"),
    SqlKeyword::new("oid"),
    SqlKeyword::new("oids"),
    SqlKeyword::new("old"),
    SqlKeyword::new("on"),
    SqlKeyword::new("only").require_prefix(),
    SqlKeyword::new("option"),
    SqlKeyword::new("or").require_prefix(),
    SqlKeyword::new("order"),
    SqlKeyword::new("ordinality"),
    SqlKeyword::new("others"),
    SqlKeyword::new("out"),
    SqlKeyword::new("outer"),
    SqlKeyword::new("over"),
    SqlKeyword::new("overriding"),
    SqlKeyword::new("owned"),
    SqlKeyword::new("owner"),
    SqlKeyword::new("parallel"),
    SqlKeyword::new("partition"),
    SqlKeyword::new("partitioned"),
    SqlKeyword::new("password"),
    SqlKeyword::new("permissive"),
    SqlKeyword::new("plain"),
    SqlKeyword::new("policy"),
    SqlKeyword::new("precedes"),
    SqlKeyword::new("preceding"),
    SqlKeyword::new("precision"),
    SqlKeyword::new("primary"),
    SqlKeyword::new("privileges"),
    SqlKeyword::new("procedure"),
    SqlKeyword::new("procedures"),
    SqlKeyword::new("process_toast"),
    SqlKeyword::new("program"),
    SqlKeyword::new("public"),
    SqlKeyword::new("quote"),
    SqlKeyword::new("range"),
    SqlKeyword::new("read"),
    SqlKeyword::new("real"),
    SqlKeyword::new("recursive"),
    SqlKeyword::new("references"),
    SqlKeyword::new("referencing"),
    SqlKeyword::new("regclass"),
    SqlKeyword::new("regnamespace"),
    SqlKeyword::new("regproc"),
    SqlKeyword::new("regtype"),
    SqlKeyword::new("rename"),
    SqlKeyword::new("repeatable"),
    SqlKeyword::new("replace"),
    SqlKeyword::new("replication"),
    SqlKeyword::new("reset").starts_statement(),
    SqlKeyword::new("restart"),
    SqlKeyword::new("restrict"),
    SqlKeyword::new("restricted"),
    SqlKeyword::new("restrictive"),
    SqlKeyword::new("return"),
    SqlKeyword::new("returning"),
    SqlKeyword::new("returns"),
    SqlKeyword::new("revoke").starts_statement(),
    SqlKeyword::new("rewrite"),
    SqlKeyword::new("right"),
    SqlKeyword::new("role"),
    SqlKeyword::new("rollback").starts_statement(),
    SqlKeyword::new("routine"),
    SqlKeyword::new("routines"),
    SqlKeyword::new("row"),
    SqlKeyword::new("rows"),
    SqlKeyword::new("safe"),
    SqlKeyword::new("schema"),
    SqlKeyword::new("security"),
    SqlKeyword::new("select").starts_statement(),
    SqlKeyword::new("sequence"),
    SqlKeyword::new("serial"),
    SqlKeyword::new("serializable"),
    SqlKeyword::new("session"),
    SqlKeyword::new("session_user"),
    SqlKeyword::new("set").starts_statement(),
    SqlKeyword::new("setof"),
    SqlKeyword::new("show").starts_statement(),
    SqlKeyword::new("similar").require_prefix(),
    SqlKeyword::new("skip_locked"),
    SqlKeyword::new("smallint"),
    SqlKeyword::new("smallserial"),
    SqlKeyword::new("snapshot"),
    SqlKeyword::new("some"),
    SqlKeyword::new("spgist"),
    SqlKeyword::new("stable"),
    SqlKeyword::new("start"),
    SqlKeyword::new("statement"),
    SqlKeyword::new("statistics"),
    SqlKeyword::new("stdin"),
    SqlKeyword::new("storage"),
    SqlKeyword::new("stored"),
    SqlKeyword::new("strict"),
    SqlKeyword::new("support"),
    SqlKeyword::new("system"),
    SqlKeyword::new("table"),
    SqlKeyword::new("tables"),
    SqlKeyword::new("tablespace"),
    SqlKeyword::new("temp"),
    SqlKeyword::new("temporary"),
    SqlKeyword::new("text"),
    SqlKeyword::new("then").require_prefix(),
    SqlKeyword::new("ties"),
    SqlKeyword::new("time"),
    SqlKeyword::new("timestamp"),
    SqlKeyword::new("timestamptz"),
    SqlKeyword::new("to"),
    SqlKeyword::new("transaction"),
    SqlKeyword::new("trigger"),
    SqlKeyword::new("true").require_prefix(),
    SqlKeyword::new("truncate").starts_statement(),
    SqlKeyword::new("type"),
    SqlKeyword::new("unbounded"),
    SqlKeyword::new("uncommitted"),
    SqlKeyword::new("union"),
    SqlKeyword::new("unique"),
    SqlKeyword::new("unlogged"),
    SqlKeyword::new("unsafe"),
    SqlKeyword::new("until"),
    SqlKeyword::new("update").starts_statement(),
    SqlKeyword::new("user"),
    SqlKeyword::new("using"),
    SqlKeyword::new("uuid"),
    SqlKeyword::new("vacuum").starts_statement(),
    SqlKeyword::new("valid"),
    SqlKeyword::new("value"),
    SqlKeyword::new("values"),
    SqlKeyword::new("varchar"),
    SqlKeyword::new("variadic"),
    SqlKeyword::new("varying"),
    SqlKeyword::new("verbose"),
    SqlKeyword::new("version"),
    SqlKeyword::new("view"),
    SqlKeyword::new("volatile"),
    SqlKeyword::new("when"),
    SqlKeyword::new("where"),
    SqlKeyword::new("window"),
    SqlKeyword::new("with").starts_statement(),
    SqlKeyword::new("without"),
    SqlKeyword::new("write"),
    SqlKeyword::new("xml"),
    SqlKeyword::new("zone"),
];

pub fn complete_keywords<'a>(
    ctx: &TreesitterContext<'a>,
    builder: &mut CompletionBuilder<'a>,
    use_upper_case: bool,
) {
    // no keyword completions if we start with a quote
    if ctx
        .get_node_under_cursor_content()
        .is_some_and(|n| n.starts_with('"'))
    {
        return;
    }

    let keywords_to_try = ALL_KEYWORDS.iter().filter(|kw| {
        ctx.tree.root_node().has_error() || ctx.possible_keywords_at_position.contains(&kw.name)
    });

    for kw in keywords_to_try {
        let relevance = CompletionRelevanceData::Keyword(kw);

        let label = if use_upper_case {
            kw.name.to_ascii_uppercase()
        } else {
            kw.name.to_string()
        };

        let item = PossibleCompletionItem {
            label: label.clone(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: "".into(),
            kind: CompletionItemKind::Keyword,
            completion_text: Some(CompletionText {
                text: label,
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

    use crate::{
        CompletionItemKind,
        test_helper::{
            CompletionAssertion, TestCompletionsCase, TestCompletionsSuite,
            assert_complete_results, assert_no_complete_results,
        },
    };

    #[sqlx::test]
    async fn completes_stmt_start_keywords(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!("{}", QueryWithCursorPosition::cursor_marker());

        assert_complete_results(
            query.as_str(),
            vec![
                CompletionAssertion::LabelAndKind(
                    "insert".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "reset".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "select".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind("set".into(), crate::CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind(
                    "truncate".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "update".into(),
                    crate::CompletionItemKind::Keyword,
                ),
            ],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn completes_keywords(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!("s{}", QueryWithCursorPosition::cursor_marker());

        assert_complete_results(
            query.as_str(),
            vec![
                CompletionAssertion::LabelAndKind(
                    "select".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind("set".into(), crate::CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind(
                    "insert".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "reset".into(),
                    crate::CompletionItemKind::Keyword,
                ),
            ],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn does_not_complete_from_after_select(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!("select f{}", QueryWithCursorPosition::cursor_marker());

        assert_complete_results(
            query.as_str(),
            vec![CompletionAssertion::LabelNotExists("from".into())], // keyword `false` is fine
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn completes_columsn_after_select(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!("select {}", QueryWithCursorPosition::cursor_marker());

        assert_complete_results(
            query.as_str(),
            vec![CompletionAssertion::KindNotExists(
                CompletionItemKind::Keyword,
            )],
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
    async fn stays_within_order_clause(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!(
            "select * from public.users order {}",
            QueryWithCursorPosition::cursor_marker()
        );

        assert_complete_results(
            query.as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "by".into(),
                crate::CompletionItemKind::Keyword,
            )],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn only_allows_identifier_in_alias_clause(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!(
            "select * from public.users as {}",
            QueryWithCursorPosition::cursor_marker()
        );

        assert_no_complete_results(query.as_str(), Some(setup), &pool).await;
    }

    #[sqlx::test]
    async fn completes_from_keyword(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!("select * f{}", QueryWithCursorPosition::cursor_marker());

        assert_complete_results(
            query.as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "from".into(),
                crate::CompletionItemKind::Keyword,
            )],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn completes_from_keyword_after_aliases(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!(
            "select email as em {}",
            QueryWithCursorPosition::cursor_marker()
        );

        assert_complete_results(
            query.as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "from".into(),
                crate::CompletionItemKind::Keyword,
            )],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn adds_where_after_clause(pool: PgPool) {
        let query = format!(
            "select * from public.users us left join client_settings as cs on us.id = cs.client_id whe{}",
            QueryWithCursorPosition::cursor_marker()
        );

        assert_complete_results(
            query.as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "where".into(),
                crate::CompletionItemKind::Keyword,
            )],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn completes_keywords_after_column_aliases(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!(
            "select email from public.users als (id, email) {}",
            QueryWithCursorPosition::cursor_marker()
        );

        assert_complete_results(
            query.as_str(),
            vec![
                CompletionAssertion::LabelAndKind(
                    "cross".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "full".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "group".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "inner".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "join".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "left".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "limit".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "natural".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "order".into(),
                    crate::CompletionItemKind::Keyword,
                ),
            ],
            Some(setup),
            &pool,
        )
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
            vec![
                CompletionAssertion::LabelAndKind(
                    "join".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "outer".into(),
                    crate::CompletionItemKind::Keyword,
                ),
            ],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn stays_in_joins(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!(
            "select * from public.users u join public.something s {}",
            QueryWithCursorPosition::cursor_marker()
        );

        assert_complete_results(
            query.as_str(),
            vec![CompletionAssertion::LabelNotExists("join".into())],
            Some(setup),
            &pool,
        )
        .await;
    }
}
