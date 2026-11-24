-- advisory_lock.sql
SELECT
	pg_advisory_xact_lock(1), pg_advisory_xact_lock_shared(2),
	pg_advisory_xact_lock(1, 1), pg_advisory_xact_lock_shared(2, 2);

-- aggregates.sql
SELECT
  -- boolean and transitions
  -- null because strict
  booland_statefunc(NULL, NULL)  IS NULL AS "t",
  booland_statefunc(TRUE, NULL)  IS NULL AS "t",
  booland_statefunc(FALSE, NULL) IS NULL AS "t",
  booland_statefunc(NULL, TRUE)  IS NULL AS "t",
  booland_statefunc(NULL, FALSE) IS NULL AS "t",
  -- and actual computations
  booland_statefunc(TRUE, TRUE) AS "t",
  NOT booland_statefunc(TRUE, FALSE) AS "t",
  NOT booland_statefunc(FALSE, TRUE) AS "t",
  NOT booland_statefunc(FALSE, FALSE) AS "t";

-- alter_generic.sql
SELECT n.nspname, proname, prorettype::regtype, prokind, a.rolname
  FROM pg_proc p, pg_namespace n, pg_authid a
  WHERE p.pronamespace = n.oid AND p.proowner = a.oid
    AND n.nspname IN ('alt_nsp1', 'alt_nsp2')
  ORDER BY nspname, proname;

-- alter_operator.sql
SELECT pg_describe_object(refclassid,refobjid,refobjsubid) as ref, deptype
FROM pg_depend
WHERE classid = 'pg_operator'::regclass AND
      objid = '===(bool,bool)'::regoperator
ORDER BY 1;

-- alter_table.sql
SELECT relation::regclass, mode FROM pg_locks
WHERE pid = pg_backend_pid() AND locktype = 'relation'
  AND relation::regclass::text LIKE 'alter\_idx%'
ORDER BY relation::regclass::text COLLATE "C";

-- amutils.sql
select amname, prop, pg_indexam_has_property(a.oid, prop) as p
  from pg_am a,
       unnest(array['can_order', 'can_unique', 'can_multi_col',
                    'can_exclude', 'can_include', 'bogus']::text[])
         with ordinality as u(prop,ord)
 where amtype = 'i'
 order by amname, ord;

-- arrays.sql
SELECT *
   FROM arrtest
   WHERE a[1] < 5 and
         c = '{"foobar"}'::_name;

-- async.sql
SELECT pg_notify('notify_async1','sample message1');

-- bit.sql
INSERT INTO BIT_TABLE VALUES (B'00000000000');

-- bitmapops.sql
SELECT count(*) FROM bmscantest WHERE a = 1 AND b = 1;

-- boolean.sql
SELECT BOOLTBL1.*, BOOLTBL2.*
   FROM BOOLTBL1, BOOLTBL2
   WHERE BOOLTBL2.f1 = BOOLTBL1.f1 and BOOLTBL1.f1 = bool 'false';

-- box.sql
SELECT b1.*, b2.*
   FROM BOX_TBL b1, BOX_TBL b2
   WHERE b1.f1 @> b2.f1 and not b1.f1 ~= b2.f1;

-- brin.sql
UPDATE brintest SET textcol = '' WHERE textcol IS NOT NULL;

-- brin_bloom.sql
UPDATE brintest_bloom SET textcol = '' WHERE textcol IS NOT NULL;

-- brin_multi.sql
CREATE TABLE brintest_multi (
	int8col bigint,
	int2col smallint,
	int4col integer,
	oidcol oid,
	tidcol tid,
	float4col real,
	float8col double precision,
	macaddrcol macaddr,
	macaddr8col macaddr8,
	inetcol inet,
	cidrcol cidr,
	datecol date,
	timecol time without time zone,
	timestampcol timestamp without time zone,
	timestamptzcol timestamp with time zone,
	intervalcol interval,
	timetzcol time with time zone,
	numericcol numeric,
	uuidcol uuid,
	lsncol pg_lsn
) WITH (fillfactor=10);

-- btree_index.sql
explain (costs off)
SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, proargtypes) >= ('abs', NULL) AND proname <= 'abs'
ORDER BY proname, proargtypes, pronamespace;

-- case.sql
SELECT
  CASE
    WHEN i >= 3 THEN i
  END AS ">= 3 or Null"
  FROM CASE_TBL;

-- char.sql
INSERT INTO CHAR_TBL (f1) VALUES ('c     ');

-- circle.sql
SELECT c1.f1 AS one, c2.f1 AS two, (c1.f1 <-> c2.f1) AS distance
  FROM CIRCLE_TBL c1, CIRCLE_TBL c2
  WHERE (c1.f1 < c2.f1) AND ((c1.f1 <-> c2.f1) > 0)
  ORDER BY distance, area(c1.f1), area(c2.f1);

-- cluster.sql
SELECT pg_class.relname FROM pg_index, pg_class, pg_class AS pg_class_2
WHERE pg_class.oid=indexrelid
	AND indrelid=pg_class_2.oid
	AND pg_class_2.relname = 'clstr_tst'
	AND indisclustered;

-- collate.icu.utf8.sql
CREATE TABLE collate_test1 (
    a int,
    b text COLLATE "en-x-icu" NOT NULL
);

-- collate.linux.utf8.sql
CREATE TABLE collate_test1 (
    a int,
    b text COLLATE "en_US" NOT NULL
);

-- collate.sql
CREATE TABLE collate_test1 (
    a int,
    b text COLLATE "C" NOT NULL
);

-- collate.utf8.sql
CREATE COLLATION regress_builtin_c (
  provider = builtin, locale = 'C');

-- combocid.sql
CREATE TABLE IF NOT EXISTS testcase(
	id int PRIMARY KEY,
	balance numeric
);

-- comments.sql
SELECT /* both embedded and trailing single line */ 'both' AS third;

-- compression.sql
CREATE TABLE cmdata(f1 text COMPRESSION pglz);

-- constraints.sql
CREATE TABLE NE_CHECK_TBL (x int,
	CONSTRAINT CHECK_CON CHECK (x > 3) NOT ENFORCED);

-- conversion.sql
CREATE USER regress_conversion_user WITH NOCREATEDB NOCREATEROLE;

-- copy.sql
CREATE TABLE parted_si (
  id int not null,
  data text not null,
  -- prevent use of bulk insert by having a volatile function
  rand float8 not null default random()
)
PARTITION BY LIST((id % 2));

-- copy2.sql
CREATE TEMP TABLE x (
	a serial,
	b int,
	c text not null default 'stuff',
	d text,
	e text
) ;

-- copydml.sql
create trigger qqqbef before insert or update or delete on copydml_test
    for each row execute procedure qqq_trig();

-- copyencoding.sql
CREATE TABLE copy_encoding_tab (t text);

-- copyselect.sql
create view v_test1
as select 'v_'||t from test1;

-- create_aggregate.sql
CREATE OR REPLACE AGGREGATE myavg (numeric)
(
	stype = internal,
	sfunc = numeric_avg_accum,
	finalfunc = numeric_avg,
	serialfunc = numeric_avg_serialize,
	deserialfunc = numeric_avg_deserialize,
	combinefunc = numeric_avg_combine,
	finalfunc_modify = shareable  -- just to test a non-default setting
);

-- create_am.sql
SELECT a.amname FROM pg_class c, pg_am a
  WHERE c.relname = 'tableam_parted_heap2' AND a.oid = c.relam;

-- create_cast.sql
SELECT pg_describe_object(classid, objid, objsubid) as obj,
       pg_describe_object(refclassid, refobjid, refobjsubid) as objref,
       deptype
FROM pg_depend
WHERE classid = 'pg_cast'::regclass AND
      objid = (SELECT oid FROM pg_cast
               WHERE castsource = 'int4'::regtype
                 AND casttarget = 'casttesttype'::regtype)
ORDER BY refclassid;

-- create_function_sql.sql
CREATE FUNCTION functest_A_1(text, date) RETURNS bool LANGUAGE 'sql'
       AS 'SELECT $1 = ''abcd'' AND $2 > ''2001-01-01''';

-- create_index.sql
SELECT * FROM array_index_op_test WHERE i @> '{32}' AND t && '{AAAAAAA80240}' ORDER BY seqno;

-- create_misc.sql
SELECT *
   FROM b_star* x
   WHERE x.b = text 'bumble' or x.a < 3;

-- create_operator.sql
CREATE OPERATOR ## (
   leftarg = path,
   rightarg = path,
   function = path_inter,
   commutator = ##
);

-- create_procedure.sql
CREATE OR REPLACE PROCEDURE ptest5(a int, b text, c int default 100)
LANGUAGE SQL
AS $$
INSERT INTO cp_test VALUES(a, b);
INSERT INTO cp_test VALUES(c, b);
$$;

-- create_role.sql
SELECT shobj_description('regress_hasprivs'::regrole, 'pg_authid') IS NOT NULL AS has_comment;

-- create_schema.sql
CREATE ROLE regress_create_schema_role SUPERUSER;

-- create_table.sql
CREATE TABLE IF NOT EXISTS as_select1 AS SELECT * FROM pg_class WHERE relkind = 'r';

-- create_table_like.sql
CREATE TABLE ctlt1 (a text CHECK (length(a) > 2) ENFORCED PRIMARY KEY,
	b text CHECK (length(b) > 100) NOT ENFORCED);

-- create_type.sql
CREATE FUNCTION int42_in(cstring)
   RETURNS int42
   AS 'int4in'
   LANGUAGE internal STRICT IMMUTABLE;

-- create_view.sql
CREATE OR REPLACE VIEW viewtest AS
	SELECT * FROM viewtest_tbl;

-- database.sql
CREATE DATABASE regression_tbd
	ENCODING utf8 LC_COLLATE "C" LC_CTYPE "C" TEMPLATE template0;

-- date.sql
SELECT f1 FROM DATE_TBL
  WHERE f1 BETWEEN '2000-01-01' AND '2001-01-01';

-- dbsize.sql
SELECT size, pg_size_pretty(size), pg_size_pretty(-1 * size) FROM
    (VALUES (10::bigint), (1000::bigint), (1000000::bigint),
            (1000000000::bigint), (1000000000000::bigint),
            (1000000000000000::bigint)) x(size);

-- delete.sql
CREATE TABLE delete_test (
    id SERIAL PRIMARY KEY,
    a INT,
    b text
);

-- dependency.sql
CREATE TABLE deptest (f1 serial primary key, f2 text);

-- domain.sql
select m, m is not null from makedcomp(1,2) m;

-- drop_if_exists.sql
CREATE domain test_domain_exists as int not null check (value > 0);

-- drop_operator.sql
SELECT  ctid, oprcom
FROM    pg_catalog.pg_operator fk
WHERE   oprcom != 0 AND
        NOT EXISTS(SELECT 1 FROM pg_catalog.pg_operator pk WHERE pk.oid = fk.oprcom);

-- encoding.sql
CREATE TABLE regress_encoding(good text, truncated text, with_nul text, truncated_with_nul text);

-- enum.sql
ALTER TYPE planets ADD VALUE IF NOT EXISTS 'mercury';

-- equivclass.sql
explain (costs off)
  select * from ec0 where ff = f1 and f1 = '42'::int8;

-- euc_kr.sql
SELECT POSITION(
	convert_from('\xbcf6c7d0', 'EUC_KR') IN
	convert_from('\xb0fac7d02c20bcf6c7d02c20b1e2bcfa2c20bbee', 'EUC_KR'));

-- event_trigger.sql
CREATE OR REPLACE FUNCTION schema_two.add(int, int) RETURNS int LANGUAGE plpgsql
  CALLED ON NULL INPUT
  AS $$ BEGIN RETURN coalesce($1,0) + coalesce($2,0); END; $$;

-- event_trigger_login.sql
CREATE TABLE user_logins(id serial, who text);

-- explain.sql
create table gen_part (
  key1 integer not null,
  key2 integer not null
) partition by list (key1);

-- expressions.sql
select return_int_input(1) not in (10, 9, 2, 8, 3, 7, 4, 6, 5, 1);

-- fast_default.sql
CREATE FUNCTION set(tabname name) RETURNS VOID
AS $$
BEGIN
  UPDATE m
  SET id = (SELECT c.relfilenode
            FROM pg_class AS c, pg_namespace AS s
            WHERE c.relname = tabname
                AND c.relnamespace = s.oid
                AND s.nspname = 'fast_default');
END;
$$ LANGUAGE 'plpgsql';

-- float4.sql
INSERT INTO FLOAT4_TBL(f1) VALUES ('    0.0');

-- float8.sql
CREATE TEMP TABLE FLOAT8_TBL(f1 float8);

-- foreign_data.sql
CREATE USER MAPPING IF NOT EXISTS FOR current_user SERVER s1;

-- foreign_key.sql
CREATE TABLE FKTABLE ( ftest1 int CONSTRAINT fktable_ftest1_fkey REFERENCES PKTABLE MATCH FULL ON DELETE CASCADE ON UPDATE CASCADE NOT ENFORCED,
					   ftest2 int );

-- functional_deps.sql
CREATE TEMP TABLE articles (
    id int CONSTRAINT articles_pkey PRIMARY KEY,
    keywords text,
    title text UNIQUE NOT NULL,
    body text UNIQUE,
    created date
);

-- generated_stored.sql
MERGE INTO gtestm t USING (VALUES (1, 10), (2, 20)) v(id, f1) ON t.id = v.id
  WHEN MATCHED THEN UPDATE SET f1 = v.f1
  WHEN NOT MATCHED THEN INSERT VALUES (v.id, v.f1, 200)
  RETURNING merge_action(), old.*, new.*;

-- generated_virtual.sql
MERGE INTO gtestm t USING (VALUES (1, 10), (2, 20)) v(id, f1) ON t.id = v.id
  WHEN MATCHED THEN UPDATE SET f1 = v.f1
  WHEN NOT MATCHED THEN INSERT VALUES (v.id, v.f1, 200)
  RETURNING merge_action(), old.*, new.*;

-- geometry.sql
SELECT pg_input_is_valid('(1', 'circle');

-- gin.sql
select * from t_gin_test_tbl where array[0] <@ i and '{}'::int4[] <@ j;

-- gist.sql
select b, p from gist_tbl
where b <@ box(point(4.5, 4.5), point(5.5, 5.5))
and p <@ box(point(5,5), point(6, 6));

-- groupingsets.sql
explain (costs off)
select a, b, count(*) from gstest2 group by grouping sets ((a, b), (a)) having a > 1 and b > 1;

-- guc.sql
create or replace function myfunc(int) returns text as $$
begin
  set local work_mem = '2MB';
  return current_setting('work_mem');
end $$
language plpgsql
set work_mem = '1MB';

-- hash_func.sql
SELECT v as value, hashint2(v)::bit(32) as standard,
       hashint2extended(v, 0)::bit(32) as extended0,
       hashint2extended(v, 1)::bit(32) as extended1
FROM   (VALUES (0::int2), (1::int2), (17::int2), (42::int2)) x(v)
WHERE  hashint2(v)::bit(32) != hashint2extended(v, 0)::bit(32)
       OR hashint2(v)::bit(32) = hashint2extended(v, 1)::bit(32);

-- hash_index.sql
CREATE TABLE hash_i4_heap (
	seqno 		int4,
	random 		int4
);

-- hash_part.sql
select satisfies_hash_partition('text_hashp'::regclass, 2, 0, 'xxx'::text) OR
	   satisfies_hash_partition('text_hashp'::regclass, 2, 1, 'xxx'::text) AS satisfies;

-- horology.sql
SELECT timestamp with time zone '20011227 040506+08';

-- identity.sql
SELECT attrelid, attname, attidentity FROM pg_attribute WHERE attidentity NOT IN ('', 'a', 'd');

-- incremental_sort.sql
create or replace function explain_analyze_without_memory(query text)
returns table (out_line text) language plpgsql
as
$$
declare
  line text;
begin
  for line in
    execute 'explain (analyze, costs off, summary off, timing off, buffers off) ' || query
  loop
    out_line := regexp_replace(line, '\d+kB', 'NNkB', 'g');
    return next;
  end loop;
end;
$$;

-- index_including.sql
SELECT pg_get_constraintdef(oid), conname, conkey FROM pg_constraint WHERE conrelid = 'tbl'::regclass::oid AND contype = 'p';

-- index_including_gist.sql
DELETE FROM tbl_gist WHERE c1 = 5 OR c3 = 12;

-- indexing.sql
select relname as child, inhparent::regclass as parent, pg_get_indexdef as childdef
  from pg_class join pg_inherits on inhrelid = oid,
  lateral pg_get_indexdef(pg_class.oid)
  where relkind in ('i', 'I') and relname like 'idxpart%' order by relname;

-- indirect_toast.sql
CREATE TRIGGER indtoasttest_update_indirect
        BEFORE INSERT OR UPDATE
        ON indtoasttest
        FOR EACH ROW
        EXECUTE PROCEDURE update_using_indirect();

-- inet.sql
SELECT i, c, i & c AS "and" FROM inet_tbl;

-- infinite_recurse.sql
create function infinite_recurse() returns int as
'select infinite_recurse()' language sql;

-- inherit.sql
explain (costs off)
update some_tab set f3 = 11 where f1 = 12 and f2 = 13;

-- init_privs.sql
GRANT SELECT ON pg_proc TO CURRENT_USER;

-- insert.sql
create table inserttest (col1 int4, col2 int4 NOT NULL, col3 text default 'testing');

-- insert_conflict.sql
insert into insertconflicttest as t values (23, 'Blackberry') on conflict (key) where fruit like '%berry' and t.fruit = 'inconsequential' do nothing;

-- int2.sql
SELECT pg_input_is_valid('asdf', 'int2');

-- int4.sql
SELECT pg_input_is_valid('asdf', 'int4');

-- int8.sql
SELECT pg_input_is_valid('asdf', 'int8');

-- interval.sql
SELECT lhst.i lhs,
    rhst.i rhs,
    lhst.i < rhst.i AS lt,
    lhst.i <= rhst.i AS le,
    lhst.i = rhst.i AS eq,
    lhst.i > rhst.i AS gt,
    lhst.i >= rhst.i AS ge,
    lhst.i <> rhst.i AS ne
    FROM INFINITE_INTERVAL_TBL lhst CROSS JOIN INFINITE_INTERVAL_TBL rhst
    WHERE NOT isfinite(lhst.i);

-- join.sql
select * from x left join y on (x1 = y1 and x2 is not null);

-- join_hash.sql
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

-- json.sql
select attname, to_json(histogram_bounds) histogram_bounds
from pg_stats
where tablename = 'rows' and
      schemaname = pg_my_temp_schema()::regnamespace::text
order by 1;

-- json_encoding.sql
select json '{ "a":  "\ud83d\ude04\ud83d\udc36" }' -> 'a' as correct_in_utf8;

-- jsonb.sql
select attname, to_jsonb(histogram_bounds) histogram_bounds
from pg_stats
where tablename = 'rows' and
      schemaname = pg_my_temp_schema()::regnamespace::text
order by 1;

-- jsonb_jsonpath.sql
select jsonb '{"a": {"a": 12}}' @? '$.a.a';

-- jsonpath.sql
select '1 * 2 + 4 % -3 != false'::jsonpath;

-- jsonpath_encoding.sql
select '"\ud83d\ude04\ud83d\udc36"'::jsonpath as correct_in_utf8;

-- largeobject.sql
ALTER LARGE OBJECT 42 OWNER TO regress_lo_user;

-- limit.sql
SELECT
  (SELECT n
     FROM (VALUES (1)) AS x,
          (SELECT n FROM generate_series(1,10) AS n
             ORDER BY n LIMIT 1 OFFSET s-1) AS y) AS z
  FROM generate_series(1,10) AS s;

-- line.sql
INSERT INTO LINE_TBL VALUES ('{0,-1,5}');

-- lock.sql
select relname from pg_locks l, pg_class c
 where l.relation = c.oid and relname like '%lock_%' and mode = 'ExclusiveLock'
 order by relname;

-- lseg.sql
INSERT INTO LSEG_TBL VALUES ('[(1,2),(3,4)]');

-- macaddr.sql
CREATE TABLE macaddr_data (a int, b macaddr);

-- macaddr8.sql
SELECT '08:00:2b:01:02:03     '::macaddr8;

-- maintain_every.sql
CREATE TEMP TABLE past_inh_db_parent () ON COMMIT DELETE ROWS;

-- matview.sql
CREATE TABLE mvtest_t (id int NOT NULL PRIMARY KEY, type text NOT NULL, amt numeric NOT NULL);

-- md5.sql
select md5('') = 'd41d8cd98f00b204e9800998ecf8427e' AS "TRUE";

-- memoize.sql
SELECT explain_memoize('
SELECT * FROM expr_key t1 INNER JOIN expr_key t2
ON t1.x = t2.t::numeric AND t1.t::numeric = t2.x;', false);

-- merge.sql
SELECT t.ctid is not null as matched, t.*, s.* FROM source s FULL OUTER JOIN target t ON s.sid = t.tid ORDER BY t.tid, s.sid;

-- misc.sql
CREATE TABLE hobbies_r (
	name		text,
	person 		text
);

-- misc_functions.sql
select pg_read_file('does not exist', true) IS NULL;

-- misc_sanity.sql
SELECT *
FROM pg_depend as d1
WHERE refclassid = 0 OR refobjid = 0 OR
      classid = 0 OR objid = 0 OR
      deptype NOT IN ('a', 'e', 'i', 'n', 'x', 'P', 'S');

-- money.sql
SELECT cashlarger(m, '$124.00') FROM money_data;

-- multirangetypes.sql
create table reservations ( room_id integer not null, booked_during daterange );

-- mvcc.sql
CREATE TABLE clean_aborted_self(key int, data text);

-- name.sql
SELECT name 'name string' = name 'name string' AS "True";

-- namespace.sql
CREATE SCHEMA IF NOT EXISTS test_ns_schema_renamed;

-- numeric.sql
SELECT t1.id1, t1.id2, t1.result, t2.expected
    FROM num_result t1, num_exp_add t2
    WHERE t1.id1 = t2.id1 AND t1.id2 = t2.id2
    AND t1.result != t2.expected;

-- numeric_big.sql
SELECT t1.id1, t1.id2, t1.result, t2.expected
    FROM num_result t1, num_exp_add t2
    WHERE t1.id1 = t2.id1 AND t1.id2 = t2.id2
    AND t1.result != t2.expected;

-- numerology.sql
SELECT 0b1111111111111111111111111111111;

-- object_address.sql
CREATE SERVER addr_fserv FOREIGN DATA WRAPPER addr_fdw;

-- oid.sql
INSERT INTO OID_TBL(f1) VALUES ('1234');

-- opr_sanity.sql
SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prolang = 0 OR p1.prorettype = 0 OR
       p1.pronargs < 0 OR
       p1.pronargdefaults < 0 OR
       p1.pronargdefaults > p1.pronargs OR
       array_lower(p1.proargtypes, 1) != 0 OR
       array_upper(p1.proargtypes, 1) != p1.pronargs-1 OR
       0::oid = ANY (p1.proargtypes) OR
       procost <= 0 OR
       CASE WHEN proretset THEN prorows <= 0 ELSE prorows != 0 END OR
       prokind NOT IN ('f', 'a', 'w', 'p') OR
       provolatile NOT IN ('i', 's', 'v') OR
       proparallel NOT IN ('s', 'r', 'u');

-- partition_aggregate.sql
EXPLAIN (COSTS OFF)
SELECT a.x, b.y, count(*) FROM (SELECT * FROM pagg_tab1 WHERE x < 20) a LEFT JOIN (SELECT * FROM pagg_tab2 WHERE y > 10) b ON a.x = b.y WHERE a.x > 5 or b.y < 20  GROUP BY a.x, b.y ORDER BY 1, 2;

-- partition_info.sql
SELECT * FROM pg_partition_ancestors(NULL);

-- partition_join.sql
EXPLAIN (COSTS OFF)
SELECT t1.a, t1.c, t2.b, t2.c FROM prt1 t1, prt2 t2 WHERE t1.a = t2.b AND t1.b = 0 ORDER BY t1.a, t2.b;

-- partition_prune.sql
explain (costs off) select * from lp where a > 'a' and a < 'd';

-- password.sql
SELECT rolname, rolpassword not like '%A6xHKoH/494E941doaPOYg==%' as is_rolpassword_rehashed
    FROM pg_authid
    WHERE rolname LIKE 'regress_passwd_sha_len%'
    ORDER BY rolname;

-- path.sql
INSERT INTO PATH_TBL VALUES ('[(1,2),(3,4)]');

-- pg_lsn.sql
EXPLAIN (COSTS OFF)
SELECT DISTINCT (i || '/' || j)::pg_lsn f
  FROM generate_series(1, 10) i,
       generate_series(1, 10) j,
       generate_series(1, 5) k
  WHERE i <= 10 AND j > 0 AND j <= 10
  ORDER BY f;

-- plancache.sql
create or replace temp view v1 as
  select 2+2+4 as f1;

-- plpgsql.sql
create function tg_wslot_biu() returns trigger as $$
begin
    if count(*) = 0 from Room where roomno = new.roomno then
        raise exception 'Room % does not exist', new.roomno;
    end if;
    return new;
end;
$$ language plpgsql;

-- point.sql
CREATE TEMP TABLE point_gist_tbl(f1 point);

-- polygon.sql
SELECT *
FROM quad_poly_tbl_ord_seq2 seq FULL JOIN quad_poly_tbl_ord_idx2 idx
	ON seq.n = idx.n AND seq.id = idx.id AND
		(seq.dist = idx.dist OR seq.dist IS NULL AND idx.dist IS NULL)
WHERE seq.id IS NULL OR idx.id IS NULL;

-- polymorphism.sql
select distinct array_ndims(histogram_bounds) from pg_stats
where histogram_bounds is not null;

-- portals.sql
SELECT name, statement, is_holdable, is_binary, is_scrollable FROM pg_cursors ORDER BY 1;

-- predicate.sql
CREATE TABLE pred_tab (a int NOT NULL, b int, c int NOT NULL);

-- prepare.sql
SELECT name, statement, parameter_types, result_types FROM pg_prepared_statements;

-- prepared_xacts.sql
CREATE TABLE pxtest1 (foobar VARCHAR(10));

-- privileges.sql
SELECT lo_unlink(oid) FROM pg_largeobject_metadata WHERE oid >= 1000 AND oid < 3000 ORDER BY oid;

-- psql.sql
select format('create index on gexec_test(%I)', attname)
from pg_attribute
where attrelid = 'gexec_test'::regclass and attnum > 0
order by attnum;

-- psql_crosstab.sql
CREATE TABLE ctv_data (v, h, c, i, d) AS
VALUES
   ('v1','h2','foo', 3, '2015-04-01'::date),
   ('v2','h1','bar', 3, '2015-01-02'),
   ('v1','h0','baz', NULL, '2015-07-12'),
   ('v0','h4','qux', 4, '2015-07-15'),
   ('v0','h4','dbl', -3, '2014-12-15'),
   ('v0',NULL,'qux', 5, '2014-07-15'),
   ('v1','h2','quux',7, '2015-04-04');

-- psql_pipeline.sql
CREATE TABLE psql_pipeline(a INTEGER PRIMARY KEY, s TEXT);

-- publication.sql
CREATE PUBLICATION testpub5 FOR TABLE testpub_rf_tbl1, testpub_rf_tbl2 WHERE (c <> 'test' AND d < 5) WITH (publish = 'insert');

-- random.sql
SELECT count(*) FILTER (WHERE r < 0 OR r >= 1) AS out_of_range,
       (count(*) FILTER (WHERE r < 0.01)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 0.99)) > 0 AS has_large
FROM (SELECT random() r FROM generate_series(1, 2000)) ss;

-- rangefuncs.sql
CREATE OR REPLACE FUNCTION rngfunc(in f1 int, out f2 int) RETURNS int
AS 'select $1+1' LANGUAGE sql;

-- rangetypes.sql
select pg_input_is_valid('(1,4)', 'int4range');

-- regex.sql
select 'abc abc abc' ~ '^(\w+)( \1)+$' as t;

-- regproc.sql
SELECT regoperator('pg_catalog.+(int4,int4)');

-- reindex_catalog.sql
REINDEX INDEX pg_class_relname_nsp_index;

-- reloptions.sql
SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass AND
       reloptions IS NULL;

-- replica_identity.sql
CREATE TABLE test_replica_identity (
       id serial primary key,
       keya text not null,
       keyb text not null,
       nonkey text,
       CONSTRAINT test_replica_identity_unique_defer UNIQUE (keya, keyb) DEFERRABLE,
       CONSTRAINT test_replica_identity_unique_nondefer UNIQUE (keya, keyb)
) ;

-- returning.sql
CREATE OR REPLACE RULE voo_i AS ON INSERT TO voo DO INSTEAD
  INSERT INTO foo VALUES(new.*, 57) RETURNING f1, f2;

-- roleattributes.sql
SELECT rolname, rolsuper, rolinherit, rolcreaterole, rolcreatedb, rolcanlogin, rolreplication, rolbypassrls, rolconnlimit, rolpassword, rolvaliduntil FROM pg_authid WHERE rolname = 'regress_test_def_superuser';

-- rowsecurity.sql
CREATE OR REPLACE FUNCTION f_leak(text) RETURNS bool
    COST 0.0000001 LANGUAGE plpgsql
    AS 'BEGIN RAISE NOTICE ''f_leak => %'', $1; RETURN true; END';

-- rowtypes.sql
CREATE TABLE price (
    id SERIAL PRIMARY KEY,
    active BOOLEAN NOT NULL,
    price NUMERIC
);

-- rules.sql
create rule rtest_t4_ins1 as on insert to rtest_t4
		where new.a >= 10 and new.a < 20 do instead
	insert into rtest_t5 values (new.a, new.b);

-- sanity_check.sql
SELECT relname, nspname
 FROM pg_class c LEFT JOIN pg_namespace n ON n.oid = relnamespace JOIN pg_attribute a ON (attrelid = c.oid AND attname = 'oid')
 WHERE relkind = 'r' and c.oid < 16384
     AND ((nspname ~ '^pg_') IS NOT FALSE)
     AND NOT EXISTS (SELECT 1 FROM pg_index i WHERE indrelid = c.oid
                     AND indkey[0] = a.attnum AND indnatts = 1
                     AND indisunique AND indimmediate);

-- security_label.sql
DROP ROLE IF EXISTS regress_seclabel_user1;

-- select.sql
select foo from (select 1 offset 0) as foo;

-- select_distinct.sql
CREATE OR REPLACE FUNCTION distinct_func(a INT) RETURNS INT AS $$
  BEGIN
    RETURN a;
  END;
$$ LANGUAGE plpgsql PARALLEL UNSAFE;

-- select_distinct_on.sql
CREATE TABLE distinct_on_tbl (x int, y int, z int);

-- select_having.sql
SELECT lower(c), count(c) FROM test_having
	GROUP BY lower(c) HAVING count(*) > 2 OR min(a) = max(a)
	ORDER BY lower(c);

-- select_implicit.sql
CREATE TABLE test_missing_target (a int, b int, c char(8), d char);

-- select_into.sql
CREATE TABLE IF NOT EXISTS ctas_ine_tbl AS SELECT 1 / 0;

-- select_parallel.sql
PREPARE pstmt(text, int[]) AS SELECT * FROM fooarr WHERE f1 = $1 AND f2 = $2;

-- select_views.sql
CREATE TABLE customer (
       cid      int primary key,
       name     text not null,
       tel      text,
       passwd	text
);

-- sequence.sql
CREATE SEQUENCE IF NOT EXISTS sequence_test;

-- spgist.sql
create table spgist_point_tbl(id int4, p point);

-- sqljson.sql
SELECT to_json(a) AS a, JSON_OBJECTAGG(k : v ABSENT ON NULL)
OVER (ORDER BY k RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
FROM (VALUES (1,1), (1,null), (2,2)) a(k,v);

-- sqljson_jsontable.sql
SELECT * FROM JSON_TABLE('[]', 'strict $.a' COLUMNS (js2 int PATH '$') EMPTY ON ERROR);

-- sqljson_queryfuncs.sql
CREATE DOMAIN sqljsonb_int_not_null AS int NOT NULL;

-- stats.sql
CREATE TABLE prevstats AS
SELECT t.seq_scan, t.seq_tup_read, t.idx_scan, t.idx_tup_fetch,
       (b.heap_blks_read + b.heap_blks_hit) AS heap_blks,
       (b.idx_blks_read + b.idx_blks_hit) AS idx_blks,
       pg_stat_get_snapshot_timestamp() as snap_ts
  FROM pg_catalog.pg_stat_user_tables AS t,
       pg_catalog.pg_statio_user_tables AS b
 WHERE t.relname='tenk2' AND b.relname='tenk2';

-- stats_ext.sql
CREATE STATISTICS IF NOT EXISTS ab1_a_b_stats ON a, b FROM ab1;

-- stats_import.sql
SELECT mode FROM pg_locks
WHERE relation = 'stats_import.test'::regclass AND
      pid = pg_backend_pid() AND granted;

-- strings.sql
SELECT 'hawkeye' NOT LIKE 'h%' AS "false";

-- subscription.sql
CREATE ROLE regress_subscription_user LOGIN SUPERUSER;

-- subselect.sql
SELECT 1 AS zero WHERE 1 NOT IN (SELECT 1);

-- sysviews.sql
with contexts as (
  select * from pg_backend_memory_contexts
)
select count(*) > 1
from contexts c1, contexts c2
where c2.name = 'CacheMemoryContext'
and c1.path[c2.level] = c2.path[c2.level];

-- tablesample.sql
CREATE TABLE test_tablesample (id int, name text) WITH (fillfactor=10);

-- tablespace.sql
SELECT c.relname FROM pg_class c, pg_tablespace s
  WHERE c.reltablespace = s.oid AND s.spcname = 'regress_tblspace';

-- temp.sql
CREATE TEMPORARY TABLE test_temp(a int not null unique, b TEXT not null, cnt int not null);

-- test_setup.sql
INSERT INTO INT2_TBL(f1) VALUES
  ('0   '),
  ('  1234 '),
  ('    -1234'),
  ('32767'),  -- largest and smallest values
  ('-32767');

-- text.sql
SELECT text 'this is a text string' = text 'this is a text string' AS true;

-- tid.sql
SELECT
  '(0,0)'::tid as tid00,
  '(0,1)'::tid as tid01,
  '(-1,0)'::tid as tidm10,
  '(4294967295,65535)'::tid as tidmax;

-- tidrangescan.sql
DELETE FROM tidrangescan
WHERE substring(ctid::text FROM ',(\d+)\)')::integer > 10 OR substring(ctid::text FROM '\((\d+),')::integer > 2;

-- tidscan.sql
EXPLAIN (COSTS OFF)
SELECT ctid, * FROM tidscan WHERE ctid = '(0,2)' OR '(0,1)' = ctid;

-- time.sql
INSERT INTO TIME_TBL VALUES ('02:03 PST');

-- timestamp.sql
SELECT d1 - timestamp without time zone '1997-01-02' AS diff
   FROM TIMESTAMP_TBL WHERE d1 BETWEEN '1902-01-01' AND '2038-01-01';

-- timestamptz.sql
SELECT d1 - timestamp with time zone '1997-01-02' AS diff
   FROM TIMESTAMPTZ_TBL WHERE d1 BETWEEN '1902-01-01' AND '2038-01-01';

-- timetz.sql
CREATE TABLE TIMETZ_TBL (f1 time(2) with time zone);

-- transactions.sql
UPDATE temptest SET a = 0 FROM writetest WHERE temptest.a = 1 AND writetest.a = temptest.a;

-- triggers.sql
create trigger trigger_alpha
	before insert or update on trigtest
	for each row execute procedure f1_times_10();

-- truncate.sql
CREATE TABLE truncate_a (col1 integer primary key);

-- tsdicts.sql
SELECT to_tsvector('synonym_tst', 'Postgresql is often called as postgres or pgsql and pronounced as postgre');

-- tsearch.sql
SELECT oid, prsname
FROM pg_ts_parser
WHERE prsnamespace = 0 OR prsstart = 0 OR prstoken = 0 OR prsend = 0 OR
      -- prsheadline is optional
      prslextype = 0;

-- tsrf.sql
SELECT generate_series(1, 3), generate_series(3,5);

-- tstypes.sql
SELECT E'''1 \\''2'' '' 3'' 4 '::tsvector;

-- tuplesort.sql
CREATE TEMP TABLE abbrev_abort_uuids (
    id serial not null,
    abort_increasing uuid,
    abort_decreasing uuid,
    noabort_increasing uuid,
    noabort_decreasing uuid);

-- txid.sql
create temp table snapshot_test (
	nr	integer,
	snap	txid_snapshot
);

-- type_sanity.sql
SELECT t1.oid, t1.typname
FROM pg_type as t1
WHERE t1.typnamespace = 0 OR
    (t1.typlen <= 0 AND t1.typlen != -1 AND t1.typlen != -2) OR
    (t1.typtype not in ('b', 'c', 'd', 'e', 'm', 'p', 'r')) OR
    NOT t1.typisdefined OR
    (t1.typalign not in ('c', 's', 'i', 'd')) OR
    (t1.typstorage not in ('p', 'x', 'e', 'm'));

-- typed_table.sql
CREATE TABLE IF NOT EXISTS persons OF person_type;

-- unicode.sql
SELECT U&'\0061\0308bc' <> U&'\00E4bc' COLLATE "C" AS sanity_check;

-- union.sql
with cte as not materialized (select s from generate_series(1,5) s)
select from cte union select from cte;

-- updatable_views.sql
MERGE INTO ro_view13 AS t USING (VALUES (2, 'Row 2')) AS v(a,b) ON t.a = v.a
  WHEN MATCHED THEN DO NOTHING
  WHEN NOT MATCHED THEN DO NOTHING;

-- update.sql
UPDATE update_test
  SET (b,a) = (select a,b from update_test where b = 41 and c = 'car')
  WHERE a = 100 AND b = 20;

-- uuid.sql
EXPLAIN (COSTS OFF)
SELECT COUNT(*) FROM guid1 WHERE guid_field <> '11111111111111111111111111111111' OR
							guid_field <> '3f3e3c3b-3a30-3938-3736-353433a2313e';

-- vacuum.sql
CREATE TEMP TABLE vac_truncate_test(i INT NOT NULL, j text)
	WITH (vacuum_truncate=true, autovacuum_enabled=false);

-- vacuum_parallel.sql
SELECT EXISTS (
SELECT 1
FROM pg_class
WHERE oid = 'vacuum_in_leader_small_index'::regclass AND
  pg_relation_size(oid) <
  pg_size_bytes(current_setting('min_parallel_index_scan_size'))
) as leader_will_handle_small_index;

-- varchar.sql
CREATE TEMP TABLE VARCHAR_TBL(f1 varchar(1));

-- window.sql
CREATE TEMP VIEW v_window AS
	SELECT i, sum(i) over (order by i rows between 1 preceding and 1 following) as sum_rows
	FROM generate_series(1, 10) i;

-- with.sql
CREATE OR REPLACE RECURSIVE VIEW nums (n) AS
    VALUES (1)
UNION ALL
    SELECT n+1 FROM nums WHERE n < 6;

-- without_overlaps.sql
UPDATE  temporal_rng
SET     id = '[11,12)'
WHERE   id = '[1,2)'
AND     valid_at @> '2018-01-15'::date;

-- xid.sql
select '010'::xid,
       '42'::xid,
       '0xffffffff'::xid,
       '-1'::xid,
	   '010'::xid8,
	   '42'::xid8,
	   '0xffffffffffffffff'::xid8,
	   '-1'::xid8;

-- xml.sql
SELECT xmltable.* FROM xmldata, LATERAL xmltable('/ROWS/ROW[COUNTRY_NAME="Japan" or COUNTRY_NAME="India"]' PASSING data COLUMNS "COUNTRY_NAME" text, "REGION_ID" int);

-- xmlmap.sql
CREATE TABLE testxmlschema.test1 (a int, b text);

