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
    SqlKeyword::new("abort"),
    SqlKeyword::new("absent"),
    SqlKeyword::new("absolute"),
    SqlKeyword::new("access"),
    SqlKeyword::new("action"),
    SqlKeyword::new("add"),
    SqlKeyword::new("admin"),
    SqlKeyword::new("after"),
    SqlKeyword::new("aggregate"),
    SqlKeyword::new("all"),
    SqlKeyword::new("also"),
    SqlKeyword::new("alter").starts_statement(),
    SqlKeyword::new("always"),
    SqlKeyword::new("analyse"),
    SqlKeyword::new("analyze")
        .starts_statement()
        .require_prefix(),
    SqlKeyword::new("and").require_prefix(),
    SqlKeyword::new("any").require_prefix(),
    SqlKeyword::new("array").require_prefix(),
    SqlKeyword::new("as").require_prefix(),
    SqlKeyword::new("asc"),
    SqlKeyword::new("asensitive"),
    SqlKeyword::new("assertion"),
    SqlKeyword::new("assignment"),
    SqlKeyword::new("asymmetric"),
    SqlKeyword::new("at"),
    SqlKeyword::new("atomic"),
    SqlKeyword::new("attach"),
    SqlKeyword::new("attribute"),
    SqlKeyword::new("authorization"),
    SqlKeyword::new("backward"),
    SqlKeyword::new("before"),
    SqlKeyword::new("begin").starts_statement(),
    SqlKeyword::new("between").require_prefix(),
    SqlKeyword::new("bigint"),
    SqlKeyword::new("bigserial"),
    SqlKeyword::new("binary"),
    SqlKeyword::new("bit"),
    SqlKeyword::new("boolean"),
    SqlKeyword::new("both"),
    SqlKeyword::new("breadth"),
    SqlKeyword::new("brin"),
    SqlKeyword::new("btree"),
    SqlKeyword::new("by"),
    SqlKeyword::new("bytea"),
    SqlKeyword::new("cache"),
    SqlKeyword::new("call"),
    SqlKeyword::new("called"),
    SqlKeyword::new("cascade"),
    SqlKeyword::new("cascaded"),
    SqlKeyword::new("case"),
    SqlKeyword::new("cast").require_prefix(),
    SqlKeyword::new("catalog"),
    SqlKeyword::new("chain"),
    SqlKeyword::new("char"),
    SqlKeyword::new("character"),
    SqlKeyword::new("characteristics"),
    SqlKeyword::new("check"),
    SqlKeyword::new("checkpoint"),
    SqlKeyword::new("class"),
    SqlKeyword::new("close"),
    SqlKeyword::new("cluster"),
    SqlKeyword::new("coalesce"),
    SqlKeyword::new("collate"),
    SqlKeyword::new("collation"),
    SqlKeyword::new("column"),
    SqlKeyword::new("columns"),
    SqlKeyword::new("comment").starts_statement(),
    SqlKeyword::new("comments"),
    SqlKeyword::new("commit").starts_statement(),
    SqlKeyword::new("committed"),
    SqlKeyword::new("compression"),
    SqlKeyword::new("concurrently"),
    SqlKeyword::new("conditional"),
    SqlKeyword::new("configuration"),
    SqlKeyword::new("conflict"),
    SqlKeyword::new("connection"),
    SqlKeyword::new("constraint"),
    SqlKeyword::new("constraints"),
    SqlKeyword::new("content"),
    SqlKeyword::new("continue"),
    SqlKeyword::new("conversion"),
    SqlKeyword::new("copy").starts_statement(),
    SqlKeyword::new("cost"),
    SqlKeyword::new("create").starts_statement(),
    SqlKeyword::new("cross"),
    SqlKeyword::new("csv"),
    SqlKeyword::new("cube"),
    SqlKeyword::new("current"),
    SqlKeyword::new("current_catalog"),
    SqlKeyword::new("current_date"),
    SqlKeyword::new("current_role"),
    SqlKeyword::new("current_schema"),
    SqlKeyword::new("current_time"),
    SqlKeyword::new("current_timestamp"),
    SqlKeyword::new("current_user"),
    SqlKeyword::new("cursor"),
    SqlKeyword::new("cycle"),
    SqlKeyword::new("data"),
    SqlKeyword::new("database"),
    SqlKeyword::new("date"),
    SqlKeyword::new("day"),
    SqlKeyword::new("deallocate"),
    SqlKeyword::new("dec"),
    SqlKeyword::new("decimal"),
    SqlKeyword::new("declare").starts_statement(),
    SqlKeyword::new("default"),
    SqlKeyword::new("defaults"),
    SqlKeyword::new("deferrable"),
    SqlKeyword::new("deferred"),
    SqlKeyword::new("definer"),
    SqlKeyword::new("delete").starts_statement(),
    SqlKeyword::new("delimiter"),
    SqlKeyword::new("delimiters"),
    SqlKeyword::new("depends"),
    SqlKeyword::new("depth"),
    SqlKeyword::new("desc"),
    SqlKeyword::new("detach"),
    SqlKeyword::new("dictionary"),
    SqlKeyword::new("disable"),
    SqlKeyword::new("disable_page_skipping"),
    SqlKeyword::new("discard"),
    SqlKeyword::new("distinct"),
    SqlKeyword::new("do").starts_statement(),
    SqlKeyword::new("document"),
    SqlKeyword::new("domain"),
    SqlKeyword::new("double"),
    SqlKeyword::new("drop").starts_statement(),
    SqlKeyword::new("each"),
    SqlKeyword::new("else"),
    SqlKeyword::new("empty"),
    SqlKeyword::new("enable"),
    SqlKeyword::new("encoding"),
    SqlKeyword::new("encrypted"),
    SqlKeyword::new("end"),
    SqlKeyword::new("enum"),
    SqlKeyword::new("error"),
    SqlKeyword::new("escape"),
    SqlKeyword::new("event"),
    SqlKeyword::new("except"),
    SqlKeyword::new("exclude"),
    SqlKeyword::new("excluding"),
    SqlKeyword::new("exclusive"),
    SqlKeyword::new("execute"),
    SqlKeyword::new("exists").require_prefix(),
    SqlKeyword::new("explain")
        .starts_statement()
        .require_prefix(),
    SqlKeyword::new("expression"),
    SqlKeyword::new("extended"),
    SqlKeyword::new("extension"),
    SqlKeyword::new("external"),
    SqlKeyword::new("extract"),
    SqlKeyword::new("false").require_prefix(),
    SqlKeyword::new("family"),
    SqlKeyword::new("fetch"),
    SqlKeyword::new("filter"),
    SqlKeyword::new("finalize"),
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
    SqlKeyword::new("forward"),
    SqlKeyword::new("freeze"),
    SqlKeyword::new("from"),
    SqlKeyword::new("full"),
    SqlKeyword::new("function"),
    SqlKeyword::new("functions"),
    SqlKeyword::new("generated"),
    SqlKeyword::new("gin"),
    SqlKeyword::new("gist"),
    SqlKeyword::new("global"),
    SqlKeyword::new("grant").starts_statement(),
    SqlKeyword::new("granted"),
    SqlKeyword::new("greatest"),
    SqlKeyword::new("group"),
    SqlKeyword::new("grouping"),
    SqlKeyword::new("groups"),
    SqlKeyword::new("handler"),
    SqlKeyword::new("hash"),
    SqlKeyword::new("having"),
    SqlKeyword::new("header"),
    SqlKeyword::new("hold"),
    SqlKeyword::new("hour"),
    SqlKeyword::new("identity"),
    SqlKeyword::new("if"),
    SqlKeyword::new("ilike"),
    SqlKeyword::new("immediate"),
    SqlKeyword::new("immutable"),
    SqlKeyword::new("implicit"),
    SqlKeyword::new("import"),
    SqlKeyword::new("in").require_prefix(),
    SqlKeyword::new("include"),
    SqlKeyword::new("including"),
    SqlKeyword::new("increment"),
    SqlKeyword::new("indent"),
    SqlKeyword::new("index"),
    SqlKeyword::new("index_cleanup"),
    SqlKeyword::new("indexes"),
    SqlKeyword::new("inet"),
    SqlKeyword::new("inherit"),
    SqlKeyword::new("inherits"),
    SqlKeyword::new("initially"),
    SqlKeyword::new("inline"),
    SqlKeyword::new("inner"),
    SqlKeyword::new("inout"),
    SqlKeyword::new("input"),
    SqlKeyword::new("insensitive"),
    SqlKeyword::new("insert").starts_statement(),
    SqlKeyword::new("instead"),
    SqlKeyword::new("int"),
    SqlKeyword::new("integer"),
    SqlKeyword::new("intersect"),
    SqlKeyword::new("interval").require_prefix(),
    SqlKeyword::new("into"),
    SqlKeyword::new("invoker"),
    SqlKeyword::new("is").require_prefix(),
    SqlKeyword::new("isnull"),
    SqlKeyword::new("isolation"),
    SqlKeyword::new("join"),
    SqlKeyword::new("json"),
    SqlKeyword::new("json_array"),
    SqlKeyword::new("json_arrayagg"),
    SqlKeyword::new("json_exists"),
    SqlKeyword::new("json_object"),
    SqlKeyword::new("json_objectagg"),
    SqlKeyword::new("json_query"),
    SqlKeyword::new("json_scalar"),
    SqlKeyword::new("json_serialize"),
    SqlKeyword::new("json_table"),
    SqlKeyword::new("json_value"),
    SqlKeyword::new("jsonb"),
    SqlKeyword::new("keep"),
    SqlKeyword::new("key"),
    SqlKeyword::new("keys"),
    SqlKeyword::new("label"),
    SqlKeyword::new("language"),
    SqlKeyword::new("large"),
    SqlKeyword::new("last"),
    SqlKeyword::new("lateral"),
    SqlKeyword::new("leading"),
    SqlKeyword::new("leakproof"),
    SqlKeyword::new("least"),
    SqlKeyword::new("left"),
    SqlKeyword::new("level"),
    SqlKeyword::new("like").require_prefix(),
    SqlKeyword::new("limit"),
    SqlKeyword::new("list"),
    SqlKeyword::new("listen"),
    SqlKeyword::new("load"),
    SqlKeyword::new("local"),
    SqlKeyword::new("localtime"),
    SqlKeyword::new("localtimestamp"),
    SqlKeyword::new("location"),
    SqlKeyword::new("lock"),
    SqlKeyword::new("locked"),
    SqlKeyword::new("logged"),
    SqlKeyword::new("main"),
    SqlKeyword::new("maintain"),
    SqlKeyword::new("mapping"),
    SqlKeyword::new("match"),
    SqlKeyword::new("matched"),
    SqlKeyword::new("materialized"),
    SqlKeyword::new("maxvalue"),
    SqlKeyword::new("merge").starts_statement(),
    SqlKeyword::new("merge_action"),
    SqlKeyword::new("method"),
    SqlKeyword::new("minute"),
    SqlKeyword::new("minvalue"),
    SqlKeyword::new("mode"),
    SqlKeyword::new("money"),
    SqlKeyword::new("month"),
    SqlKeyword::new("move"),
    SqlKeyword::new("name"),
    SqlKeyword::new("names"),
    SqlKeyword::new("national"),
    SqlKeyword::new("natural"),
    SqlKeyword::new("nchar"),
    SqlKeyword::new("nested"),
    SqlKeyword::new("new"),
    SqlKeyword::new("next"),
    SqlKeyword::new("nfc"),
    SqlKeyword::new("nfd"),
    SqlKeyword::new("nfkc"),
    SqlKeyword::new("nfkd"),
    SqlKeyword::new("no"),
    SqlKeyword::new("none"),
    SqlKeyword::new("normalize"),
    SqlKeyword::new("normalized"),
    SqlKeyword::new("not").require_prefix(),
    SqlKeyword::new("nothing"),
    SqlKeyword::new("notify"),
    SqlKeyword::new("notnull"),
    SqlKeyword::new("nowait"),
    SqlKeyword::new("null").require_prefix(),
    SqlKeyword::new("nullif"),
    SqlKeyword::new("nulls"),
    SqlKeyword::new("numeric"),
    SqlKeyword::new("object"),
    SqlKeyword::new("of"),
    SqlKeyword::new("off"),
    SqlKeyword::new("offset"),
    SqlKeyword::new("oid"),
    SqlKeyword::new("oids"),
    SqlKeyword::new("old"),
    SqlKeyword::new("omit"),
    SqlKeyword::new("on"),
    SqlKeyword::new("only").require_prefix(),
    SqlKeyword::new("operator"),
    SqlKeyword::new("option"),
    SqlKeyword::new("options"),
    SqlKeyword::new("or").require_prefix(),
    SqlKeyword::new("order"),
    SqlKeyword::new("ordinality"),
    SqlKeyword::new("others"),
    SqlKeyword::new("out"),
    SqlKeyword::new("outer"),
    SqlKeyword::new("over"),
    SqlKeyword::new("overlaps"),
    SqlKeyword::new("overlay"),
    SqlKeyword::new("overriding"),
    SqlKeyword::new("owned"),
    SqlKeyword::new("owner"),
    SqlKeyword::new("parallel"),
    SqlKeyword::new("parameter"),
    SqlKeyword::new("parser"),
    SqlKeyword::new("partial"),
    SqlKeyword::new("partition"),
    SqlKeyword::new("partitioned"),
    SqlKeyword::new("passing"),
    SqlKeyword::new("password"),
    SqlKeyword::new("path"),
    SqlKeyword::new("permissive"),
    SqlKeyword::new("placing"),
    SqlKeyword::new("plain"),
    SqlKeyword::new("plan"),
    SqlKeyword::new("plans"),
    SqlKeyword::new("policy"),
    SqlKeyword::new("position"),
    SqlKeyword::new("precedes"),
    SqlKeyword::new("preceding"),
    SqlKeyword::new("precision"),
    SqlKeyword::new("prepare"),
    SqlKeyword::new("prepared"),
    SqlKeyword::new("preserve"),
    SqlKeyword::new("primary"),
    SqlKeyword::new("prior"),
    SqlKeyword::new("privileges"),
    SqlKeyword::new("procedural"),
    SqlKeyword::new("procedure"),
    SqlKeyword::new("procedures"),
    SqlKeyword::new("process_toast"),
    SqlKeyword::new("program"),
    SqlKeyword::new("public"),
    SqlKeyword::new("publication"),
    SqlKeyword::new("quote"),
    SqlKeyword::new("quotes"),
    SqlKeyword::new("range"),
    SqlKeyword::new("read"),
    SqlKeyword::new("real"),
    SqlKeyword::new("reassign"),
    SqlKeyword::new("recheck"),
    SqlKeyword::new("recursive"),
    SqlKeyword::new("ref"),
    SqlKeyword::new("references"),
    SqlKeyword::new("referencing"),
    SqlKeyword::new("refresh"),
    SqlKeyword::new("regclass"),
    SqlKeyword::new("regnamespace"),
    SqlKeyword::new("regproc"),
    SqlKeyword::new("regtype"),
    SqlKeyword::new("reindex"),
    SqlKeyword::new("relative"),
    SqlKeyword::new("release"),
    SqlKeyword::new("rename"),
    SqlKeyword::new("repeatable"),
    SqlKeyword::new("replace"),
    SqlKeyword::new("replica"),
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
    SqlKeyword::new("rollup"),
    SqlKeyword::new("routine"),
    SqlKeyword::new("routines"),
    SqlKeyword::new("row"),
    SqlKeyword::new("rows"),
    SqlKeyword::new("rule"),
    SqlKeyword::new("safe"),
    SqlKeyword::new("savepoint"),
    SqlKeyword::new("scalar"),
    SqlKeyword::new("schema"),
    SqlKeyword::new("schemas"),
    SqlKeyword::new("scroll"),
    SqlKeyword::new("search"),
    SqlKeyword::new("second"),
    SqlKeyword::new("security"),
    SqlKeyword::new("select").starts_statement(),
    SqlKeyword::new("sequence"),
    SqlKeyword::new("sequences"),
    SqlKeyword::new("serial"),
    SqlKeyword::new("serializable"),
    SqlKeyword::new("server"),
    SqlKeyword::new("session"),
    SqlKeyword::new("session_user"),
    SqlKeyword::new("set").starts_statement(),
    SqlKeyword::new("setof"),
    SqlKeyword::new("sets"),
    SqlKeyword::new("share"),
    SqlKeyword::new("show").starts_statement(),
    SqlKeyword::new("similar").require_prefix(),
    SqlKeyword::new("simple"),
    SqlKeyword::new("skip"),
    SqlKeyword::new("skip_locked"),
    SqlKeyword::new("smallint"),
    SqlKeyword::new("smallserial"),
    SqlKeyword::new("snapshot"),
    SqlKeyword::new("some"),
    SqlKeyword::new("source"),
    SqlKeyword::new("spgist"),
    SqlKeyword::new("sql"),
    SqlKeyword::new("stable"),
    SqlKeyword::new("standalone"),
    SqlKeyword::new("start"),
    SqlKeyword::new("statement"),
    SqlKeyword::new("statistics"),
    SqlKeyword::new("stdin"),
    SqlKeyword::new("stdout"),
    SqlKeyword::new("storage"),
    SqlKeyword::new("stored"),
    SqlKeyword::new("strict"),
    SqlKeyword::new("string"),
    SqlKeyword::new("strip"),
    SqlKeyword::new("subscription"),
    SqlKeyword::new("substring"),
    SqlKeyword::new("support"),
    SqlKeyword::new("symmetric"),
    SqlKeyword::new("sysid"),
    SqlKeyword::new("system"),
    SqlKeyword::new("system_user"),
    SqlKeyword::new("table"),
    SqlKeyword::new("tables"),
    SqlKeyword::new("tablesample"),
    SqlKeyword::new("tablespace"),
    SqlKeyword::new("target"),
    SqlKeyword::new("temp"),
    SqlKeyword::new("template"),
    SqlKeyword::new("temporary"),
    SqlKeyword::new("text"),
    SqlKeyword::new("then").require_prefix(),
    SqlKeyword::new("ties"),
    SqlKeyword::new("time"),
    SqlKeyword::new("timestamp"),
    SqlKeyword::new("timestamptz"),
    SqlKeyword::new("to"),
    SqlKeyword::new("trailing"),
    SqlKeyword::new("transaction"),
    SqlKeyword::new("transform"),
    SqlKeyword::new("treat"),
    SqlKeyword::new("trigger"),
    SqlKeyword::new("trim"),
    SqlKeyword::new("true").require_prefix(),
    SqlKeyword::new("truncate").starts_statement(),
    SqlKeyword::new("trusted"),
    SqlKeyword::new("type"),
    SqlKeyword::new("types"),
    SqlKeyword::new("uescape"),
    SqlKeyword::new("unbounded"),
    SqlKeyword::new("uncommitted"),
    SqlKeyword::new("unconditional"),
    SqlKeyword::new("unencrypted"),
    SqlKeyword::new("union"),
    SqlKeyword::new("unique"),
    SqlKeyword::new("unknown"),
    SqlKeyword::new("unlisten"),
    SqlKeyword::new("unlogged"),
    SqlKeyword::new("unsafe"),
    SqlKeyword::new("until"),
    SqlKeyword::new("update").starts_statement(),
    SqlKeyword::new("user"),
    SqlKeyword::new("using"),
    SqlKeyword::new("uuid"),
    SqlKeyword::new("vacuum").starts_statement(),
    SqlKeyword::new("valid"),
    SqlKeyword::new("validate"),
    SqlKeyword::new("validator"),
    SqlKeyword::new("value"),
    SqlKeyword::new("values"),
    SqlKeyword::new("varchar"),
    SqlKeyword::new("variadic"),
    SqlKeyword::new("varying"),
    SqlKeyword::new("verbose"),
    SqlKeyword::new("version"),
    SqlKeyword::new("view"),
    SqlKeyword::new("views"),
    SqlKeyword::new("volatile"),
    SqlKeyword::new("when"),
    SqlKeyword::new("where"),
    SqlKeyword::new("whitespace"),
    SqlKeyword::new("window"),
    SqlKeyword::new("with").starts_statement(),
    SqlKeyword::new("within"),
    SqlKeyword::new("without"),
    SqlKeyword::new("work"),
    SqlKeyword::new("wrapper"),
    SqlKeyword::new("write"),
    SqlKeyword::new("xml"),
    SqlKeyword::new("xmlattributes"),
    SqlKeyword::new("xmlconcat"),
    SqlKeyword::new("xmlelement"),
    SqlKeyword::new("xmlexists"),
    SqlKeyword::new("xmlforest"),
    SqlKeyword::new("xmlnamespaces"),
    SqlKeyword::new("xmlparse"),
    SqlKeyword::new("xmlpi"),
    SqlKeyword::new("xmlroot"),
    SqlKeyword::new("xmlserialize"),
    SqlKeyword::new("xmltable"),
    SqlKeyword::new("year"),
    SqlKeyword::new("yes"),
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
    use std::{collections::BTreeSet, path::Path};

    use pgls_query::protobuf::KeywordKind;
    use pgls_test_utils::QueryWithCursorPosition;
    use prost_reflect::DescriptorPool;
    use sqlx::PgPool;

    use crate::{
        CompletionItemKind,
        providers::keywords::ALL_KEYWORDS,
        test_helper::{
            CompletionAssertion, TestCompletionsCase, TestCompletionsSuite,
            assert_complete_results, assert_no_complete_results,
        },
    };

    #[test]
    fn has_all_keywords_from_pg_query_proto() {
        let proto_file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pgls_query/vendor/libpg_query/protobuf/pg_query.proto");
        let include_path = proto_file
            .parent()
            .expect("pg_query.proto should have a parent directory");
        let file_name = proto_file
            .file_name()
            .expect("pg_query.proto should have a file name");

        let descriptor_set =
            protox::compile([file_name], [include_path]).expect("failed to parse pg_query.proto");
        let pool = DescriptorPool::from_file_descriptor_set(descriptor_set)
            .expect("failed to load protobuf descriptor pool");
        let token_enum = pool
            .get_enum_by_name(".pg_query.Token")
            .expect(".pg_query.Token enum should exist");

        let mut expected_keywords = BTreeSet::new();

        for value in token_enum.values() {
            for candidate in keyword_candidates(value.name()) {
                if is_sql_keyword(&candidate) {
                    expected_keywords.insert(candidate);
                    break;
                }
            }
        }

        let actual_keywords = ALL_KEYWORDS
            .iter()
            .map(|keyword| keyword.name.to_string())
            .collect::<BTreeSet<_>>();

        let missing_keywords = expected_keywords
            .difference(&actual_keywords)
            .cloned()
            .collect::<Vec<_>>();

        assert!(
            missing_keywords.is_empty(),
            "Found {} keyword(s) derived from pg_query.proto that are missing from ALL_KEYWORDS.\n\
             Add missing entries to ALL_KEYWORDS in crates/pgls_completions/src/providers/keywords.rs.\n\
             Missing keywords:\n{}",
            missing_keywords.len(),
            missing_keywords.join("\n")
        );
    }

    fn keyword_candidates(enum_value_name: &str) -> Vec<String> {
        let mut candidates = Vec::with_capacity(3);

        candidates.push(enum_value_name.to_ascii_lowercase());

        if let Some(without_suffix) = enum_value_name.strip_suffix("_P") {
            candidates.push(without_suffix.to_ascii_lowercase());
        }

        if let Some(without_suffix) = enum_value_name.strip_suffix("_LA") {
            candidates.push(without_suffix.to_ascii_lowercase());
        }

        candidates
    }

    fn is_sql_keyword(candidate: &str) -> bool {
        let Ok(scan_result) = pgls_query::scan(candidate) else {
            return false;
        };
        let Some(token) = scan_result.tokens.first() else {
            return false;
        };
        let Ok(keyword_kind) = KeywordKind::try_from(token.keyword_kind) else {
            return false;
        };

        keyword_kind != KeywordKind::NoKeyword
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
                    "copy".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "create".into(),
                    crate::CompletionItemKind::Keyword,
                ),
                CompletionAssertion::LabelAndKind(
                    "drop".into(),
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
}
