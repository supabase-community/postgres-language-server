create table gin_test_tbl(i int4[]) with (autovacuum_enabled = off);

create index gin_test_idx on gin_test_tbl using gin (i)
  with (fastupdate = on, gin_pending_list_limit = 4096);

insert into gin_test_tbl select array[1, 2, g] from generate_series(1, 20000) g;

insert into gin_test_tbl select array[1, 3, g] from generate_series(1, 1000) g;

select gin_clean_pending_list('gin_test_idx')>10 as many;

insert into gin_test_tbl select array[3, 1, g] from generate_series(1, 1000) g;

vacuum gin_test_tbl;

select gin_clean_pending_list('gin_test_idx');

delete from gin_test_tbl where i @> array[2];

vacuum gin_test_tbl;

alter index gin_test_idx set (fastupdate = off);

insert into gin_test_tbl select array[1, 2, g] from generate_series(1, 1000) g;

insert into gin_test_tbl select array[1, 3, g] from generate_series(1, 1000) g;

delete from gin_test_tbl where i @> array[2];

vacuum gin_test_tbl;

select count(*) from gin_test_tbl where i @> array[1, 999];

select count(*) from gin_test_tbl where i @> array[1, 999];

set gin_fuzzy_search_limit = 1000;

select count(*) > 0 as ok from gin_test_tbl where i @> array[1];

select count(*) > 0 as ok from gin_test_tbl where i @> array[1];

reset gin_fuzzy_search_limit;

create temp table t_gin_test_tbl(i int4[], j int4[]);

create index on t_gin_test_tbl using gin (i, j);

insert into t_gin_test_tbl
values
  (null,    null),
  ('{}',    null),
  ('{1}',   null),
  ('{1,2}', null),
  (null,    '{}'),
  (null,    '{10}'),
  ('{1,2}', '{10}'),
  ('{2}',   '{10}'),
  ('{1,3}', '{}'),
  ('{1,1}', '{10}');

set enable_seqscan = off;

select * from t_gin_test_tbl where array[0] <@ i;

select * from t_gin_test_tbl where array[0] <@ i;

select * from t_gin_test_tbl where array[0] <@ i and '{}'::int4[] <@ j;

select * from t_gin_test_tbl where i @> '{}';

select * from t_gin_test_tbl where i @> '{}';

create function explain_query_json(query_sql text)
returns table (explain_line json)
language plpgsql as
$$
begin
  set enable_seqscan = off;
  set enable_bitmapscan = on;
  return query execute 'EXPLAIN (ANALYZE, FORMAT json) ' || query_sql;
end;
$$;

create function execute_text_query_index(query_sql text)
returns setof text
language plpgsql
as
$$
begin
  set enable_seqscan = off;
  set enable_bitmapscan = on;
  return query execute query_sql;
end;
$$;

create function execute_text_query_heap(query_sql text)
returns setof text
language plpgsql
as
$$
begin
  set enable_seqscan = on;
  set enable_bitmapscan = off;
  return query execute query_sql;
end;
$$;

select
  query,
  js->0->'Plan'->'Plans'->0->'Actual Rows' as "return by index",
  js->0->'Plan'->'Rows Removed by Index Recheck' as "removed by recheck",
  (res_index = res_heap) as "match"
from
  (values
    ($$ i @> '{}' $$),
    ($$ j @> '{}' $$),
    ($$ i @> '{}' and j @> '{}' $$),
    ($$ i @> '{1}' $$),
    ($$ i @> '{1}' and j @> '{}' $$),
    ($$ i @> '{1}' and i @> '{}' and j @> '{}' $$),
    ($$ j @> '{10}' $$),
    ($$ j @> '{10}' and i @> '{}' $$),
    ($$ j @> '{10}' and j @> '{}' and i @> '{}' $$),
    ($$ i @> '{1}' and j @> '{10}' $$)
  ) q(query),
  lateral explain_query_json($$select * from t_gin_test_tbl where $$ || query) js,
  lateral execute_text_query_index($$select string_agg((i, j)::text, ' ') from t_gin_test_tbl where $$ || query) res_index,
  lateral execute_text_query_heap($$select string_agg((i, j)::text, ' ') from t_gin_test_tbl where $$ || query) res_heap;

reset enable_seqscan;

reset enable_bitmapscan;

insert into t_gin_test_tbl select array[1, g, g/10], array[2, g, g/10]
  from generate_series(1, 20000) g;

select gin_clean_pending_list('t_gin_test_tbl_i_j_idx') is not null;

analyze t_gin_test_tbl;

set enable_seqscan = off;

set enable_bitmapscan = on;

select count(*) from t_gin_test_tbl where j @> array[50];

select count(*) from t_gin_test_tbl where j @> array[50];

select count(*) from t_gin_test_tbl where j @> array[2];

select count(*) from t_gin_test_tbl where j @> array[2];

select count(*) from t_gin_test_tbl where j @> '{}'::int[];

select count(*) from t_gin_test_tbl where j @> '{}'::int[];

delete from t_gin_test_tbl where j @> array[2];

vacuum t_gin_test_tbl;

select count(*) from t_gin_test_tbl where j @> array[50];

select count(*) from t_gin_test_tbl where j @> array[2];

select count(*) from t_gin_test_tbl where j @> '{}'::int[];

reset enable_seqscan;

reset enable_bitmapscan;

drop table t_gin_test_tbl;

create unlogged table t_gin_test_tbl(i int4[], j int4[]);

create index on t_gin_test_tbl using gin (i, j);

insert into t_gin_test_tbl
values
  (null,    null),
  ('{}',    null),
  ('{1}',   '{2,3}');

drop table t_gin_test_tbl;
