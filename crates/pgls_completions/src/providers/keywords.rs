use pgls_treesitter::TreesitterContext;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    providers::helper::get_keyword_range_to_replace,
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
    SqlKeyword::new("analyze")
        .starts_statement()
        .require_prefix(),
    SqlKeyword::new("and").require_prefix(),
    SqlKeyword::new("any").require_prefix(),
    SqlKeyword::new("array").require_prefix(),
    SqlKeyword::new("as").require_prefix(),
    SqlKeyword::new("asc"),
    SqlKeyword::new("atomic"),
    SqlKeyword::new("attribute"),
    SqlKeyword::new("authorization"),
    SqlKeyword::new("auto"),
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
    SqlKeyword::new("buffer_usage_limit"),
    SqlKeyword::new("buffers"),
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
    SqlKeyword::new("costs"),
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
    SqlKeyword::new("enforced"),
    SqlKeyword::new("enum"),
    SqlKeyword::new("escape"),
    SqlKeyword::new("except"),
    SqlKeyword::new("exclude"),
    SqlKeyword::new("execute"),
    SqlKeyword::new("exists").require_prefix(),
    SqlKeyword::new("explain")
        .starts_statement()
        .require_prefix(),
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
    SqlKeyword::new("generic_plan"),
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
    SqlKeyword::new("ignore"),
    SqlKeyword::new("immediate"),
    SqlKeyword::new("immutable"),
    SqlKeyword::new("in").require_prefix(),
    SqlKeyword::new("increment"),
    SqlKeyword::new("index"),
    SqlKeyword::new("index_cleanup"),
    SqlKeyword::new("inet"),
    SqlKeyword::new("inherit"),
    SqlKeyword::new("inherits"),
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
    SqlKeyword::new("is").require_prefix(),
    SqlKeyword::new("isolation"),
    SqlKeyword::new("join"),
    SqlKeyword::new("json"),
    SqlKeyword::new("jsonb"),
    SqlKeyword::new("key"),
    SqlKeyword::new("language"),
    SqlKeyword::new("large"),
    SqlKeyword::new("last"),
    SqlKeyword::new("lateral"),
    SqlKeyword::new("leakproof"),
    SqlKeyword::new("left"),
    SqlKeyword::new("level"),
    SqlKeyword::new("like").require_prefix(),
    SqlKeyword::new("limit"),
    SqlKeyword::new("list"),
    SqlKeyword::new("local"),
    SqlKeyword::new("log_verbosity"),
    SqlKeyword::new("logged"),
    SqlKeyword::new("main"),
    SqlKeyword::new("maintain"),
    SqlKeyword::new("match"),
    SqlKeyword::new("matched"),
    SqlKeyword::new("materialized"),
    SqlKeyword::new("maxvalue"),
    SqlKeyword::new("memory"),
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
    SqlKeyword::new("null").require_prefix(),
    SqlKeyword::new("nulls"),
    SqlKeyword::new("numeric"),
    SqlKeyword::new("objects"),
    SqlKeyword::new("of"),
    SqlKeyword::new("off"),
    SqlKeyword::new("offset"),
    SqlKeyword::new("oid"),
    SqlKeyword::new("oids"),
    SqlKeyword::new("old"),
    SqlKeyword::new("on"),
    SqlKeyword::new("on_error"),
    SqlKeyword::new("only").require_prefix(),
    SqlKeyword::new("only_database_stats"),
    SqlKeyword::new("option"),
    SqlKeyword::new("or").require_prefix(),
    SqlKeyword::new("order"),
    SqlKeyword::new("ordinality"),
    SqlKeyword::new("others"),
    SqlKeyword::new("out"),
    SqlKeyword::new("outer"),
    SqlKeyword::new("over"),
    SqlKeyword::new("overlaps"),
    SqlKeyword::new("overriding"),
    SqlKeyword::new("owned"),
    SqlKeyword::new("owner"),
    SqlKeyword::new("parallel"),
    SqlKeyword::new("partition"),
    SqlKeyword::new("password"),
    SqlKeyword::new("period"),
    SqlKeyword::new("permissive"),
    SqlKeyword::new("plain"),
    SqlKeyword::new("policy"),
    SqlKeyword::new("preceding"),
    SqlKeyword::new("precision"),
    SqlKeyword::new("primary"),
    SqlKeyword::new("privileges"),
    SqlKeyword::new("procedure"),
    SqlKeyword::new("procedures"),
    SqlKeyword::new("process_main"),
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
    SqlKeyword::new("reject_limit"),
    SqlKeyword::new("rename"),
    SqlKeyword::new("repeatable"),
    SqlKeyword::new("replace"),
    SqlKeyword::new("reset").starts_statement(),
    SqlKeyword::new("restart"),
    SqlKeyword::new("restrict"),
    SqlKeyword::new("restricted"),
    SqlKeyword::new("restrictive"),
    SqlKeyword::new("return"),
    SqlKeyword::new("returning"),
    SqlKeyword::new("returns"),
    SqlKeyword::new("revoke").starts_statement(),
    SqlKeyword::new("right"),
    SqlKeyword::new("role"),
    SqlKeyword::new("rollback").starts_statement(),
    SqlKeyword::new("routine"),
    SqlKeyword::new("routines"),
    SqlKeyword::new("row"),
    SqlKeyword::new("rows"),
    SqlKeyword::new("safe"),
    SqlKeyword::new("schema"),
    SqlKeyword::new("schemas"),
    SqlKeyword::new("security"),
    SqlKeyword::new("select").starts_statement(),
    SqlKeyword::new("sequence"),
    SqlKeyword::new("sequences"),
    SqlKeyword::new("serial"),
    SqlKeyword::new("serializable"),
    SqlKeyword::new("session"),
    SqlKeyword::new("session_user"),
    SqlKeyword::new("set").starts_statement(),
    SqlKeyword::new("setof"),
    SqlKeyword::new("settings"),
    SqlKeyword::new("share"),
    SqlKeyword::new("show").starts_statement(),
    SqlKeyword::new("silent"),
    SqlKeyword::new("similar").require_prefix(),
    SqlKeyword::new("skip_database_stats"),
    SqlKeyword::new("skip_locked"),
    SqlKeyword::new("smallint"),
    SqlKeyword::new("smallserial"),
    SqlKeyword::new("snapshot"),
    SqlKeyword::new("some"),
    SqlKeyword::new("source"),
    SqlKeyword::new("spgist"),
    SqlKeyword::new("stable"),
    SqlKeyword::new("start"),
    SqlKeyword::new("statement"),
    SqlKeyword::new("statistics"),
    SqlKeyword::new("stdin"),
    SqlKeyword::new("stdout"),
    SqlKeyword::new("stop"),
    SqlKeyword::new("storage"),
    SqlKeyword::new("stored"),
    SqlKeyword::new("strict"),
    SqlKeyword::new("summary"),
    SqlKeyword::new("support"),
    SqlKeyword::new("system"),
    SqlKeyword::new("table"),
    SqlKeyword::new("tables"),
    SqlKeyword::new("tablespace"),
    SqlKeyword::new("target"),
    SqlKeyword::new("temp"),
    SqlKeyword::new("temporary"),
    SqlKeyword::new("text"),
    SqlKeyword::new("then").require_prefix(),
    SqlKeyword::new("ties"),
    SqlKeyword::new("time"),
    SqlKeyword::new("timestamp"),
    SqlKeyword::new("timestamptz"),
    SqlKeyword::new("timing"),
    SqlKeyword::new("to"),
    SqlKeyword::new("transaction"),
    SqlKeyword::new("trigger"),
    SqlKeyword::new("true").require_prefix(),
    SqlKeyword::new("truncate").starts_statement(),
    SqlKeyword::new("type"),
    SqlKeyword::new("types"),
    SqlKeyword::new("unbounded"),
    SqlKeyword::new("uncommitted"),
    SqlKeyword::new("union"),
    SqlKeyword::new("unique"),
    SqlKeyword::new("unlogged"),
    SqlKeyword::new("unsafe"),
    SqlKeyword::new("until"),
    SqlKeyword::new("update").starts_statement(),
    SqlKeyword::new("usage"),
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
    SqlKeyword::new("virtual"),
    SqlKeyword::new("volatile"),
    SqlKeyword::new("wal"),
    SqlKeyword::new("when"),
    SqlKeyword::new("where"),
    SqlKeyword::new("window"),
    SqlKeyword::new("with").starts_statement(),
    SqlKeyword::new("without"),
    SqlKeyword::new("write"),
    SqlKeyword::new("xml"),
    SqlKeyword::new("yaml"),
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
                range: get_keyword_range_to_replace(ctx),
                is_snippet: false,
            }),
            detail: None,
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use pgls_test_utils::QueryWithCursorPosition;
    use regex::Regex;
    use sqlx::PgPool;
    use tree_sitter::Language;

    use crate::{
        CompletionItemKind, complete,
        providers::keywords::ALL_KEYWORDS,
        test_helper::{
            CompletionAssertion, TestCompletionsCase, TestCompletionsSuite,
            assert_complete_results, assert_no_complete_results, get_test_deps, get_test_params,
        },
    };

    #[test]
    fn has_all_keywords_from_treesitter_grammar() {
        let expected_keywords = keywords_from_treesitter_grammar();
        let actual_keywords = all_keywords_set();

        let mut missing_keywords = expected_keywords
            .difference(&actual_keywords)
            .cloned()
            .collect::<Vec<_>>();
        missing_keywords.sort_unstable();

        assert!(
            missing_keywords.is_empty(),
            "Found {} keyword(s) from tree-sitter grammar that are missing from ALL_KEYWORDS.\n\
             Add missing entries to ALL_KEYWORDS in crates/pgls_completions/src/providers/keywords.rs.\n\
             Missing keywords:\n{}",
            missing_keywords.len(),
            missing_keywords.join("\n")
        );
    }

    #[test]
    fn has_no_extra_keywords_outside_treesitter_grammar() {
        let expected_keywords = keywords_from_treesitter_grammar();
        let actual_keywords = all_keywords_set();

        let mut extra_keywords = actual_keywords
            .difference(&expected_keywords)
            .cloned()
            .collect::<Vec<_>>();
        extra_keywords.sort_unstable();

        assert!(
            extra_keywords.is_empty(),
            "Found {} keyword(s) in ALL_KEYWORDS that are not tree-sitter grammar keywords.\n\
             Remove these from ALL_KEYWORDS or add matching keyword_* symbols to grammar.js.\n\
             Extra keywords:\n{}",
            extra_keywords.len(),
            extra_keywords.join("\n")
        );
    }

    fn all_keywords_set() -> HashSet<String> {
        ALL_KEYWORDS
            .iter()
            .map(|keyword| keyword.name.to_string())
            .collect::<HashSet<_>>()
    }

    fn keywords_from_treesitter_grammar() -> HashSet<String> {
        let language: Language = pgls_treesitter_grammar::LANGUAGE.into();
        // Tree-sitter generates auxiliary symbol names like `keyword_int_token2` for keyword
        // rules that use `choice(...)`. Strip the suffix to compare canonical keyword names.
        let generated_token_suffix =
            Regex::new(r"_token\d+$").expect("valid regex for generated token suffix");
        let mut keywords = HashSet::new();

        for id in 0..language.node_kind_count() {
            let Some(kind) = language.node_kind_for_id(id as u16) else {
                continue;
            };
            if let Some(keyword) = kind.strip_prefix("keyword_") {
                let keyword = generated_token_suffix.replace(keyword, "").to_string();
                keywords.insert(keyword);
            }
        }

        keywords
    }

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
                    "alter".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "copy".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "create".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "delete".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "drop".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "merge".into(),
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
    async fn completes_columns_after_select(pool: PgPool) {
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
            vec![
                CompletionAssertion::LabelAndKind("for".into(), crate::CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind(
                    "from".into(),
                    crate::CompletionItemKind::Keyword,
                ),
            ],
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
            vec![
                CompletionAssertion::LabelAndKind(
                    "except".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind("for".into(), crate::CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind(
                    "from".into(),
                    crate::CompletionItemKind::Keyword,
                ),
            ],
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
                    "except".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind("for".into(), crate::CompletionItemKind::Keyword),
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
                    "intersect".into(),
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
                    "offset".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "order".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "right".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "union".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "where".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "window".into(),
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

    #[sqlx::test]
    async fn completes_join_after_alias(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!(
            "select * from public.users u {}",
            QueryWithCursorPosition::cursor_marker()
        );

        assert_complete_results(
            query.as_str(),
            vec![
                CompletionAssertion::LabelAndKind("cross".into(), CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind("except".into(), CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind("for".into(), CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind("full".into(), CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind("group".into(), CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind("inner".into(), CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind("intersect".into(), CompletionItemKind::Keyword),
                CompletionAssertion::LabelAndKind("join".into(), CompletionItemKind::Keyword),
            ],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test]
    async fn allows_starting_new_select_stmt(pool: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                email varchar(255)
            );
        "#;

        let query = format!(
            "select * from public.users u; sel{}",
            QueryWithCursorPosition::cursor_marker()
        );

        assert_complete_results(
            query.as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "select".into(),
                CompletionItemKind::Keyword,
            )],
            Some(setup),
            &pool,
        )
        .await;
    }

    /// The keywords offered at the cursor, sorted.
    async fn keywords_at(query: &str, pool: &PgPool) -> Vec<String> {
        let (tree, snapshot) = get_test_deps(None, query.into(), pool).await;
        let params = get_test_params(&tree, &snapshot, query.into());
        let mut keywords = complete(params)
            .into_iter()
            .filter(|item| item.kind == CompletionItemKind::Keyword)
            .map(|item| item.label)
            .collect::<Vec<_>>();
        keywords.sort_unstable();
        keywords
    }

    #[sqlx::test]
    async fn completes_default_privileges_targets(pool: PgPool) {
        let cursor = QueryWithCursorPosition::cursor_marker();

        assert_eq!(
            keywords_at(
                &format!("alter default privileges grant select on {cursor}"),
                &pool
            )
            .await,
            [
                "functions",
                "large",
                "routines",
                "schemas",
                "sequences",
                "tables",
                "types"
            ]
        );
        assert_eq!(
            keywords_at(
                &format!(
                    "alter default privileges in schema public grant select on large {cursor}"
                ),
                &pool
            )
            .await,
            ["objects"]
        );
    }

    #[sqlx::test]
    async fn completes_default_privileges_options_in_any_order(pool: PgPool) {
        let cursor = QueryWithCursorPosition::cursor_marker();

        // `in` needs a prefix.
        assert_eq!(
            keywords_at(&format!("alter default privileges {cursor}"), &pool).await,
            ["for", "grant", "revoke"]
        );
        assert_eq!(
            keywords_at(&format!("alter default privileges i{cursor}"), &pool).await,
            ["in"]
        );
        assert_eq!(
            keywords_at(
                &format!("alter default privileges for role admin i{cursor}"),
                &pool
            )
            .await,
            ["in"]
        );
        assert_eq!(
            keywords_at(
                &format!("alter default privileges in schema public {cursor}"),
                &pool
            )
            .await,
            ["for", "grant", "revoke"]
        );
        assert_eq!(
            keywords_at(
                &format!("alter default privileges in schema public for {cursor}"),
                &pool
            )
            .await,
            ["role", "user"]
        );
        // Postgres rejects an option given twice.
        for options in [
            "for role admin in schema public",
            "in schema public for role admin",
        ] {
            assert_eq!(
                keywords_at(
                    &format!("alter default privileges {options} {cursor}"),
                    &pool
                )
                .await,
                ["grant", "revoke"],
                "{options}"
            );
        }
    }

    #[sqlx::test]
    async fn completes_constraint_attributes(pool: PgPool) {
        let cursor = QueryWithCursorPosition::cursor_marker();

        // `not` needs a prefix.
        assert_eq!(
            keywords_at(&format!("alter table t alter constraint c {cursor}"), &pool).await,
            ["deferrable", "enforced", "inherit", "initially", "no"]
        );
        assert_eq!(
            keywords_at(
                &format!("alter table t alter constraint c not {cursor}"),
                &pool
            )
            .await,
            ["deferrable", "enforced"]
        );
        assert_eq!(
            keywords_at(
                &format!("alter table t add constraint c check (a > 0) not {cursor}"),
                &pool
            )
            .await,
            ["deferrable", "enforced", "valid"]
        );
    }

    #[sqlx::test]
    async fn completes_options_after_opening_parenthesis(pool: PgPool) {
        let cursor = QueryWithCursorPosition::cursor_marker();

        // `analyze` needs a prefix.
        let explain = [
            "buffers",
            "costs",
            "format",
            "generic_plan",
            "memory",
            "serializable",
            "settings",
            "summary",
            "timing",
            "verbose",
            "wal",
        ];
        let analyze = ["buffer_usage_limit", "skip_locked", "verbose"];
        // `null` needs a prefix.
        let copy = [
            "default",
            "delimiter",
            "encoding",
            "escape",
            "force_not_null",
            "force_null",
            "force_quote",
            "format",
            "freeze",
            "header",
            "log_verbosity",
            "on_error",
            "quote",
            "reject_limit",
        ];
        let returning_with = ["new", "old"];

        let cases: [(&str, &[&str]); 13] = [
            ("explain ({})", &[explain.as_slice(), &["select"]].concat()),
            ("explain ({}", &explain),
            ("explain (analyze, {})", &explain),
            ("analyze ({})", &analyze),
            ("analyze ({}", &analyze),
            ("analyze (verbose, {})", &analyze),
            ("copy t from stdin with ({})", &copy),
            ("copy t from stdin with ({}", &copy),
            ("copy t to stdout (format csv, {})", &copy),
            ("update t set a = 1 returning with ({})", &returning_with),
            ("update t set a = 1 returning with ({}", &returning_with),
            (
                "update t set a = 1 returning with (old as o, {})",
                &returning_with,
            ),
            ("delete from t returning with ({})", &returning_with),
        ];

        for (sql, expected) in cases {
            let mut expected = expected.to_vec();
            expected.sort_unstable();
            let query = sql.replace("{}", &cursor.to_string());
            assert_eq!(keywords_at(&query, &pool).await, expected, "{sql}");
        }

        // The keyword replaces only what was typed after the comma.
        let query = format!("explain (analyze, ver{cursor})");
        let (tree, snapshot) = get_test_deps(None, query.as_str().into(), &pool).await;
        let params = get_test_params(&tree, &snapshot, query.as_str().into());
        let items = complete(params);
        let verbose = items.iter().find(|item| item.label == "verbose").unwrap();
        assert_eq!(
            verbose.completion_text.as_ref().map(|text| text.range),
            Some(pgls_text_size::TextRange::new(18.into(), 21.into()))
        );
    }

    #[sqlx::test]
    async fn completes_vacuum_options(pool: PgPool) {
        let cursor = QueryWithCursorPosition::cursor_marker();

        // `analyze` needs a prefix.
        let options = [
            "buffer_usage_limit",
            "disable_page_skipping",
            "freeze",
            "full",
            "index_cleanup",
            "only_database_stats",
            "parallel",
            "process_main",
            "process_toast",
            "skip_database_stats",
            "skip_locked",
            "truncate",
            "verbose",
        ];

        for sql in ["vacuum ({})", "vacuum ({}", "vacuum (verbose, {})"] {
            let query = sql.replace("{}", &cursor.to_string());
            assert_eq!(keywords_at(&query, &pool).await, options, "{sql}");
        }

        assert_eq!(
            keywords_at(&format!("vacuum (index_cleanup {cursor})"), &pool).await,
            ["auto", "off", "on"]
        );
        assert_eq!(
            keywords_at(&format!("explain (format {cursor})"), &pool).await,
            ["json", "text", "xml", "yaml"]
        );
        // The options from before the option list.
        assert_eq!(
            keywords_at(&format!("vacuum {cursor}"), &pool).await,
            ["freeze", "full", "verbose"]
        );
    }

    #[sqlx::test]
    async fn completes_keywords_in_create_and_alter_table(pool: PgPool) {
        let cursor = QueryWithCursorPosition::cursor_marker();

        // `not`, `null` and `as` need a prefix.
        let cases: [(&str, &[&str]); 15] = [
            (
                "create table t (id int {})",
                &[
                    "asc",
                    "check",
                    "collate",
                    "comment",
                    "constraint",
                    "default",
                    "desc",
                    "enforced",
                    "generated",
                    "no",
                    "primary",
                    "references",
                    "stored",
                    "unique",
                    "virtual",
                ],
            ),
            ("create table t (id int primary {})", &["key"]),
            ("create table t (id int generated {})", &["always", "by"]),
            ("create table t (id int generated by {})", &["default"]),
            // `as` follows `by default`, not `by`.
            ("create table t (id int generated by a{})", &["default"]),
            ("create table t (id int generated by default a{})", &["as"]),
            (
                "create table t (id int references u (id) on {})",
                &["delete", "update"],
            ),
            (
                "create table t (id int references u (id) on delete {})",
                &["cascade", "no", "restrict", "set"],
            ),
            (
                "create table t (id int, constraint c {})",
                &["check", "foreign", "index", "key", "primary", "unique"],
            ),
            ("create table t (id int, foreign {})", &["key"]),
            (
                "create table t (id int, unique {})",
                &["index", "key", "nulls"],
            ),
            (
                "create table t (id int) {}",
                &["default", "inherits", "partition", "with"],
            ),
            ("alter table t alter column a {}", &["drop", "set", "type"]),
            (
                "alter table t alter column a set {}",
                &["compression", "data", "default", "statistics", "storage"],
            ),
            ("alter table t drop column a {}", &["cascade", "restrict"]),
        ];

        for (sql, expected) in cases {
            let query = sql.replace("{}", &cursor.to_string());
            assert_eq!(keywords_at(&query, &pool).await, expected, "{sql}");
        }
    }

    #[sqlx::test]
    async fn completes_options_after_opening_parenthesis_with_multibyte_text(pool: PgPool) {
        let cursor = QueryWithCursorPosition::cursor_marker();

        // The cursor is a byte offset; a multi-byte char before it must not move the edit.
        for sql in ["explain /*é*/ ({}", "copy café from stdin with ({}"] {
            let query = sql.replace("{}", &cursor.to_string());
            let source = sql.replace("{}", "");
            let (tree, snapshot) = get_test_deps(None, query.as_str().into(), &pool).await;
            let params = get_test_params(&tree, &snapshot, query.as_str().into());
            let items = complete(params);

            assert!(!items.is_empty(), "{sql}");
            for item in &items {
                let range = item.completion_text.as_ref().map(|text| text.range);
                let cursor_range = pgls_text_size::TextRange::empty((source.len() as u32).into());
                assert_eq!(range, Some(cursor_range), "{sql}: {}", item.label);
            }
        }
    }
}
