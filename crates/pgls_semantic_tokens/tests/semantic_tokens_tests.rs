use std::sync::Arc;

use pgls_catalog::resolve::{FunctionContext, FunctionParam};
use pgls_catalog::{Catalog, CatalogBase, Snapshot};
use pgls_semantic_tokens::{SemanticTokenKind, SemanticTokensParams, semantic_tokens};
use sqlx::{Executor, PgPool};

const SETUP: &str = r#"
    create type mood as enum ('happy', 'sad');
    create table users (id int8 primary key, name text, current mood);
    create table posts (id int8 primary key, user_id int8, title text);
    create view active_users as select id, name from users;
    create function make_point(x int, y int) returns int language sql as 'select x + y';
    create function amb(a int, b text) returns int language sql as 'select 1';
    create function amb(c text, d int) returns int language sql as 'select 1';
    create table "cafÉ" (größe int, "Ölstand" int);
"#;

struct Fixture {
    catalog: Catalog,
    search_path: Vec<String>,
}

async fn fixture(test_db: &PgPool) -> Fixture {
    test_db
        .execute(SETUP)
        .await
        .expect("Failed to setup test database");
    let snapshot = Snapshot::load(test_db)
        .await
        .expect("Failed to load snapshot");
    Fixture {
        search_path: snapshot.expand_search_path(&["public".to_string()]),
        catalog: Catalog::new(Some(Arc::new(CatalogBase::new(Arc::new(snapshot))))),
    }
}

impl Fixture {
    /// The statement with each token marked as `[text:kind]`, with a `+` after the kind of
    /// built-in objects.
    fn highlighted(&self, sql: &str) -> String {
        self.highlighted_in(sql, None)
    }

    fn highlighted_in(&self, sql: &str, function: Option<&FunctionContext>) -> String {
        let stmt = pgls_query::parse(sql)
            .expect("Failed to parse")
            .into_root()
            .expect("Expected a statement");

        let tokens = semantic_tokens(SemanticTokensParams {
            stmt: &stmt,
            sql,
            catalog: &self.catalog,
            search_path: &self.search_path,
            function,
        });

        let mut result = sql.to_string();
        for token in tokens.iter().rev() {
            let kind = match token.kind {
                SemanticTokenKind::Namespace => "namespace",
                SemanticTokenKind::Class => "class",
                SemanticTokenKind::Property => "property",
                SemanticTokenKind::Function => "function",
                SemanticTokenKind::Type => "type",
                SemanticTokenKind::Parameter => "parameter",
                SemanticTokenKind::EnumMember => "enumMember",
            };
            let builtin = if token.default_library { "+" } else { "" };
            result.insert_str(
                usize::from(token.range.end()),
                &format!(":{kind}{builtin}]"),
            );
            result.insert(usize::from(token.range.start()), '[');
        }
        result
    }
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn relations_and_columns(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.highlighted("select u.name, title from public.users u join posts on true"),
        "select u.[name:property], [title:property] \
         from [public:namespace].[users:class] u join [posts:class] on true"
    );
    assert_eq!(
        f.highlighted("select users.id from users"),
        "select [users:class].[id:property] from [users:class]"
    );
    assert_eq!(
        f.highlighted("select public.users.id from public.users"),
        "select [public:namespace].[users:class].[id:property] \
         from [public:namespace].[users:class]"
    );
    assert_eq!(
        f.highlighted(r#"select "name" from "active_users""#),
        r#"select ["name":property] from ["active_users":class]"#
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn functions_and_types(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.highlighted("select make_point(1, 2)::int8, pg_catalog.now(), cast(null as text)"),
        "select [make_point:function](1, 2)::[int8:type+], \
         [pg_catalog:namespace+].[now:function+](), cast(null as [text:type+])"
    );
    // Ambiguous and unknown calls have no token.
    assert_eq!(
        f.highlighted("select amb('1', '2'), nope(1)"),
        "select amb('1', '2'), nope(1)"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn types_in_ddl(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.highlighted(
            "create table t (id int8 primary key, m public.mood, n numeric(10, 2), \
             at timestamp with time zone default now(), x nope)"
        ),
        "create table t (id [int8:type+] primary key, m [public:namespace].[mood:type], \
         n [numeric:type+](10, 2), \
         at [timestamp:type+] [with:type+] [time:type+] [zone:type+] default now(), x nope)"
    );
    assert_eq!(
        f.highlighted("alter table users add column c mood, alter column name type varchar(20)"),
        "alter table users add column c [mood:type], alter column name type [varchar:type+](20)"
    );
    assert_eq!(
        f.highlighted(
            "create function f(a mood, out b int) returns setof record language sql as 'select 1'"
        ),
        "create function f(a [mood:type], out b [int:type+]) returns setof [record:type+] \
         language sql as 'select 1'"
    );
    assert_eq!(
        f.highlighted("drop function make_point(int, integer)"),
        "drop function make_point([int:type+], [integer:type+])"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn enum_members(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.highlighted("update users set current = 'sad' where current in ('happy', 'sad')"),
        "update [users:class] set current = ['sad':enumMember] \
         where [current:property] in (['happy':enumMember], ['sad':enumMember])"
    );
    assert_eq!(
        f.highlighted("select 'happy'::mood, 'happy'"),
        "select ['happy':enumMember]::[mood:type], 'happy'"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn function_parameters(test_db: PgPool) {
    let f = fixture(&test_db).await;
    let param = |name: &str| FunctionParam {
        name: Some(name.to_owned()),
        type_schema: None,
        type_name: "int8".to_owned(),
        is_array: false,
    };
    let function = FunctionContext {
        function_name: "lookup".to_owned(),
        params: vec![param("user_id"), param("name")],
    };
    assert_eq!(
        f.highlighted_in(
            "select title from posts where user_id = $1 or id = lookup.user_id",
            Some(&function)
        ),
        "select [title:property] from [posts:class] where [user_id:property] = [$1:parameter] \
         or [id:property] = [lookup:function].[user_id:parameter]"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn comments_inside_qualified_names(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.highlighted("select u.name from public /* c */ . users u"),
        "select u.[name:property] from [public:namespace] /* c */ . [users:class] u"
    );
    assert_eq!(
        f.highlighted("select users -- c\n .id from users"),
        "select [users:class] -- c\n .[id:property] from [users:class]"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn multi_word_types(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.highlighted(
            "select 1::double precision, cast(now() as timestamp(3) with time zone), \
             'a'::character varying(10), 1::int[]"
        ),
        "select 1::[double:type+] [precision:type+], \
         cast([now:function+]() as [timestamp:type+](3) [with:type+] [time:type+] [zone:type+]), \
         'a'::[character:type+] [varying:type+](10), 1::[int:type+][]"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn non_ascii_identifiers(test_db: PgPool) {
    let f = fixture(&test_db).await;
    // Postgres only folds ASCII letters, so `CAFÉ` is `cafÉ`.
    assert_eq!(
        f.highlighted(r#"select CAFÉ.größe, "Ölstand" from CAFÉ"#),
        r#"select [CAFÉ:class].[größe:property], ["Ölstand":property] from [CAFÉ:class]"#
    );
}
