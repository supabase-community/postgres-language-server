//! A formatter only changes whitespace, line breaks, keyword case and redundant quoting. These
//! cases come from formatting a pg_dump-style schema, where it used to rewrite what was written.

use pgls_pretty_print::{FormatConfig, format_statement};

fn format(sql: &str) -> String {
    let config = FormatConfig::default();
    let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let first = format_statement(&ast, sql, &config).unwrap().formatted;

    let ast = pgls_query::parse(&first).unwrap().into_root().unwrap();
    let second = format_statement(&ast, &first, &config).unwrap().formatted;
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
