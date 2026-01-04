SELECT getdatabaseencoding() <> 'UTF8' OR
       (SELECT count(*) FROM pg_collation WHERE collname IN ('de_DE', 'en_US', 'sv_SE', 'tr_TR') AND collencoding = pg_char_to_encoding('UTF8')) <> 4 OR
       version() !~ 'linux-gnu'
       AS skip_test ;

SET client_encoding TO UTF8;

CREATE SCHEMA collate_tests;

SET search_path = collate_tests;

CREATE TABLE collate_test1 (
    a int,
    b text COLLATE "en_US" NOT NULL
);

CREATE TABLE collate_test_fail (
    a int,
    b text COLLATE "ja_JP.eucjp"
);

CREATE TABLE collate_test_fail (
    a int,
    b text COLLATE "foo"
);

CREATE TABLE collate_test_fail (
    a int COLLATE "en_US",
    b text
);

CREATE TABLE collate_test_like (
    LIKE collate_test1
);

CREATE TABLE collate_test2 (
    a int,
    b text COLLATE "sv_SE"
);

CREATE TABLE collate_test3 (
    a int,
    b text COLLATE "C"
);

INSERT INTO collate_test1 VALUES (1, 'abc'), (2, 'Ã¤bc'), (3, 'bbc'), (4, 'ABC');

INSERT INTO collate_test2 SELECT * FROM collate_test1;

INSERT INTO collate_test3 SELECT * FROM collate_test1;

SELECT * FROM collate_test1 WHERE b >= 'bbc';

SELECT * FROM collate_test2 WHERE b >= 'bbc';

SELECT * FROM collate_test3 WHERE b >= 'bbc';

SELECT * FROM collate_test3 WHERE b >= 'BBC';

SELECT * FROM collate_test1 WHERE b COLLATE "C" >= 'bbc';

SELECT * FROM collate_test1 WHERE b >= 'bbc' COLLATE "C";

SELECT * FROM collate_test1 WHERE b COLLATE "C" >= 'bbc' COLLATE "C";

SELECT * FROM collate_test1 WHERE b COLLATE "C" >= 'bbc' COLLATE "en_US";

CREATE DOMAIN testdomain_sv AS text COLLATE "sv_SE";

CREATE DOMAIN testdomain_i AS int COLLATE "sv_SE";

CREATE TABLE collate_test4 (
    a int,
    b testdomain_sv
);

INSERT INTO collate_test4 SELECT * FROM collate_test1;

SELECT a, b FROM collate_test4 ORDER BY b;

CREATE TABLE collate_test5 (
    a int,
    b testdomain_sv COLLATE "en_US"
);

INSERT INTO collate_test5 SELECT * FROM collate_test1;

SELECT a, b FROM collate_test5 ORDER BY b;

SELECT a, b FROM collate_test1 ORDER BY b;

SELECT a, b FROM collate_test2 ORDER BY b;

SELECT a, b FROM collate_test3 ORDER BY b;

SELECT a, b FROM collate_test1 ORDER BY b COLLATE "C";

SELECT * FROM collate_test1 ORDER BY b;

SELECT * FROM collate_test2 ORDER BY b;

SELECT * FROM collate_test3 ORDER BY b;

SELECT 'bbc' COLLATE "en_US" > 'Ã¤bc' COLLATE "en_US" AS "true";

SELECT 'bbc' COLLATE "sv_SE" > 'Ã¤bc' COLLATE "sv_SE" AS "false";

CREATE TABLE collate_test10 (
    a int,
    x text COLLATE "en_US",
    y text COLLATE "tr_TR"
);

INSERT INTO collate_test10 VALUES (1, 'hij', 'hij'), (2, 'HIJ', 'HIJ');

SELECT a, lower(x), lower(y), upper(x), upper(y), initcap(x), initcap(y) FROM collate_test10;

SELECT a, lower(x COLLATE "C"), lower(y COLLATE "C") FROM collate_test10;

SELECT a, x, y FROM collate_test10 ORDER BY lower(y), a;

SELECT * FROM collate_test1 WHERE b LIKE 'abc';

SELECT * FROM collate_test1 WHERE b LIKE 'abc%';

SELECT * FROM collate_test1 WHERE b LIKE '%bc%';

SELECT * FROM collate_test1 WHERE b ILIKE 'abc';

SELECT * FROM collate_test1 WHERE b ILIKE 'abc%';

SELECT * FROM collate_test1 WHERE b ILIKE '%bc%';

SELECT 'TÃ¼rkiye' COLLATE "en_US" ILIKE '%KI%' AS "true";

SELECT 'TÃ¼rkiye' COLLATE "tr_TR" ILIKE '%KI%' AS "false";

SELECT 'bÄ±t' ILIKE 'BIT' COLLATE "en_US" AS "false";

SELECT 'bÄ±t' ILIKE 'BIT' COLLATE "tr_TR" AS "true";

SELECT relname FROM pg_class WHERE relname ILIKE 'abc%';

SELECT * FROM collate_test1 WHERE b ~ '^abc$';

SELECT * FROM collate_test1 WHERE b ~ '^abc';

SELECT * FROM collate_test1 WHERE b ~ 'bc';

SELECT * FROM collate_test1 WHERE b ~* '^abc$';

SELECT * FROM collate_test1 WHERE b ~* '^abc';

SELECT * FROM collate_test1 WHERE b ~* 'bc';

CREATE TABLE collate_test6 (
    a int,
    b text COLLATE "en_US"
);

INSERT INTO collate_test6 VALUES (1, 'abc'), (2, 'ABC'), (3, '123'), (4, 'ab1'),
                                 (5, 'a1!'), (6, 'a c'), (7, '!.;'), (8, '   '),
                                 (9, 'Ã¤bÃ§'), (10, 'ÃBÃ');

SELECT b,
       b ~ '^[[:alpha:]]+$' AS is_alpha,
       b ~ '^[[:upper:]]+$' AS is_upper,
       b ~ '^[[:lower:]]+$' AS is_lower,
       b ~ '^[[:digit:]]+$' AS is_digit,
       b ~ '^[[:alnum:]]+$' AS is_alnum,
       b ~ '^[[:graph:]]+$' AS is_graph,
       b ~ '^[[:print:]]+$' AS is_print,
       b ~ '^[[:punct:]]+$' AS is_punct,
       b ~ '^[[:space:]]+$' AS is_space
FROM collate_test6;

SELECT 'TÃ¼rkiye' COLLATE "en_US" ~* 'KI' AS "true";

SELECT 'TÃ¼rkiye' COLLATE "tr_TR" ~* 'KI' AS "false";

SELECT 'bÄ±t' ~* 'BIT' COLLATE "en_US" AS "false";

SELECT 'bÄ±t' ~* 'BIT' COLLATE "tr_TR" AS "true";

SELECT relname FROM pg_class WHERE relname ~* '^abc';

SET lc_time TO 'tr_TR';

SELECT to_char(date '2010-02-01', 'DD TMMON YYYY');

SELECT to_char(date '2010-02-01', 'DD TMMON YYYY' COLLATE "tr_TR");

SELECT to_char(date '2010-04-01', 'DD TMMON YYYY');

SELECT to_char(date '2010-04-01', 'DD TMMON YYYY' COLLATE "tr_TR");

SELECT to_date('01 ÅUB 2010', 'DD TMMON YYYY');

SELECT to_date('01 Åub 2010', 'DD TMMON YYYY');

SELECT to_date('1234567890ab 2010', 'TMMONTH YYYY');

CREATE VIEW collview1 AS SELECT * FROM collate_test1 WHERE b COLLATE "C" >= 'bbc';

CREATE VIEW collview2 AS SELECT a, b FROM collate_test1 ORDER BY b COLLATE "C";

CREATE VIEW collview3 AS SELECT a, lower((x || x) COLLATE "C") FROM collate_test10;

SELECT table_name, view_definition FROM information_schema.views
  WHERE table_name LIKE 'collview%' ORDER BY 1;

SELECT a, coalesce(b, 'foo') FROM collate_test1 ORDER BY 2;

SELECT a, coalesce(b, 'foo') FROM collate_test2 ORDER BY 2;

SELECT a, coalesce(b, 'foo') FROM collate_test3 ORDER BY 2;

SELECT a, lower(coalesce(x, 'foo')), lower(coalesce(y, 'foo')) FROM collate_test10;

SELECT a, b, greatest(b, 'CCC') FROM collate_test1 ORDER BY 3;

SELECT a, b, greatest(b, 'CCC') FROM collate_test2 ORDER BY 3;

SELECT a, b, greatest(b, 'CCC') FROM collate_test3 ORDER BY 3;

SELECT a, x, y, lower(greatest(x, 'foo')), lower(greatest(y, 'foo')) FROM collate_test10;

SELECT a, nullif(b, 'abc') FROM collate_test1 ORDER BY 2;

SELECT a, nullif(b, 'abc') FROM collate_test2 ORDER BY 2;

SELECT a, nullif(b, 'abc') FROM collate_test3 ORDER BY 2;

SELECT a, lower(nullif(x, 'foo')), lower(nullif(y, 'foo')) FROM collate_test10;

SELECT a, CASE b WHEN 'abc' THEN 'abcd' ELSE b END FROM collate_test1 ORDER BY 2;

SELECT a, CASE b WHEN 'abc' THEN 'abcd' ELSE b END FROM collate_test2 ORDER BY 2;

SELECT a, CASE b WHEN 'abc' THEN 'abcd' ELSE b END FROM collate_test3 ORDER BY 2;

CREATE DOMAIN testdomain AS text;

SELECT a, b::testdomain FROM collate_test1 ORDER BY 2;

SELECT a, b::testdomain FROM collate_test2 ORDER BY 2;

SELECT a, b::testdomain FROM collate_test3 ORDER BY 2;

SELECT a, b::testdomain_sv FROM collate_test3 ORDER BY 2;

SELECT a, lower(x::testdomain), lower(y::testdomain) FROM collate_test10;

SELECT min(b), max(b) FROM collate_test1;

SELECT min(b), max(b) FROM collate_test2;

SELECT min(b), max(b) FROM collate_test3;

SELECT array_agg(b ORDER BY b) FROM collate_test1;

SELECT array_agg(b ORDER BY b) FROM collate_test2;

SELECT array_agg(b ORDER BY b) FROM collate_test3;

SELECT a, b FROM collate_test1 UNION ALL SELECT a, b FROM collate_test1 ORDER BY 2;

SELECT a, b FROM collate_test2 UNION SELECT a, b FROM collate_test2 ORDER BY 2;

SELECT a, b FROM collate_test3 WHERE a > 1 ORDER BY 2;

SELECT a, b FROM collate_test3 EXCEPT SELECT a, b FROM collate_test3 WHERE a < 2 ORDER BY 2;

SELECT a, b FROM collate_test1 UNION ALL SELECT a, b FROM collate_test3 ORDER BY 2;

SELECT a, b FROM collate_test1 UNION ALL SELECT a, b FROM collate_test3;

SELECT a, b FROM collate_test1 UNION SELECT a, b FROM collate_test3 ORDER BY 2;

SELECT a, b COLLATE "C" FROM collate_test1 UNION SELECT a, b FROM collate_test3 ORDER BY 2;

SELECT a, b FROM collate_test3 ORDER BY 2;

SELECT a, b FROM collate_test1 EXCEPT SELECT a, b FROM collate_test3 ORDER BY 2;

CREATE TABLE test_u AS SELECT a, b FROM collate_test1 UNION ALL SELECT a, b FROM collate_test3;

select x < y from collate_test10;

select x || y from collate_test10;

select x, y from collate_test10 order by x || y;

WITH RECURSIVE foo(x) AS
   (SELECT x FROM (VALUES('a' COLLATE "en_US"),('b')) t(x)
   UNION ALL
   SELECT (x || 'c') COLLATE "de_DE" FROM foo WHERE length(x) < 10)
SELECT * FROM foo;

SELECT a, CAST(b AS varchar) FROM collate_test1 ORDER BY 2;

SELECT a, CAST(b AS varchar) FROM collate_test2 ORDER BY 2;

SELECT a, CAST(b AS varchar) FROM collate_test3 ORDER BY 2;

CREATE FUNCTION mylt (text, text) RETURNS boolean LANGUAGE sql
    AS $$ select $1 < $2 $$;

CREATE FUNCTION mylt_noninline (text, text) RETURNS boolean LANGUAGE sql
    AS $$ select $1 < $2 limit 1 $$;

CREATE FUNCTION mylt_plpgsql (text, text) RETURNS boolean LANGUAGE plpgsql
    AS $$ begin return $1 < $2; end $$;

SELECT a.b AS a, b.b AS b, a.b < b.b AS lt,
       mylt(a.b, b.b), mylt_noninline(a.b, b.b), mylt_plpgsql(a.b, b.b)
FROM collate_test1 a, collate_test1 b
ORDER BY a.b, b.b;

SELECT a.b AS a, b.b AS b, a.b < b.b COLLATE "C" AS lt,
       mylt(a.b, b.b COLLATE "C"), mylt_noninline(a.b, b.b COLLATE "C"),
       mylt_plpgsql(a.b, b.b COLLATE "C")
FROM collate_test1 a, collate_test1 b
ORDER BY a.b, b.b;

CREATE FUNCTION mylt2 (x text, y text) RETURNS boolean LANGUAGE plpgsql AS $$
declare
  xx text := x;
  yy text := y;
begin
  return xx < yy;
end
$$;

SELECT mylt2('a', 'B' collate "en_US") as t, mylt2('a', 'B' collate "C") as f;

CREATE OR REPLACE FUNCTION
  mylt2 (x text, y text) RETURNS boolean LANGUAGE plpgsql AS $$
declare
  xx text COLLATE "POSIX" := x;
  yy text := y;
begin
  return xx < yy;
end
$$;

SELECT mylt2('a', 'B') as f;

SELECT mylt2('a', 'B' collate "C") as fail;

SELECT mylt2('a', 'B' collate "POSIX") as f;

SELECT * FROM unnest((SELECT array_agg(b ORDER BY b) FROM collate_test1)) ORDER BY 1;

SELECT * FROM unnest((SELECT array_agg(b ORDER BY b) FROM collate_test2)) ORDER BY 1;

SELECT * FROM unnest((SELECT array_agg(b ORDER BY b) FROM collate_test3)) ORDER BY 1;

CREATE FUNCTION dup (anyelement) RETURNS anyelement
    AS 'select $1' LANGUAGE sql;

SELECT a, dup(b) FROM collate_test1 ORDER BY 2;

SELECT a, dup(b) FROM collate_test2 ORDER BY 2;

SELECT a, dup(b) FROM collate_test3 ORDER BY 2;

CREATE INDEX collate_test1_idx1 ON collate_test1 (b);

CREATE INDEX collate_test1_idx2 ON collate_test1 (b COLLATE "C");

CREATE INDEX collate_test1_idx3 ON collate_test1 ((b COLLATE "C"));

CREATE INDEX collate_test1_idx4 ON collate_test1 (((b||'foo') COLLATE "POSIX"));

CREATE INDEX collate_test1_idx5 ON collate_test1 (a COLLATE "C");

CREATE INDEX collate_test1_idx6 ON collate_test1 ((a COLLATE "C"));

SELECT relname, pg_get_indexdef(oid) FROM pg_class WHERE relname LIKE 'collate_test%_idx%' ORDER BY 1;

CREATE ROLE regress_test_role;

CREATE SCHEMA test_schema;

do $$
BEGIN
  EXECUTE 'CREATE COLLATION test0 (locale = ' ||
          quote_literal((SELECT datcollate FROM pg_database WHERE datname = current_database())) || ');';
END
$$;

CREATE COLLATION test0 FROM "C";

CREATE COLLATION IF NOT EXISTS test0 FROM "C";

CREATE COLLATION IF NOT EXISTS test0 (locale = 'foo');

do $$
BEGIN
  EXECUTE 'CREATE COLLATION test1 (lc_collate = ' ||
          quote_literal((SELECT datcollate FROM pg_database WHERE datname = current_database())) ||
          ', lc_ctype = ' ||
          quote_literal((SELECT datctype FROM pg_database WHERE datname = current_database())) || ');';
END
$$;

CREATE COLLATION test3 (lc_collate = 'en_US.utf8');

CREATE COLLATION testx (locale = 'nonsense');

CREATE COLLATION test4 FROM nonsense;

CREATE COLLATION test5 FROM test0;

SELECT collname FROM pg_collation WHERE collname LIKE 'test%' ORDER BY 1;

ALTER COLLATION test1 RENAME TO test11;

ALTER COLLATION test0 RENAME TO test11;

ALTER COLLATION test1 RENAME TO test22;

ALTER COLLATION test11 OWNER TO regress_test_role;

ALTER COLLATION test11 OWNER TO nonsense;

ALTER COLLATION test11 SET SCHEMA test_schema;

COMMENT ON COLLATION test0 IS 'US English';

SELECT collname, nspname, obj_description(pg_collation.oid, 'pg_collation')
    FROM pg_collation JOIN pg_namespace ON (collnamespace = pg_namespace.oid)
    WHERE collname LIKE 'test%'
    ORDER BY 1;

DROP COLLATION test0, test_schema.test11, test5;

DROP COLLATION test0;

DROP COLLATION IF EXISTS test0;

SELECT collname FROM pg_collation WHERE collname LIKE 'test%';

DROP SCHEMA test_schema;

DROP ROLE regress_test_role;

ALTER COLLATION "en_US" REFRESH VERSION;

SELECT current_database() AS datname ;

CREATE COLLATION test0 FROM "C";

CREATE TABLE collate_dep_test1 (a int, b text COLLATE test0);

CREATE DOMAIN collate_dep_dom1 AS text COLLATE test0;

CREATE TYPE collate_dep_test2 AS (x int, y text COLLATE test0);

CREATE VIEW collate_dep_test3 AS SELECT text 'foo' COLLATE test0 AS foo;

CREATE TABLE collate_dep_test4t (a int, b text);

CREATE INDEX collate_dep_test4i ON collate_dep_test4t (b COLLATE test0);

DROP COLLATION test0 RESTRICT;

DROP COLLATION test0 CASCADE;

DROP TABLE collate_dep_test1, collate_dep_test4t;

DROP TYPE collate_dep_test2;

create type textrange_c as range(subtype=text, collation="C");

create type textrange_en_us as range(subtype=text, collation="en_US");

select textrange_c('A','Z') @> 'b'::text;

select textrange_en_us('A','Z') @> 'b'::text;

drop type textrange_c;

drop type textrange_en_us;

SELECT * FROM collate_test2 ORDER BY b COLLATE UCS_BASIC;

CREATE COLLATION ctest_det (locale = 'en_US.utf8', deterministic = true);

CREATE COLLATION ctest_nondet (locale = 'en_US.utf8', deterministic = false);

SET client_min_messages TO warning;

DROP SCHEMA collate_tests CASCADE;
