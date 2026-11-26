//! A formatter only changes whitespace, line breaks, keyword case and redundant quoting. These
//! cases come from formatting a pg_dump-style schema, where it used to rewrite what was written.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use pgls_pretty_print::{FormatConfig, FormatStatementParams, format_statement};

fn format(sql: &str) -> String {
    let config = FormatConfig::default();
    let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let first = format_statement(FormatStatementParams::new(&ast, sql, &config))
        .unwrap()
        .formatted;

    let ast = pgls_query::parse(&first).unwrap().into_root().unwrap();
    let second = format_statement(FormatStatementParams::new(&ast, &first, &config))
        .unwrap()
        .formatted;
    assert_eq!(second, first, "formatting is not idempotent");
    first
}

#[test]
fn pg_catalog_qualification_is_kept() {
    assert_eq!(
        format(
            r#"CREATE TABLE "public"."t" ("id" "uuid" DEFAULT pg_catalog.gen_random_uuid() NOT NULL, "n" "text" DEFAULT pg_catalog.lower('X'));"#
        ),
        "create table public.t (
  id uuid default pg_catalog.gen_random_uuid() not null,
  n text default pg_catalog.lower('X')
);"
    );
}

#[test]
fn pg_catalog_qualification_of_a_type_is_kept() {
    assert_eq!(
        format("SELECT CAST(x AS pg_catalog.text), CAST(y AS pg_catalog.int4), CAST(z AS int4)"),
        "select cast(x as pg_catalog.text), cast(y as int), cast(z as int4);"
    );
}

#[test]
fn sql_standard_syntax_stays_unqualified() {
    assert_eq!(
        format(
            "SELECT EXTRACT(year FROM d), SUBSTRING(s FROM 2 FOR 3), TRIM(BOTH 'x' FROM s), \
             POSITION('a' IN s), (ts AT TIME ZONE 'UTC'), COLLATION FOR (s) FROM t"
        ),
        "select
  extract(year from d),
  substring(s from 2 for 3),
  trim('x' from s),
  position('a' in s),
  (ts at time zone 'UTC'),
  collation for (s)
from
  t;"
    );
}

#[test]
fn function_names_keep_their_case_and_quotes() {
    assert_eq!(
        format(r#"SELECT "now"(), NOW(), "MyFunc"(1), "coalesce"(a), public."select"(b)"#),
        r#"select now(), now(), "MyFunc"(1), "coalesce"(a), public.select(b);"#
    );
}

#[test]
fn function_bodies_are_kept_byte_for_byte() {
    assert_eq!(
        format(
            r#"CREATE OR REPLACE FUNCTION "public"."reset_user_password"("p_user_id" "uuid", "p_password" "text") RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO ''
    AS $$
BEGIN
  RETURN true;
END
$$;"#
        ),
        "create or replace function public.reset_user_password(p_user_id uuid, p_password text)
returns boolean
language plpgsql
security definer
set search_path to ''
as $$
BEGIN
  RETURN true;
END
$$;"
    );
}

#[test]
fn every_body_is_dollar_quoted_the_same_way() {
    assert_eq!(
        format(
            "CREATE FUNCTION f() RETURNS int LANGUAGE plpgsql AS $function$ begin return 1; end $function$"
        ),
        "create function f()
returns int
language plpgsql
as $$ begin return 1; end $$;"
    );
    assert_eq!(
        format("CREATE FUNCTION f() RETURNS text LANGUAGE sql AS 'SELECT ''a''::text'"),
        "create function f()
returns text
language sql
as $$SELECT 'a'::text$$;"
    );
    assert_eq!(
        format("DO $do$ BEGIN PERFORM 1; END $do$"),
        "do $$ BEGIN PERFORM 1; END $$;"
    );
}

#[test]
fn a_body_containing_the_default_tag_gets_another_one() {
    assert_eq!(
        format("DO $outer$ BEGIN EXECUTE $$SELECT 1$$; END $outer$"),
        "do $body$ BEGIN EXECUTE $$SELECT 1$$; END $body$;"
    );
}

#[test]
fn signatures_keep_argument_names_and_modes() {
    assert_eq!(
        format(
            r#"ALTER FUNCTION "public"."reset_user_password"("p_user_id" "uuid", "p_password" "text") OWNER TO "postgres";"#
        ),
        "alter function public.reset_user_password(p_user_id uuid, p_password text) owner to postgres;"
    );
    assert_eq!(
        format(
            r#"REVOKE ALL ON FUNCTION "public"."reset_user_password"("p_user_id" "uuid", "p_password" "text") FROM PUBLIC;"#
        ),
        "revoke all on function public.reset_user_password(p_user_id uuid, p_password text) from public;"
    );
    assert_eq!(
        format(
            r#"GRANT EXECUTE ON FUNCTION "public"."reset_user_password"("p_user_id" "uuid", "p_password" "text") TO "retool";"#
        ),
        "grant execute on function public.reset_user_password(p_user_id uuid, p_password text) to retool;"
    );
    assert_eq!(
        format("DROP FUNCTION f(IN a int, OUT b text, VARIADIC c int[])"),
        "drop function f(in a int, out b text, variadic c int[]);"
    );
}

#[test]
fn privileges_follow_the_keyword_case() {
    assert_eq!(
        format("GRANT SELECT, INSERT, USAGE, TRUNCATE ON TABLE t TO r"),
        "grant select, insert, usage, truncate on table t to r;"
    );
}

#[test]
fn constraint_and_index_names_lose_redundant_quotes_only() {
    assert_eq!(
        format(
            r#"ALTER TABLE ONLY "public"."organisation_vocabulary" ADD CONSTRAINT "organisation_vocabulary_pkey" PRIMARY KEY ("id");"#
        ),
        "alter table only public.organisation_vocabulary
  add constraint organisation_vocabulary_pkey primary key (id);"
    );
    assert_eq!(
        format(r#"CREATE INDEX "Idx_Mixed" ON "user" ("select", "Name")"#),
        r#"create index "Idx_Mixed" on "user" using btree ("select", "Name");"#
    );
}

#[test]
fn column_definitions_stay_on_one_line_when_they_fit() {
    assert_eq!(
        format(
            r#"CREATE TABLE IF NOT EXISTS "public"."organisation_vocabulary" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "organisation_id" "uuid" DEFAULT "private"."current_organisation_id"() NOT NULL,
    "value" "extensions"."citext" NOT NULL,
    "pronunciations" "text"[],
    "language" "text" NOT NULL,
    "intensity" numeric DEFAULT 0.5 NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "updated_at" timestamp with time zone DEFAULT "now"() NOT NULL
);"#
        ),
        "create table if not exists public.organisation_vocabulary (
  id uuid default gen_random_uuid() not null,
  organisation_id uuid default private.current_organisation_id() not null,
  value extensions.citext not null,
  pronunciations text[],
  language text not null,
  intensity numeric default 0.5 not null,
  created_at timestamp with time zone default now() not null,
  updated_at timestamp with time zone default now() not null
);"
    );
}

#[test]
fn a_broken_column_definition_continues_one_level_deeper() {
    assert_eq!(
        format(
            "CREATE TABLE t (organisation_id uuid DEFAULT private.current_organisation_id_of_the_signed_in_user_or_the_default() NOT NULL)"
        ),
        "create table t (
  organisation_id uuid
    default private.current_organisation_id_of_the_signed_in_user_or_the_default()
    not null
);"
    );
}

#[test]
fn constraint_columns_break_inside_their_parentheses() {
    assert_eq!(
        format(
            r#"ALTER TABLE ONLY "public"."organisation_vocabulary"
    ADD CONSTRAINT "organisation_vocabulary_org_value_lang_unique" UNIQUE ("organisation_id", "value", "language");"#
        ),
        "alter table only public.organisation_vocabulary
  add constraint organisation_vocabulary_org_value_lang_unique unique (
    organisation_id,
    value,
    language
  );"
    );
    assert_eq!(
        format(
            r#"ALTER TABLE ONLY "public"."vocabulary" ADD CONSTRAINT "vocabulary_unique" UNIQUE ("organisation_id", "value", "language");"#
        ),
        "alter table only public.vocabulary
  add constraint vocabulary_unique unique (organisation_id, value, language);"
    );
}

#[test]
fn create_index_keeps_its_head_opclass_and_short_predicate_together() {
    assert_eq!(
        format(
            r#"CREATE INDEX "idx_contact_org_search" ON "public"."contact" USING gin (
  "organisation_id" extensions.uuid_ops,
  ( public.search_text("display_name", "full_name", "username", "email", "sms", "whatsapp", "instagram",
      "facebook", "google_business_messaging", "postal", "external_id", "secondary_external_id", "description") ) extensions.gin_trgm_ops
)
WHERE "deleted_at" IS NULL;"#
        ),
        "create index idx_contact_org_search on public.contact using gin (
  organisation_id extensions.uuid_ops,
  (public.search_text(
    display_name,
    full_name,
    username,
    email,
    sms,
    whatsapp,
    instagram,
    facebook,
    google_business_messaging,
    postal,
    external_id,
    secondary_external_id,
    description
  )) extensions.gin_trgm_ops
)
where deleted_at is null;"
    );
    assert_eq!(
        format(r#"CREATE INDEX "idx_t_a" ON "public"."t" USING btree ("a") WHERE "b" IS NULL;"#),
        "create index idx_t_a on public.t using btree (a) where b is null;"
    );
}

#[test]
fn end_of_line_comments_stay_on_their_line() {
    assert_eq!(
        format(
            r#"create table "public"."t" (
  "id" "uuid" not null, -- trailing comment on column
  /* block comment */ "n" "text"
);"#
        ),
        "create table public.t (
  id uuid not null, -- trailing comment on column
  /* block comment */ n text
);"
    );
    assert_eq!(
        format("select f(a, b), -- after a call\n  c as x, -- after an alias\n  d from t"),
        "select
  f(a, b), -- after a call
  c as x, -- after an alias
  d
from
  t;"
    );
}

#[test]
fn set_values_keep_their_case() {
    assert_eq!(
        format("SET search_path TO 'MySchema', public"),
        "set search_path to 'MySchema', public;"
    );
    assert_eq!(
        format("SET datestyle = 'German'"),
        "set datestyle = 'German';"
    );
}

/// `with admin true` parses to the same AST, but only Postgres 16 and newer accept it.
#[test]
fn grant_role_with_admin_option_keeps_the_portable_spelling() {
    assert_eq!(
        format("GRANT r1 TO u1 WITH ADMIN OPTION"),
        "grant r1 to u1 with admin option;"
    );
    assert_eq!(
        format("GRANT r1 TO u1 WITH ADMIN TRUE"),
        "grant r1 to u1 with admin option;"
    );
    // Only Postgres 16 and newer have the other options, so they keep the boolean spelling.
    assert_eq!(
        format("GRANT r1 TO u1 WITH INHERIT FALSE, ADMIN FALSE, SET TRUE"),
        "grant r1 to u1 with inherit false, admin false, set true;"
    );
}

/// Postgres keeps the first 63 bytes of an identifier, and so does the AST. The formatter writes
/// the name as it was written, not as the parser cut it down.
#[test]
fn identifiers_longer_than_postgres_keeps_are_written_in_full() {
    let role = "a_role_name_that_is_definitely_way_too_long_for_anything_postgres_keeps";
    assert_eq!(
        format(&format!("GRANT SELECT ON t TO {role}")),
        format!("grant select on table t to {role};")
    );

    let column = "A Quoted Column Name That Is Definitely Way Too Long For Anything Postgres Keeps";
    assert_eq!(
        format(&format!("SELECT \"{column}\" FROM t")),
        format!("select \"{column}\" from t;")
    );

    let table = "an_unquoted_table_name_that_is_definitely_way_too_long_for_anything";
    assert_eq!(
        format(&format!("CREATE TABLE {table} (id int)")),
        format!("create table {table} (id int);")
    );
}

// Found by the format round-trip oracle of the corpus check (`pgls_cli/tests/corpus.rs`): the
// output didn't parse, or parsed to a different statement.

#[test]
fn a_unique_index_includes_before_nulls_not_distinct() {
    assert_eq!(
        format("CREATE UNIQUE INDEX i ON ONLY t USING btree (a) INCLUDE (b) NULLS NOT DISTINCT"),
        "create unique index i on only t using btree (a) include (b) nulls not distinct;"
    );
}

#[test]
fn an_identity_keeps_its_sequence_name() {
    assert_eq!(
        format(
            "ALTER TABLE t ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (SEQUENCE NAME public.t_id_seq START WITH 1)"
        ),
        "alter table t
  alter column id
  add generated always as identity (sequence name public.t_id_seq start with 1);"
    );
}

#[test]
fn qualified_object_names_are_dotted() {
    assert_eq!(
        format("ALTER EXTENSION hstore DROP TABLE public.ref_table"),
        "alter extension hstore drop table public.ref_table;"
    );
    assert_eq!(
        format("SECURITY LABEL FOR anon ON COLUMN t.c IS 'l'"),
        "security label for anon on column t.c is 'l';"
    );
}

#[test]
fn depends_on_extension_names_its_object() {
    assert_eq!(
        format("ALTER INDEX i NO DEPENDS ON EXTENSION e"),
        "alter index i no depends on extension e;"
    );
    assert_eq!(
        format("ALTER MATERIALIZED VIEW v DEPENDS ON EXTENSION e"),
        "alter materialized view v depends on extension e;"
    );
    assert_eq!(
        format("ALTER TRIGGER tg ON t DEPENDS ON EXTENSION e"),
        "alter trigger tg on t depends on extension e;"
    );
}

#[test]
fn a_qualified_operator_is_wrapped_in_operator() {
    assert_eq!(
        format("ALTER OPERATOR === (box, box) SET (NEGATOR = OPERATOR(s.!==))"),
        "alter operator === (box, box) set (negator = operator(s.!==));"
    );
    assert_eq!(
        format(
            "CREATE OPERATOR s.=== (FUNCTION = s.f, LEFTARG = int, COMMUTATOR = OPERATOR(s.===))"
        ),
        "create operator s.=== (function = s.f, leftarg = int, commutator = operator(s.===));"
    );
    assert_eq!(
        format(r#"SELECT a OPERATOR("My Schema".===) b"#),
        r#"select a operator("My Schema".===) b;"#
    );
}

#[test]
fn a_publication_of_the_current_schema() {
    assert_eq!(
        format("ALTER PUBLICATION p SET TABLES IN SCHEMA CURRENT_SCHEMA"),
        "alter publication p set tables in schema current_schema;"
    );
}

#[test]
fn alter_role_all_keeps_all() {
    assert_eq!(
        format("ALTER ROLE ALL RESET ALL"),
        "alter role all reset all;"
    );
    assert_eq!(
        format("ALTER USER ALL IN DATABASE d RESET x"),
        "alter role all in database d reset x;"
    );
}

#[test]
fn a_server_version_can_be_null() {
    assert_eq!(
        format("ALTER SERVER s VERSION NULL"),
        "alter server s version null;"
    );
}

#[test]
fn a_constraint_keeps_its_index_parameters() {
    assert_eq!(
        format(
            "CREATE TABLE t (a int UNIQUE WITH (fillfactor = 70), b int PRIMARY KEY WITH (fillfactor = 70) USING INDEX TABLESPACE s)"
        ),
        "create table t (
  a int unique with (fillfactor = 70),
  b int primary key with (fillfactor = 70) using index tablespace s
);"
    );
    assert_eq!(
        format(
            "CREATE TABLE t (r tsrange, EXCLUDE USING gist (r WITH &&) WITH (fillfactor = 70) USING INDEX TABLESPACE s WHERE (r IS NOT NULL))"
        ),
        "create table t (
  r tsrange,
  exclude using gist (r with &&)
    with (fillfactor = 70)
    using index tablespace s where (r is not null)
);"
    );
}

#[test]
fn cluster_keeps_its_options() {
    assert_eq!(
        format("CLUSTER (VERBOSE OFF) t USING i"),
        "cluster (verbose off) t using i;"
    );
}

#[test]
fn comment_on_a_cast_or_operator_class() {
    assert_eq!(
        format("COMMENT ON CAST (text AS int4) IS 'c'"),
        "comment on cast (text as int4) is 'c';"
    );
    assert_eq!(
        format("COMMENT ON OPERATOR CLASS s.c USING btree IS 'c'"),
        "comment on operator class s.c using btree is 'c';"
    );
    assert_eq!(
        format("COMMENT ON OPERATOR FAMILY f USING btree IS 'f'"),
        "comment on operator family f using btree is 'f';"
    );
}

#[test]
fn create_database_in_the_default_tablespace() {
    assert_eq!(
        format("CREATE DATABASE db TABLESPACE DEFAULT"),
        "create database db with tablespace = default;"
    );
}

#[test]
fn a_foreign_table_keeps_if_not_exists_inherits_and_column_options() {
    assert_eq!(
        format("CREATE FOREIGN TABLE IF NOT EXISTS t (a int) INHERITS (p) SERVER s"),
        "create foreign table if not exists t (a int) inherits (p) server s;"
    );
    assert_eq!(
        format(r#"CREATE FOREIGN TABLE t (a char(5) OPTIONS (a 'foo') COLLATE "C") SERVER s"#),
        r#"create foreign table t (a char(5) options (a 'foo') collate "C") server s;"#
    );
}

#[test]
fn column_storage_comes_before_compression() {
    assert_eq!(
        format("CREATE TABLE t (a text STORAGE MAIN COMPRESSION lz4)"),
        "create table t (a text storage main compression lz4);"
    );
}

#[test]
fn trigger_update_columns_follow_update() {
    assert_eq!(
        format(
            "CREATE TRIGGER tg BEFORE UPDATE OF a, b OR TRUNCATE ON t FOR EACH STATEMENT EXECUTE FUNCTION f()"
        ),
        "create trigger tg before update of a, b or truncate on t for each statement execute function f();"
    );
}

#[test]
fn an_enum_without_values() {
    assert_eq!(
        format("CREATE TYPE e AS ENUM ()"),
        "create type e as enum ();"
    );
}

#[test]
fn drop_transform() {
    assert_eq!(
        format("DROP TRANSFORM IF EXISTS FOR s.hstore LANGUAGE plpython3u CASCADE"),
        "drop transform if exists for s.hstore language plpython3u cascade;"
    );
}

#[test]
fn grant_on_a_parameter() {
    assert_eq!(
        format("GRANT SET, ALTER SYSTEM ON PARAMETER work_mem, plpgsql.extra_warnings TO u"),
        r#"grant set, alter system on parameter work_mem, "plpgsql.extra_warnings" to u;"#
    );
}
