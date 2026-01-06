CREATE TABLE unknowntab (
	u unknown    -- fail
);

CREATE TYPE unknown_comptype AS (
	u unknown    -- fail
);

CREATE TABLE tas_case WITH ("Fillfactor" = 10) AS SELECT 1 a;

CREATE UNLOGGED TABLE unlogged1 (a int primary key);

CREATE TEMPORARY TABLE unlogged2 (a int primary key);

SELECT relname, relkind, relpersistence FROM pg_class WHERE relname ~ '^unlogged\d' ORDER BY relname;

REINDEX INDEX unlogged1_pkey;

REINDEX INDEX unlogged2_pkey;

SELECT relname, relkind, relpersistence FROM pg_class WHERE relname ~ '^unlogged\d' ORDER BY relname;

DROP TABLE unlogged2;

INSERT INTO unlogged1 VALUES (42);

CREATE UNLOGGED TABLE public.unlogged2 (a int primary key);

CREATE UNLOGGED TABLE pg_temp.unlogged3 (a int primary key);

CREATE TABLE pg_temp.implicitly_temp (a int primary key);

CREATE TEMP TABLE explicitly_temp (a int primary key);

CREATE TEMP TABLE pg_temp.doubly_temp (a int primary key);

CREATE TEMP TABLE public.temp_to_perm (a int primary key);

DROP TABLE unlogged1, public.unlogged2;

CREATE UNLOGGED TABLE unlogged1 (a int) PARTITION BY RANGE (a);

CREATE TABLE unlogged1 (a int) PARTITION BY RANGE (a);

ALTER TABLE unlogged1 SET LOGGED;

ALTER TABLE unlogged1 SET UNLOGGED;

DROP TABLE unlogged1;

CREATE TABLE as_select1 AS SELECT * FROM pg_class WHERE relkind = 'r';

CREATE TABLE as_select1 AS SELECT * FROM pg_class WHERE relkind = 'r';

CREATE TABLE IF NOT EXISTS as_select1 AS SELECT * FROM pg_class WHERE relkind = 'r';

DROP TABLE as_select1;

PREPARE select1 AS SELECT 1 as a;

CREATE TABLE as_select1 AS EXECUTE select1;

CREATE TABLE as_select1 AS EXECUTE select1;

SELECT * FROM as_select1;

CREATE TABLE IF NOT EXISTS as_select1 AS EXECUTE select1;

DROP TABLE as_select1;

DEALLOCATE select1;

SELECT 'CREATE TABLE extra_wide_table(firstc text, '|| array_to_string(array_agg('c'||i||' bool'),',')||', lastc text);'
FROM generate_series(1, 1100) g(i)

INSERT INTO extra_wide_table(firstc, lastc) VALUES('first col', 'last col');

SELECT firstc, lastc FROM extra_wide_table;

CREATE TABLE withoid() WITH (oids);

CREATE TABLE withoid() WITH (oids = true);

CREATE TEMP TABLE withoutoid() WITHOUT OIDS;

DROP TABLE withoutoid;

CREATE TEMP TABLE withoutoid() WITH (oids = false);

DROP TABLE withoutoid;

CREATE TEMP TABLE relation_filenode_check(c1 int);

SELECT relpersistence,
  pg_filenode_relation (reltablespace, pg_relation_filenode(oid))
  FROM pg_class
  WHERE relname = 'relation_filenode_check';

DROP TABLE relation_filenode_check;

CREATE TABLE default_expr_column (id int DEFAULT (id));

CREATE TABLE default_expr_column (id int DEFAULT (bar.id));

CREATE TABLE default_expr_agg_column (id int DEFAULT (avg(id)));

CREATE TABLE default_expr_non_column (a int DEFAULT (avg(non_existent)));

CREATE TABLE default_expr_agg (a int DEFAULT (avg(1)));

CREATE TABLE default_expr_agg (a int DEFAULT (select 1));

CREATE TABLE default_expr_agg (a int DEFAULT (generate_series(1,3)));

BEGIN;

CREATE TABLE remember_create_subid (c int);

SAVEPOINT q;

DROP TABLE remember_create_subid;

ROLLBACK TO q;

COMMIT;

DROP TABLE remember_create_subid;

CREATE TABLE remember_node_subid (c int);

BEGIN;

ALTER TABLE remember_node_subid ALTER c TYPE bigint;

SAVEPOINT q;

DROP TABLE remember_node_subid;

ROLLBACK TO q;

COMMIT;

DROP TABLE remember_node_subid;

CREATE TABLE partitioned (
	a int
) INHERITS (some_table) PARTITION BY LIST (a);

CREATE TABLE partitioned (
	a1 int,
	a2 int
) PARTITION BY LIST (a1, a2);

CREATE FUNCTION retset (a int) RETURNS SETOF int AS $$ SELECT 1; $$ LANGUAGE SQL IMMUTABLE;

CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE (retset(a));

DROP FUNCTION retset(int);

CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE ((avg(a)));

CREATE TABLE partitioned (
	a int,
	b int
) PARTITION BY RANGE ((avg(a) OVER (PARTITION BY b)));

CREATE TABLE partitioned (
	a int
) PARTITION BY LIST ((a LIKE (SELECT 1)));

CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE ((42));

CREATE FUNCTION const_func () RETURNS int AS $$ SELECT 1; $$ LANGUAGE SQL IMMUTABLE;

CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE (const_func());

DROP FUNCTION const_func();

CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE (b);

CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE (xmin);

CREATE TABLE partitioned (
	a int,
	b int
) PARTITION BY RANGE (((a, b)));

CREATE TABLE partitioned (
	a int,
	b int
) PARTITION BY RANGE (a, ('unknown'));

CREATE FUNCTION immut_func (a int) RETURNS int AS $$ SELECT a + random()::int; $$ LANGUAGE SQL;

CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE (immut_func(a));

DROP FUNCTION immut_func(int);

CREATE TABLE partitioned (
	a point
) PARTITION BY LIST (a);

CREATE TABLE partitioned (
	a point
) PARTITION BY LIST (a point_ops);

CREATE TABLE partitioned (
	a point
) PARTITION BY RANGE (a);

CREATE TABLE partitioned (
	a point
) PARTITION BY RANGE (a point_ops);

CREATE TABLE partitioned (
	a int,
	CONSTRAINT check_a CHECK (a > 0) NO INHERIT
) PARTITION BY RANGE (a);

CREATE FUNCTION plusone(a int) RETURNS INT AS $$ SELECT a+1; $$ LANGUAGE SQL;

CREATE TABLE partitioned (
	a int,
	b int,
	c text,
	d text
) PARTITION BY RANGE (a oid_ops, plusone(b), c collate "default", d collate "C");

SELECT relkind FROM pg_class WHERE relname = 'partitioned';

DROP FUNCTION plusone(int);

CREATE TABLE partitioned2 (
	a int,
	b text
) PARTITION BY RANGE ((a+1), substr(b, 1, 5));

CREATE TABLE fail () INHERITS (partitioned2);

INSERT INTO partitioned2 VALUES (1, 'hello');

CREATE TABLE part2_1 PARTITION OF partitioned2 FOR VALUES FROM (-1, 'aaaaa') TO (100, 'ccccc');

DROP TABLE partitioned, partitioned2;

create table partitioned (a int, b int)
  partition by list ((row(a, b)::partitioned));

create table partitioned1
  partition of partitioned for values in ('(1,2)'::partitioned);

create table partitioned2
  partition of partitioned for values in ('(2,4)'::partitioned);

select * from partitioned where row(a,b)::partitioned = '(1,2)'::partitioned;

drop table partitioned;

create table partitioned (a int, b int)
  partition by list ((partitioned));

create table partitioned1
  partition of partitioned for values in ('(1,2)');

create table partitioned2
  partition of partitioned for values in ('(2,4)');

select * from partitioned where partitioned = '(1,2)'::partitioned;

drop table partitioned;

create domain intdom1 as int;

create table partitioned (
	a intdom1,
	b text
) partition by range (a);

alter table partitioned drop column a;

drop domain intdom1;

drop domain intdom1 cascade;

table partitioned;

create domain intdom1 as int;

create table partitioned (
	a intdom1,
	b text
) partition by range (plusone(a));

alter table partitioned drop column a;

drop domain intdom1;

drop domain intdom1 cascade;

table partitioned;

CREATE TABLE list_parted (
	a int
) PARTITION BY LIST (a);

CREATE TABLE part_p1 PARTITION OF list_parted FOR VALUES IN ('1');

CREATE TABLE part_p2 PARTITION OF list_parted FOR VALUES IN (2);

CREATE TABLE part_p3 PARTITION OF list_parted FOR VALUES IN ((2+1));

CREATE TABLE part_null PARTITION OF list_parted FOR VALUES IN (null);

CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN (somename);

CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN (somename.somename);

CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN (a);

CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN (sum(a));

CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN (sum(somename));

CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN (sum(1));

CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN ((select 1));

CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN (generate_series(4, 6));

CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN ((1+1) collate "POSIX");

CREATE TABLE fail_part PARTITION OF list_parted FOR VALUES FROM (1) TO (2);

CREATE TABLE fail_part PARTITION OF list_parted FOR VALUES WITH (MODULUS 10, REMAINDER 1);

CREATE TABLE part_default PARTITION OF list_parted DEFAULT;

CREATE TABLE fail_default_part PARTITION OF list_parted DEFAULT;

CREATE TABLE bools (
	a bool
) PARTITION BY LIST (a);

CREATE TABLE bools_true PARTITION OF bools FOR VALUES IN (1);

DROP TABLE bools;

CREATE TABLE moneyp (
	a money
) PARTITION BY LIST (a);

CREATE TABLE moneyp_10 PARTITION OF moneyp FOR VALUES IN (10);

CREATE TABLE moneyp_11 PARTITION OF moneyp FOR VALUES IN ('11');

CREATE TABLE moneyp_12 PARTITION OF moneyp FOR VALUES IN (to_char(12, '99')::int);

DROP TABLE moneyp;

CREATE TABLE bigintp (
	a bigint
) PARTITION BY LIST (a);

CREATE TABLE bigintp_10 PARTITION OF bigintp FOR VALUES IN (10);

CREATE TABLE bigintp_10_2 PARTITION OF bigintp FOR VALUES IN ('10');

DROP TABLE bigintp;

CREATE TABLE range_parted (
	a date
) PARTITION BY RANGE (a);

CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (somename) TO ('2019-01-01');

CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (somename.somename) TO ('2019-01-01');

CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (a) TO ('2019-01-01');

CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (max(a)) TO ('2019-01-01');

CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (max(somename)) TO ('2019-01-01');

CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (max('2019-02-01'::date)) TO ('2019-01-01');

CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM ((select 1)) TO ('2019-01-01');

CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (generate_series(1, 3)) TO ('2019-01-01');

CREATE TABLE fail_part PARTITION OF range_parted FOR VALUES IN ('a');

CREATE TABLE fail_part PARTITION OF range_parted FOR VALUES WITH (MODULUS 10, REMAINDER 1);

CREATE TABLE fail_part PARTITION OF range_parted FOR VALUES FROM ('a', 1) TO ('z');

CREATE TABLE fail_part PARTITION OF range_parted FOR VALUES FROM ('a') TO ('z', 1);

CREATE TABLE fail_part PARTITION OF range_parted FOR VALUES FROM (null) TO (maxvalue);

CREATE TABLE fail_part PARTITION OF range_parted FOR VALUES WITH (MODULUS 10, REMAINDER 1);

CREATE TABLE hash_parted (
	a int
) PARTITION BY HASH (a);

CREATE TABLE hpart_1 PARTITION OF hash_parted FOR VALUES WITH (MODULUS 10, REMAINDER 0);

CREATE TABLE hpart_2 PARTITION OF hash_parted FOR VALUES WITH (MODULUS 50, REMAINDER 1);

CREATE TABLE hpart_3 PARTITION OF hash_parted FOR VALUES WITH (MODULUS 200, REMAINDER 2);

CREATE TABLE hpart_4 PARTITION OF hash_parted FOR VALUES WITH (MODULUS 10, REMAINDER 3);

CREATE TABLE fail_part PARTITION OF hash_parted FOR VALUES WITH (MODULUS 25, REMAINDER 3);

CREATE TABLE fail_part PARTITION OF hash_parted FOR VALUES WITH (MODULUS 150, REMAINDER 3);

CREATE TABLE fail_part PARTITION OF hash_parted FOR VALUES WITH (MODULUS 100, REMAINDER 3);

CREATE TABLE fail_part PARTITION OF hash_parted FOR VALUES FROM ('a', 1) TO ('z');

CREATE TABLE fail_part PARTITION OF hash_parted FOR VALUES IN (1000);

CREATE TABLE fail_default_part PARTITION OF hash_parted DEFAULT;

CREATE TABLE unparted (
	a int
);

CREATE TABLE fail_part PARTITION OF unparted FOR VALUES IN ('a');

CREATE TABLE fail_part PARTITION OF unparted FOR VALUES WITH (MODULUS 2, REMAINDER 1);

DROP TABLE unparted;

CREATE TEMP TABLE temp_parted (
	a int
) PARTITION BY LIST (a);

CREATE TABLE fail_part PARTITION OF temp_parted FOR VALUES IN ('a');

DROP TABLE temp_parted;

CREATE TABLE list_parted2 (
	a varchar
) PARTITION BY LIST (a);

CREATE TABLE part_null_z PARTITION OF list_parted2 FOR VALUES IN (null, 'z');

CREATE TABLE part_ab PARTITION OF list_parted2 FOR VALUES IN ('a', 'b');

CREATE TABLE list_parted2_def PARTITION OF list_parted2 DEFAULT;

CREATE TABLE fail_part PARTITION OF list_parted2 FOR VALUES IN (null);

CREATE TABLE fail_part PARTITION OF list_parted2 FOR VALUES IN ('b', 'c');

INSERT INTO list_parted2 VALUES('X');

CREATE TABLE fail_part PARTITION OF list_parted2 FOR VALUES IN ('W', 'X', 'Y');

CREATE TABLE range_parted2 (
	a int
) PARTITION BY RANGE (a);

CREATE TABLE fail_part PARTITION OF range_parted2 FOR VALUES FROM (1) TO (0);

CREATE TABLE fail_part PARTITION OF range_parted2 FOR VALUES FROM (1) TO (1);

CREATE TABLE part0 PARTITION OF range_parted2 FOR VALUES FROM (minvalue) TO (1);

CREATE TABLE fail_part PARTITION OF range_parted2 FOR VALUES FROM (minvalue) TO (2);

CREATE TABLE part1 PARTITION OF range_parted2 FOR VALUES FROM (1) TO (10);

CREATE TABLE fail_part PARTITION OF range_parted2 FOR VALUES FROM (-1) TO (1);

CREATE TABLE fail_part PARTITION OF range_parted2 FOR VALUES FROM (9) TO (maxvalue);

CREATE TABLE part2 PARTITION OF range_parted2 FOR VALUES FROM (20) TO (30);

CREATE TABLE part3 PARTITION OF range_parted2 FOR VALUES FROM (30) TO (40);

CREATE TABLE fail_part PARTITION OF range_parted2 FOR VALUES FROM (10) TO (30);

CREATE TABLE fail_part PARTITION OF range_parted2 FOR VALUES FROM (10) TO (50);

CREATE TABLE range2_default PARTITION OF range_parted2 DEFAULT;

CREATE TABLE fail_default_part PARTITION OF range_parted2 DEFAULT;

INSERT INTO range_parted2 VALUES (85);

CREATE TABLE fail_part PARTITION OF range_parted2 FOR VALUES FROM (80) TO (90);

CREATE TABLE part4 PARTITION OF range_parted2 FOR VALUES FROM (90) TO (100);

CREATE TABLE range_parted3 (
	a int,
	b int
) PARTITION BY RANGE (a, (b+1));

CREATE TABLE part00 PARTITION OF range_parted3 FOR VALUES FROM (0, minvalue) TO (0, maxvalue);

CREATE TABLE fail_part PARTITION OF range_parted3 FOR VALUES FROM (0, minvalue) TO (0, 1);

CREATE TABLE part10 PARTITION OF range_parted3 FOR VALUES FROM (1, minvalue) TO (1, 1);

CREATE TABLE part11 PARTITION OF range_parted3 FOR VALUES FROM (1, 1) TO (1, 10);

CREATE TABLE part12 PARTITION OF range_parted3 FOR VALUES FROM (1, 10) TO (1, maxvalue);

CREATE TABLE fail_part PARTITION OF range_parted3 FOR VALUES FROM (1, 10) TO (1, 20);

CREATE TABLE range3_default PARTITION OF range_parted3 DEFAULT;

CREATE TABLE fail_part PARTITION OF range_parted3 FOR VALUES FROM (1, minvalue) TO (1, maxvalue);

CREATE TABLE hash_parted2 (
	a varchar
) PARTITION BY HASH (a);

CREATE TABLE h2part_1 PARTITION OF hash_parted2 FOR VALUES WITH (MODULUS 4, REMAINDER 2);

CREATE TABLE h2part_2 PARTITION OF hash_parted2 FOR VALUES WITH (MODULUS 8, REMAINDER 0);

CREATE TABLE h2part_3 PARTITION OF hash_parted2 FOR VALUES WITH (MODULUS 8, REMAINDER 4);

CREATE TABLE h2part_4 PARTITION OF hash_parted2 FOR VALUES WITH (MODULUS 8, REMAINDER 5);

CREATE TABLE fail_part PARTITION OF hash_parted2 FOR VALUES WITH (MODULUS 2, REMAINDER 1);

CREATE TABLE fail_part PARTITION OF hash_parted2 FOR VALUES WITH (MODULUS 0, REMAINDER 1);

CREATE TABLE fail_part PARTITION OF hash_parted2 FOR VALUES WITH (MODULUS 8, REMAINDER 8);

CREATE TABLE parted (
	a text,
	b int NOT NULL DEFAULT 0,
	CONSTRAINT check_a CHECK (length(a) > 0)
) PARTITION BY LIST (a);

CREATE TABLE part_a PARTITION OF parted FOR VALUES IN ('a');

SELECT attname, attislocal, attinhcount FROM pg_attribute
  WHERE attrelid = 'part_a'::regclass and attnum > 0
  ORDER BY attnum;

CREATE TABLE part_b PARTITION OF parted (
	b NOT NULL,
	b DEFAULT 1,
	b CHECK (b >= 0),
	CONSTRAINT check_a CHECK (length(a) > 0)
) FOR VALUES IN ('b');

CREATE TABLE part_b PARTITION OF parted (
	b NOT NULL DEFAULT 1,
	CONSTRAINT check_a CHECK (length(a) > 0),
	CONSTRAINT check_b CHECK (b >= 0)
) FOR VALUES IN ('b');

SELECT conname, conislocal, coninhcount FROM pg_constraint WHERE conrelid = 'part_b'::regclass ORDER BY coninhcount DESC, conname;

ALTER TABLE parted ADD CONSTRAINT check_b CHECK (b >= 0);

SELECT conname, conislocal, coninhcount FROM pg_constraint WHERE conrelid = 'part_b'::regclass ORDER BY coninhcount DESC, conname;

ALTER TABLE part_b DROP CONSTRAINT check_a;

ALTER TABLE part_b DROP CONSTRAINT check_b;

ALTER TABLE parted DROP CONSTRAINT check_a, DROP CONSTRAINT check_b;

SELECT conname, conislocal, coninhcount FROM pg_constraint WHERE conrelid = 'part_b'::regclass ORDER BY coninhcount DESC, conname;

CREATE TABLE fail_part_col_not_found PARTITION OF parted FOR VALUES IN ('c') PARTITION BY RANGE (c);

CREATE TABLE part_c PARTITION OF parted (b WITH OPTIONS NOT NULL DEFAULT 0) FOR VALUES IN ('c') PARTITION BY RANGE ((b));

CREATE TABLE part_c_1_10 PARTITION OF part_c FOR VALUES FROM (1) TO (10);

create table parted_notnull_inh_test (a int default 1, b int not null default 0) partition by list (a);

create table parted_notnull_inh_test1 partition of parted_notnull_inh_test (a not null, b default 1) for values in (1);

insert into parted_notnull_inh_test (b) values (null);

drop table parted_notnull_inh_test;

create table parted_boolean_col (a bool, b text) partition by list(a);

create table parted_boolean_less partition of parted_boolean_col
  for values in ('foo' < 'bar');

create table parted_boolean_greater partition of parted_boolean_col
  for values in ('foo' > 'bar');

drop table parted_boolean_col;

create table parted_collate_must_match (a text collate "C", b text collate "C")
  partition by range (a);

create table parted_collate_must_match1 partition of parted_collate_must_match
  (a collate "POSIX") for values from ('a') to ('m');

create table parted_collate_must_match2 partition of parted_collate_must_match
  (b collate "POSIX") for values from ('m') to ('z');

drop table parted_collate_must_match;

create table test_part_coll_posix (a text) partition by range (a collate "POSIX");

create table test_part_coll partition of test_part_coll_posix for values from ('a' collate "C") to ('g');

create table test_part_coll2 partition of test_part_coll_posix for values from ('g') to ('m');

create table test_part_coll_cast partition of test_part_coll_posix for values from (name 'm' collate "C") to ('s');

create table test_part_coll_cast2 partition of test_part_coll_posix for values from (name 's') to ('z');

drop table test_part_coll_posix;

CREATE TABLE range_parted4 (a int, b int, c int) PARTITION BY RANGE (abs(a), abs(b), c);

CREATE TABLE unbounded_range_part PARTITION OF range_parted4 FOR VALUES FROM (MINVALUE, MINVALUE, MINVALUE) TO (MAXVALUE, MAXVALUE, MAXVALUE);

DROP TABLE unbounded_range_part;

CREATE TABLE range_parted4_1 PARTITION OF range_parted4 FOR VALUES FROM (MINVALUE, MINVALUE, MINVALUE) TO (1, MAXVALUE, MAXVALUE);

CREATE TABLE range_parted4_2 PARTITION OF range_parted4 FOR VALUES FROM (3, 4, 5) TO (6, 7, MAXVALUE);

CREATE TABLE range_parted4_3 PARTITION OF range_parted4 FOR VALUES FROM (6, 8, MINVALUE) TO (9, MAXVALUE, MAXVALUE);

DROP TABLE range_parted4;

CREATE FUNCTION my_int4_sort(int4,int4) RETURNS int LANGUAGE sql
  AS $$ SELECT CASE WHEN $1 = $2 THEN 0 WHEN $1 > $2 THEN 1 ELSE -1 END; $$;

CREATE OPERATOR CLASS test_int4_ops FOR TYPE int4 USING btree AS
  OPERATOR 1 < (int4,int4), OPERATOR 2 <= (int4,int4),
  OPERATOR 3 = (int4,int4), OPERATOR 4 >= (int4,int4),
  OPERATOR 5 > (int4,int4), FUNCTION 1 my_int4_sort(int4,int4);

CREATE TABLE partkey_t (a int4) PARTITION BY RANGE (a test_int4_ops);

CREATE TABLE partkey_t_1 PARTITION OF partkey_t FOR VALUES FROM (0) TO (1000);

INSERT INTO partkey_t VALUES (100);

INSERT INTO partkey_t VALUES (200);

DROP TABLE parted, list_parted, range_parted, list_parted2, range_parted2, range_parted3;

DROP TABLE partkey_t, hash_parted, hash_parted2;

DROP OPERATOR CLASS test_int4_ops USING btree;

DROP FUNCTION my_int4_sort(int4,int4);

CREATE TABLE parted_col_comment (a int, b text) PARTITION BY LIST (a);

COMMENT ON TABLE parted_col_comment IS 'Am partitioned table';

COMMENT ON COLUMN parted_col_comment.a IS 'Partition key';

SELECT obj_description('parted_col_comment'::regclass);

DROP TABLE parted_col_comment;

CREATE TABLE parted_col_comment (a int, b text) PARTITION BY LIST (a) WITH (fillfactor=100);

CREATE TABLE arrlp (a int[]) PARTITION BY LIST (a);

CREATE TABLE arrlp12 PARTITION OF arrlp FOR VALUES IN ('{1}', '{2}');

DROP TABLE arrlp;

create table boolspart (a bool) partition by list (a);

create table boolspart_t partition of boolspart for values in (true);

create table boolspart_f partition of boolspart for values in (false);

drop table boolspart;

create table perm_parted (a int) partition by list (a);

create temporary table temp_parted (a int) partition by list (a);

create table perm_part partition of temp_parted default;

create temp table temp_part partition of perm_parted default;

create temp table temp_part partition of temp_parted default;

drop table perm_parted cascade;

drop table temp_parted cascade;

create table tab_part_create (a int) partition by list (a);

create or replace function func_part_create() returns trigger
  language plpgsql as $$
  begin
    execute 'create table tab_part_create_1 partition of tab_part_create for values in (1)';
    return null;
  end $$;

create trigger trig_part_create before insert on tab_part_create
  for each statement execute procedure func_part_create();

insert into tab_part_create values (1);

drop table tab_part_create;

drop function func_part_create();

create table volatile_partbound_test (partkey timestamp) partition by range (partkey);

create table volatile_partbound_test1 partition of volatile_partbound_test for values from (minvalue) to (current_timestamp);

create table volatile_partbound_test2 partition of volatile_partbound_test for values from (current_timestamp) to (maxvalue);

insert into volatile_partbound_test values (current_timestamp);

select tableoid::regclass from volatile_partbound_test;

drop table volatile_partbound_test;

create table defcheck (a int, b int) partition by list (b);

create table defcheck_def (a int, c int, b int);

alter table defcheck_def drop c;

alter table defcheck attach partition defcheck_def default;

alter table defcheck_def add check (b <= 0 and b is not null);

create table defcheck_1 partition of defcheck for values in (1, null);

insert into defcheck_def values (0, 0);

create table defcheck_0 partition of defcheck for values in (0);

drop table defcheck;

create table part_column_drop (
  useless_1 int,
  id int,
  useless_2 int,
  d int,
  b int,
  useless_3 int
) partition by range (id);

alter table part_column_drop drop column useless_1;

alter table part_column_drop drop column useless_2;

alter table part_column_drop drop column useless_3;

create index part_column_drop_b_pred on part_column_drop(b) where b = 1;

create index part_column_drop_b_expr on part_column_drop((b = 1));

create index part_column_drop_d_pred on part_column_drop(d) where d = 2;

create index part_column_drop_d_expr on part_column_drop((d = 2));

create table part_column_drop_1_10 partition of
  part_column_drop for values from (1) to (10);

drop table part_column_drop;
