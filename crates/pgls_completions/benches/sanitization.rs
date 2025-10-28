use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pgls_completions::{CompletionParams, benchmark_sanitization};
use pgls_schema_cache::SchemaCache;
use pgls_text_size::TextSize;

static CURSOR_POS: &str = "€";

fn sql_and_pos(sql: &str) -> (String, usize) {
    let pos = sql.find(CURSOR_POS).unwrap();
    (sql.replace(CURSOR_POS, ""), pos)
}

fn get_tree(sql: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .unwrap();
    parser.parse(sql, None).unwrap()
}

fn to_params<'a>(
    text: String,
    tree: &'a tree_sitter::Tree,
    pos: usize,
    cache: &'a SchemaCache,
) -> CompletionParams<'a> {
    let pos: u32 = pos.try_into().unwrap();
    CompletionParams {
        position: TextSize::new(pos),
        schema: cache,
        text,
        tree,
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("small sql, adjusted", |b| {
        let content = format!("select {CURSOR_POS} from users;");

        let cache = SchemaCache::default();
        let (sql, pos) = sql_and_pos(content.as_str());
        let tree = get_tree(sql.as_str());

        b.iter(|| benchmark_sanitization(black_box(to_params(sql.clone(), &tree, pos, &cache))));
    });

    c.bench_function("mid sql, adjusted", |b| {
        let content = format!(
            r#"select
  n.oid :: int8 as "id!",
  n.nspname as name,
  u.rolname as "owner!"
from
  pg_namespace n,
        {CURSOR_POS}
where
  n.nspowner = u.oid
  and (
    pg_has_role(n.nspowner, 'USAGE')
    or has_schema_privilege(n.oid, 'CREATE, USAGE')
  )
  and not pg_catalog.starts_with(n.nspname, 'pg_temp_')
  and not pg_catalog.starts_with(n.nspname, 'pg_toast_temp_');"#
        );

        let cache = SchemaCache::default();
        let (sql, pos) = sql_and_pos(content.as_str());
        let tree = get_tree(sql.as_str());

        b.iter(|| benchmark_sanitization(black_box(to_params(sql.clone(), &tree, pos, &cache))));
    });

    c.bench_function("large sql, adjusted", |b| {
        let content = format!(
            r#"with
  available_tables as (
    select
      c.relname as table_name,
      c.oid as table_oid,
      c.relkind as class_kind,
      n.nspname as schema_name
    from
      pg_catalog.pg_class c
      join pg_catalog.pg_namespace n on n.oid = c.relnamespace
    where
      -- r: normal tables
      -- v: views
      -- m: materialized views
      -- f: foreign tables
      -- p: partitioned tables
      c.relkind in ('r', 'v', 'm', 'f', 'p')
  ),
  available_indexes as (
    select
      unnest (ix.indkey) as attnum,
      ix.indisprimary as is_primary,
      ix.indisunique as is_unique,
      ix.indrelid as table_oid
    from
        {CURSOR_POS}
    where
      c.relkind = 'i'
  )
select
  atts.attname as name,
  ts.table_name,
  ts.table_oid :: int8 as "table_oid!",
  ts.class_kind :: char as "class_kind!",
  ts.schema_name,
  atts.atttypid :: int8 as "type_id!",
  not atts.attnotnull as "is_nullable!",
  nullif(
    information_schema._pg_char_max_length (atts.atttypid, atts.atttypmod),
    -1
  ) as varchar_length,
  pg_get_expr (def.adbin, def.adrelid) as default_expr,
  coalesce(ix.is_primary, false) as "is_primary_key!",
  coalesce(ix.is_unique, false) as "is_unique!",
  pg_catalog.col_description (ts.table_oid, atts.attnum) as comment
from
  pg_catalog.pg_attribute atts
  join available_tables ts on atts.attrelid = ts.table_oid
  left join available_indexes ix on atts.attrelid = ix.table_oid
  and atts.attnum = ix.attnum
  left join pg_catalog.pg_attrdef def on atts.attrelid = def.adrelid
  and atts.attnum = def.adnum
where
  -- system columns, such as `cmax` or `tableoid`, have negative `attnum`s
  atts.attnum >= 0;
"#
        );

        let cache = SchemaCache::default();
        let (sql, pos) = sql_and_pos(content.as_str());
        let tree = get_tree(sql.as_str());

        b.iter(|| benchmark_sanitization(black_box(to_params(sql.clone(), &tree, pos, &cache))));
    });

    c.bench_function("small sql, unadjusted", |b| {
        let content = format!("select e{CURSOR_POS} from users;");

        let cache = SchemaCache::default();
        let (sql, pos) = sql_and_pos(content.as_str());
        let tree = get_tree(sql.as_str());

        b.iter(|| benchmark_sanitization(black_box(to_params(sql.clone(), &tree, pos, &cache))));
    });

    c.bench_function("mid sql, unadjusted", |b| {
        let content = format!(
            r#"select
  n.oid :: int8 as "id!",
  n.nspname as name,
  u.rolname as "owner!"
from
  pg_namespace n,
  pg_r{CURSOR_POS}
where
  n.nspowner = u.oid
  and (
    pg_has_role(n.nspowner, 'USAGE')
    or has_schema_privilege(n.oid, 'CREATE, USAGE')
  )
  and not pg_catalog.starts_with(n.nspname, 'pg_temp_')
  and not pg_catalog.starts_with(n.nspname, 'pg_toast_temp_');"#
        );

        let cache = SchemaCache::default();
        let (sql, pos) = sql_and_pos(content.as_str());
        let tree = get_tree(sql.as_str());

        b.iter(|| benchmark_sanitization(black_box(to_params(sql.clone(), &tree, pos, &cache))));
    });

    c.bench_function("large sql, unadjusted", |b| {
        let content = format!(
            r#"with
  available_tables as (
    select
      c.relname as table_name,
      c.oid as table_oid,
      c.relkind as class_kind,
      n.nspname as schema_name
    from
      pg_catalog.pg_class c
      join pg_catalog.pg_namespace n on n.oid = c.relnamespace
    where
      -- r: normal tables
      -- v: views
      -- m: materialized views
      -- f: foreign tables
      -- p: partitioned tables
      c.relkind in ('r', 'v', 'm', 'f', 'p')
  ),
  available_indexes as (
    select
      unnest (ix.indkey) as attnum,
      ix.indisprimary as is_primary,
      ix.indisunique as is_unique,
      ix.indrelid as table_oid
    from
      pg_catalog.pg_class c
      join pg_catalog.pg_index ix on c.oid = ix.indexrelid
    where
      c.relkind = 'i'
  )
select
  atts.attname as name,
  ts.table_name,
  ts.table_oid :: int8 as "table_oid!",
  ts.class_kind :: char as "class_kind!",
  ts.schema_name,
  atts.atttypid :: int8 as "type_id!",
  not atts.attnotnull as "is_nullable!",
  nullif(
    information_schema._pg_char_max_length (atts.atttypid, atts.atttypmod),
    -1
  ) as varchar_length,
  pg_get_expr (def.adbin, def.adrelid) as default_expr,
  coalesce(ix.is_primary, false) as "is_primary_key!",
  coalesce(ix.is_unique, false) as "is_unique!",
  pg_catalog.col_description (ts.table_oid, atts.attnum) as comment
from
  pg_catalog.pg_attribute atts
  join available_tables ts on atts.attrelid = ts.table_oid
  left join available_indexes ix on atts.attrelid = ix.table_oid
  and atts.attnum = ix.attnum
  left join pg_catalog.pg_attrdef def on atts.attrelid = def.adrelid
  and atts.attnum = def.adnum
where
  -- system columns, such as `cmax` or `tableoid`, have negative `attnum`s
  atts.attnum >= 0
order by
  sch{CURSOR_POS} "#
        );

        let cache = SchemaCache::default();
        let (sql, pos) = sql_and_pos(content.as_str());
        let tree = get_tree(sql.as_str());

        b.iter(|| benchmark_sanitization(black_box(to_params(sql.clone(), &tree, pos, &cache))));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
