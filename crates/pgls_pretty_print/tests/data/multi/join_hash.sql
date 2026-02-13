begin;

set local min_parallel_table_scan_size = 0;

set local parallel_setup_cost = 0;

set local enable_hashjoin = on;

create or replace function find_hash(node json)
returns json language plpgsql
as
$$
declare
  x json;
  child json;
begin
  if node->>'Node Type' = 'Hash' then
    return node;
  else
    for child in select json_array_elements(node->'Plans')
    loop
      x := find_hash(child);
      if x is not null then
        return x;
      end if;
    end loop;
    return null;
  end if;
end;
$$;

create or replace function hash_join_batches(query text)
returns table (original int, final int) language plpgsql
as
$$
declare
  whole_plan json;
  hash_node json;
begin
  for whole_plan in
    execute 'explain (analyze, format ''json'') ' || query
  loop
    hash_node := find_hash(json_extract_path(whole_plan, '0', 'Plan'));
    original := hash_node->>'Original Hash Batches';
    final := hash_node->>'Hash Batches';
    return next;
  end loop;
end;
$$;

create table simple as
  select generate_series(1, 20000) AS id, 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';

alter table simple set (parallel_workers = 2);

analyze simple;

create table bigger_than_it_looks as
  select generate_series(1, 20000) as id, 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';

alter table bigger_than_it_looks set (autovacuum_enabled = 'false');

alter table bigger_than_it_looks set (parallel_workers = 2);

analyze bigger_than_it_looks;

update pg_class set reltuples = 1000 where relname = 'bigger_than_it_looks';

create table extremely_skewed (id int, t text);

alter table extremely_skewed set (autovacuum_enabled = 'false');

alter table extremely_skewed set (parallel_workers = 2);

analyze extremely_skewed;

insert into extremely_skewed
  select 42 as id, 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
  from generate_series(1, 20000);

update pg_class
  set reltuples = 2, relpages = pg_relation_size('extremely_skewed') / 8192
  where relname = 'extremely_skewed';

create table wide as select generate_series(1, 2) as id, rpad('', 320000, 'x') as t;

alter table wide set (parallel_workers = 2);

savepoint settings;

set local max_parallel_workers_per_gather = 0;

set local work_mem = '4MB';

set local hash_mem_multiplier = 1.0;

select count(*) from simple r join simple s using (id);

select count(*) from simple r join simple s using (id);

select original > 1 as initially_multibatch, final > original as increased_batches
  from hash_join_batches(
$$
  select count(*) from simple r join simple s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 2;

set local work_mem = '4MB';

set local hash_mem_multiplier = 1.0;

set local enable_parallel_hash = off;

select count(*) from simple r join simple s using (id);

select count(*) from simple r join simple s using (id);

select original > 1 as initially_multibatch, final > original as increased_batches
  from hash_join_batches(
$$
  select count(*) from simple r join simple s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 2;

set local work_mem = '4MB';

set local hash_mem_multiplier = 1.0;

set local enable_parallel_hash = on;

select count(*) from simple r join simple s using (id);

select count(*) from simple r join simple s using (id);

select original > 1 as initially_multibatch, final > original as increased_batches
  from hash_join_batches(
$$
  select count(*) from simple r join simple s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 0;

set local work_mem = '128kB';

set local hash_mem_multiplier = 1.0;

select count(*) from simple r join simple s using (id);

select count(*) from simple r join simple s using (id);

select original > 1 as initially_multibatch, final > original as increased_batches
  from hash_join_batches(
$$
  select count(*) from simple r join simple s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 2;

set local work_mem = '128kB';

set local hash_mem_multiplier = 1.0;

set local enable_parallel_hash = off;

select count(*) from simple r join simple s using (id);

select count(*) from simple r join simple s using (id);

select original > 1 as initially_multibatch, final > original as increased_batches
  from hash_join_batches(
$$
  select count(*) from simple r join simple s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 2;

set local work_mem = '192kB';

set local hash_mem_multiplier = 1.0;

set local enable_parallel_hash = on;

select count(*) from simple r join simple s using (id);

select count(*) from simple r join simple s using (id);

select original > 1 as initially_multibatch, final > original as increased_batches
  from hash_join_batches(
$$
  select count(*) from simple r join simple s using (id);
$$);

select count(*) from simple r full outer join simple s using (id);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 0;

set local work_mem = '128kB';

set local hash_mem_multiplier = 1.0;

select count(*) FROM simple r JOIN bigger_than_it_looks s USING (id);

select count(*) FROM simple r JOIN bigger_than_it_looks s USING (id);

select original > 1 as initially_multibatch, final > original as increased_batches
  from hash_join_batches(
$$
  select count(*) FROM simple r JOIN bigger_than_it_looks s USING (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 2;

set local work_mem = '128kB';

set local hash_mem_multiplier = 1.0;

set local enable_parallel_hash = off;

select count(*) from simple r join bigger_than_it_looks s using (id);

select count(*) from simple r join bigger_than_it_looks s using (id);

select original > 1 as initially_multibatch, final > original as increased_batches
  from hash_join_batches(
$$
  select count(*) from simple r join bigger_than_it_looks s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 1;

set local work_mem = '192kB';

set local hash_mem_multiplier = 1.0;

set local enable_parallel_hash = on;

select count(*) from simple r join bigger_than_it_looks s using (id);

select count(*) from simple r join bigger_than_it_looks s using (id);

select original > 1 as initially_multibatch, final > original as increased_batches
  from hash_join_batches(
$$
  select count(*) from simple r join bigger_than_it_looks s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 0;

set local work_mem = '128kB';

set local hash_mem_multiplier = 1.0;

select count(*) from simple r join extremely_skewed s using (id);

select count(*) from simple r join extremely_skewed s using (id);

select * from hash_join_batches(
$$
  select count(*) from simple r join extremely_skewed s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 2;

set local work_mem = '128kB';

set local hash_mem_multiplier = 1.0;

set local enable_parallel_hash = off;

select count(*) from simple r join extremely_skewed s using (id);

select count(*) from simple r join extremely_skewed s using (id);

select * from hash_join_batches(
$$
  select count(*) from simple r join extremely_skewed s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 1;

set local work_mem = '128kB';

set local hash_mem_multiplier = 1.0;

set local enable_parallel_hash = on;

select count(*) from simple r join extremely_skewed s using (id);

select count(*) from simple r join extremely_skewed s using (id);

select * from hash_join_batches(
$$
  select count(*) from simple r join extremely_skewed s using (id);
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 2;

set local work_mem = '4MB';

set local hash_mem_multiplier = 1.0;

set local parallel_leader_participation = off;

select * from hash_join_batches(
$$
  select count(*) from simple r join simple s using (id);
$$);

rollback to settings;

create table join_foo as select generate_series(1, 3) as id, 'xxxxx'::text as t;

alter table join_foo set (parallel_workers = 0);

create table join_bar as select generate_series(1, 10000) as id, 'xxxxx'::text as t;

alter table join_bar set (parallel_workers = 2);

savepoint settings;

set enable_parallel_hash = off;

set parallel_leader_participation = off;

set min_parallel_table_scan_size = 0;

set parallel_setup_cost = 0;

set parallel_tuple_cost = 0;

set max_parallel_workers_per_gather = 2;

set enable_material = off;

set enable_mergejoin = off;

set work_mem = '64kB';

set hash_mem_multiplier = 1.0;

select count(*) from join_foo
    left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
    on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;

select count(*) from join_foo
  left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
  on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;

select final > 1 as multibatch
  from hash_join_batches(
$$
  select count(*) from join_foo
    left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
    on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;
$$);

rollback to settings;

savepoint settings;

set enable_parallel_hash = off;

set parallel_leader_participation = off;

set min_parallel_table_scan_size = 0;

set parallel_setup_cost = 0;

set parallel_tuple_cost = 0;

set max_parallel_workers_per_gather = 2;

set enable_material = off;

set enable_mergejoin = off;

set work_mem = '4MB';

set hash_mem_multiplier = 1.0;

select count(*) from join_foo
    left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
    on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;

select count(*) from join_foo
  left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
  on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;

select final > 1 as multibatch
  from hash_join_batches(
$$
  select count(*) from join_foo
    left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
    on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;
$$);

rollback to settings;

savepoint settings;

set enable_parallel_hash = on;

set parallel_leader_participation = off;

set min_parallel_table_scan_size = 0;

set parallel_setup_cost = 0;

set parallel_tuple_cost = 0;

set max_parallel_workers_per_gather = 2;

set enable_material = off;

set enable_mergejoin = off;

set work_mem = '64kB';

set hash_mem_multiplier = 1.0;

select count(*) from join_foo
    left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
    on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;

select count(*) from join_foo
  left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
  on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;

select final > 1 as multibatch
  from hash_join_batches(
$$
  select count(*) from join_foo
    left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
    on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;
$$);

rollback to settings;

savepoint settings;

set enable_parallel_hash = on;

set parallel_leader_participation = off;

set min_parallel_table_scan_size = 0;

set parallel_setup_cost = 0;

set parallel_tuple_cost = 0;

set max_parallel_workers_per_gather = 2;

set enable_material = off;

set enable_mergejoin = off;

set work_mem = '4MB';

set hash_mem_multiplier = 1.0;

select count(*) from join_foo
    left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
    on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;

select count(*) from join_foo
  left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
  on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;

select final > 1 as multibatch
  from hash_join_batches(
$$
  select count(*) from join_foo
    left join (select b1.id, b1.t from join_bar b1 join join_bar b2 using (id)) ss
    on join_foo.id < ss.id + 1 and join_foo.id > ss.id - 1;
$$);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 0;

select  count(*) from simple r full outer join simple s using (id);

select  count(*) from simple r full outer join simple s using (id);

rollback to settings;

savepoint settings;

set enable_parallel_hash = off;

set local max_parallel_workers_per_gather = 2;

select  count(*) from simple r full outer join simple s using (id);

select  count(*) from simple r full outer join simple s using (id);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 2;

select  count(*) from simple r full outer join simple s using (id);

select  count(*) from simple r full outer join simple s using (id);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 0;

select  count(*) from simple r full outer join simple s on (r.id = 0 - s.id);

select  count(*) from simple r full outer join simple s on (r.id = 0 - s.id);

rollback to settings;

savepoint settings;

set enable_parallel_hash = off;

set local max_parallel_workers_per_gather = 2;

select  count(*) from simple r full outer join simple s on (r.id = 0 - s.id);

select  count(*) from simple r full outer join simple s on (r.id = 0 - s.id);

rollback to settings;

savepoint settings;

set local max_parallel_workers_per_gather = 2;

select  count(*) from simple r full outer join simple s on (r.id = 0 - s.id);

select  count(*) from simple r full outer join simple s on (r.id = 0 - s.id);

rollback to settings;

savepoint settings;

set max_parallel_workers_per_gather = 2;

set enable_parallel_hash = on;

set work_mem = '128kB';

set hash_mem_multiplier = 1.0;

select length(max(s.t))
  from wide left join (select id, coalesce(t, '') || '' as t from wide) s using (id);

select length(max(s.t))
from wide left join (select id, coalesce(t, '') || '' as t from wide) s using (id);

select final > 1 as multibatch
  from hash_join_batches(
$$
  select length(max(s.t))
  from wide left join (select id, coalesce(t, '') || '' as t from wide) s using (id);
$$);

rollback to settings;

SAVEPOINT settings;

SET enable_parallel_hash = on;

SET min_parallel_table_scan_size = 0;

SET parallel_setup_cost = 0;

SET parallel_tuple_cost = 0;

CREATE TABLE hjtest_matchbits_t1(id int);

CREATE TABLE hjtest_matchbits_t2(id int);

INSERT INTO hjtest_matchbits_t1 VALUES (1);

INSERT INTO hjtest_matchbits_t2 VALUES (2);

UPDATE hjtest_matchbits_t2 set id = 2;

SELECT * FROM hjtest_matchbits_t1 t1 FULL JOIN hjtest_matchbits_t2 t2 ON t1.id = t2.id
  ORDER BY t1.id;

RESET parallel_setup_cost;

SET enable_parallel_hash = off;

SELECT * FROM hjtest_matchbits_t1 t1 FULL JOIN hjtest_matchbits_t2 t2 ON t1.id = t2.id;

ROLLBACK TO settings;

rollback;

BEGIN;

SET LOCAL enable_sort = OFF;

SET LOCAL from_collapse_limit = 1;

CREATE TABLE hjtest_1 (a text, b int, id int, c bool);

CREATE TABLE hjtest_2 (a bool, id int, b text, c int);

INSERT INTO hjtest_1(a, b, id, c) VALUES ('text', 2, 1, false);

INSERT INTO hjtest_1(a, b, id, c) VALUES ('text', 1, 2, false);

INSERT INTO hjtest_1(a, b, id, c) VALUES ('text', 20, 1, false);

INSERT INTO hjtest_1(a, b, id, c) VALUES ('text', 1, 1, false);

INSERT INTO hjtest_2(a, id, b, c) VALUES (true, 1, 'another', 2);

INSERT INTO hjtest_2(a, id, b, c) VALUES (true, 3, 'another', 7);

INSERT INTO hjtest_2(a, id, b, c) VALUES (true, 1, 'another', 90);

INSERT INTO hjtest_2(a, id, b, c) VALUES (true, 1, 'another', 3);

INSERT INTO hjtest_2(a, id, b, c) VALUES (true, 1, 'text', 1);

SELECT hjtest_1.a a1, hjtest_2.a a2,hjtest_1.tableoid::regclass t1, hjtest_2.tableoid::regclass t2
FROM hjtest_1, hjtest_2
WHERE
    hjtest_1.id = (SELECT 1 WHERE hjtest_2.id = 1)
    AND (SELECT hjtest_1.b * 5) = (SELECT hjtest_2.c*5)
    AND (SELECT hjtest_1.b * 5) < 50
    AND (SELECT hjtest_2.c * 5) < 55
    AND hjtest_1.a <> hjtest_2.b;

SELECT hjtest_1.a a1, hjtest_2.a a2,hjtest_1.tableoid::regclass t1, hjtest_2.tableoid::regclass t2
FROM hjtest_1, hjtest_2
WHERE
    hjtest_1.id = (SELECT 1 WHERE hjtest_2.id = 1)
    AND (SELECT hjtest_1.b * 5) = (SELECT hjtest_2.c*5)
    AND (SELECT hjtest_1.b * 5) < 50
    AND (SELECT hjtest_2.c * 5) < 55
    AND hjtest_1.a <> hjtest_2.b;

SELECT hjtest_1.a a1, hjtest_2.a a2,hjtest_1.tableoid::regclass t1, hjtest_2.tableoid::regclass t2
FROM hjtest_2, hjtest_1
WHERE
    hjtest_1.id = (SELECT 1 WHERE hjtest_2.id = 1)
    AND (SELECT hjtest_1.b * 5) = (SELECT hjtest_2.c*5)
    AND (SELECT hjtest_1.b * 5) < 50
    AND (SELECT hjtest_2.c * 5) < 55
    AND hjtest_1.a <> hjtest_2.b;

SELECT hjtest_1.a a1, hjtest_2.a a2,hjtest_1.tableoid::regclass t1, hjtest_2.tableoid::regclass t2
FROM hjtest_2, hjtest_1
WHERE
    hjtest_1.id = (SELECT 1 WHERE hjtest_2.id = 1)
    AND (SELECT hjtest_1.b * 5) = (SELECT hjtest_2.c*5)
    AND (SELECT hjtest_1.b * 5) < 50
    AND (SELECT hjtest_2.c * 5) < 55
    AND hjtest_1.a <> hjtest_2.b;

ROLLBACK;

begin;

set local enable_hashjoin = on;

select i8.q2, ss.* from
int8_tbl i8,
lateral (select t1.fivethous, i4.f1 from tenk1 t1 join int4_tbl i4
         on t1.fivethous = i4.f1+i8.q2 order by 1,2) ss;

select i8.q2, ss.* from
int8_tbl i8,
lateral (select t1.fivethous, i4.f1 from tenk1 t1 join int4_tbl i4
         on t1.fivethous = i4.f1+i8.q2 order by 1,2) ss;

rollback;
