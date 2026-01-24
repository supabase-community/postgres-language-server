use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tree_sitter::{InputEdit, Point, StreamingIterator};

pub fn criterion_benchmark(c: &mut Criterion) {
    // takes about 3 microseconds on MacBook Pro M2 with 16GB Memory
    c.bench_function("parsing with existing tree > small sql", |b| {
        let content = format!("select * from users;");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(black_box(content.clone()), None).unwrap();

        let query = tree_sitter::Query::new(
            &pgls_treesitter_grammar::LANGUAGE.into(),
            r#"
          (relation
            (table_reference
              (any_identifier) @tbl
            )
          )
        "#,
        )
        .expect("Invalid TS Query");

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        let tbl_token = matches
            .next()
            .expect("invalid TS query for the SQL")
            .captures[0]
            .node;

        let token_to_replace = black_box("clients");

        let mut shared_tree = tree.clone();

        b.iter(|| {
            let start_position = tbl_token.start_position();
            let start_byte = tbl_token.start_byte();
            let old_end_position = tbl_token.end_position();
            let old_end_byte = tbl_token.end_byte();

            let new_end_byte = start_byte + token_to_replace.len();
            let new_end_position = Point {
                row: start_position.row,
                column: start_position.column + token_to_replace.len(),
            };

            shared_tree.edit(&InputEdit {
                new_end_byte,
                new_end_position,
                old_end_byte,
                old_end_position,
                start_byte,
                start_position,
            });

            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
                .unwrap();

            let replaced_sql = format!(
                "{}{}{}",
                &content[..start_byte],
                token_to_replace,
                &content[old_end_byte..]
            );

            let changed_tree = parser
                .parse(black_box(replaced_sql), Some(&shared_tree))
                .unwrap();
            black_box(changed_tree);

            shared_tree.edit(&InputEdit {
                new_end_byte: old_end_byte,
                new_end_position: old_end_position,
                old_end_byte: new_end_byte,
                old_end_position: new_end_position,
                start_byte,
                start_position,
            });
        });
    });

    // takes about 12 microseconds on MacBook Pro M2 with 16GB Memory
    c.bench_function("parsing with existing tree > mid sql", |b| {
        let content = format!(
            r#"
select
  n.oid :: int8 as "id!",
  n.nspname as name,
  u.rolname as "owner!"
from
  pg_namespace n,
        something
where
  n.nspowner = u.oid
  and (
    pg_has_role(n.nspowner, 'USAGE')
    or has_schema_privilege(n.oid, 'CREATE, USAGE')
  )
  and not pg_catalog.starts_with(n.nspname, 'pg_temp_')
  and not pg_catalog.starts_with(n.nspname, 'pg_toast_temp_');
"#
        );

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(black_box(content.clone()), None).unwrap();

        let query = tree_sitter::Query::new(
            &pgls_treesitter_grammar::LANGUAGE.into(),
            r#"
          (keyword_or) @token
        "#,
        )
        .expect("Invalid TS Query");

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        let tbl_token = matches
            .next()
            .expect("invalid TS query for the SQL")
            .captures[0]
            .node;

        let token_to_replace = black_box("and");

        let mut shared_tree = tree.clone();

        b.iter(|| {
            let start_position = tbl_token.start_position();
            let start_byte = tbl_token.start_byte();
            let old_end_position = tbl_token.end_position();
            let old_end_byte = tbl_token.end_byte();

            let new_end_byte = start_byte + token_to_replace.len();
            let new_end_position = Point {
                row: start_position.row,
                column: start_position.column + token_to_replace.len(),
            };

            shared_tree.edit(&InputEdit {
                new_end_byte,
                new_end_position,
                old_end_byte,
                old_end_position,
                start_byte,
                start_position,
            });

            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
                .unwrap();

            let replaced_sql = format!(
                "{}{}{}",
                &content[..start_byte],
                token_to_replace,
                &content[old_end_byte..]
            );

            let changed_tree = parser
                .parse(black_box(replaced_sql), Some(&shared_tree))
                .unwrap();
            black_box(changed_tree);

            shared_tree.edit(&InputEdit {
                new_end_byte: old_end_byte,
                new_end_position: old_end_position,
                old_end_byte: new_end_byte,
                old_end_position: new_end_position,
                start_byte,
                start_position,
            });
        });
    });

    // takes about 12 microseconds on MacBook Pro M2 with 16GB Memory
    c.bench_function("parsing with existing tree > large sql", |b| {
        let content = format!(
            r#"
  with available_tables as (
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
        indices
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

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(black_box(content.clone()), None).unwrap();

        let query = tree_sitter::Query::new(
            &pgls_treesitter_grammar::LANGUAGE.into(),
            r#"
          (keyword_in) @token
        "#,
        )
        .expect("Invalid TS Query");

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        let tbl_token = matches
            .next()
            .expect("invalid TS query for the SQL")
            .captures[0]
            .node;

        let token_to_replace = black_box("not in");

        let mut shared_tree = tree.clone();

        b.iter(|| {
            let start_position = tbl_token.start_position();
            let start_byte = tbl_token.start_byte();
            let old_end_position = tbl_token.end_position();
            let old_end_byte = tbl_token.end_byte();

            let new_end_byte = start_byte + token_to_replace.len();
            let new_end_position = Point {
                row: start_position.row,
                column: start_position.column + token_to_replace.len(),
            };

            shared_tree.edit(&InputEdit {
                new_end_byte,
                new_end_position,
                old_end_byte,
                old_end_position,
                start_byte,
                start_position,
            });

            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
                .unwrap();

            let replaced_sql = format!(
                "{}{}{}",
                &content[..start_byte],
                token_to_replace,
                &content[old_end_byte..]
            );

            let changed_tree = parser
                .parse(black_box(replaced_sql), Some(&shared_tree))
                .unwrap();
            black_box(changed_tree);

            shared_tree.edit(&InputEdit {
                new_end_byte: old_end_byte,
                new_end_position: old_end_position,
                old_end_byte: new_end_byte,
                old_end_position: new_end_position,
                start_byte,
                start_position,
            });
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
