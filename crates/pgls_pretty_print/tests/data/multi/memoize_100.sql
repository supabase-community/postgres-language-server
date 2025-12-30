create function explain_memoize(query text, hide_hitmiss bool) returns setof text
language plpgsql as
$$
declare
    ln text;
begin
    for ln in
        execute format('explain (analyze, costs off, summary off, timing off, buffers off) %s',
            query)
    loop
        if hide_hitmiss = true then
                ln := regexp_replace(ln, 'Hits: 0', 'Hits: Zero');
                ln := regexp_replace(ln, 'Hits: \d+', 'Hits: N');
                ln := regexp_replace(ln, 'Misses: 0', 'Misses: Zero');
                ln := regexp_replace(ln, 'Misses: \d+', 'Misses: N');
        end if;
        ln := regexp_replace(ln, 'Evictions: 0', 'Evictions: Zero');
        ln := regexp_replace(ln, 'Evictions: \d+', 'Evictions: N');
        ln := regexp_replace(ln, 'Memory Usage: \d+', 'Memory Usage: N');
        ln := regexp_replace(ln, 'Heap Fetches: \d+', 'Heap Fetches: N');
        ln := regexp_replace(ln, 'loops=\d+', 'loops=N');
        ln := regexp_replace(ln, 'Index Searches: \d+', 'Index Searches: N');
        ln := regexp_replace(ln, 'Memory: \d+kB', 'Memory: NkB');
        return next ln;
    end loop;
end;
$$;

SET enable_hashjoin TO off;

SET enable_bitmapscan TO off;

SELECT explain_memoize('
SELECT COUNT(*),AVG(t1.unique1) FROM tenk1 t1
INNER JOIN tenk1 t2 ON t1.unique1 = t2.twenty
WHERE t2.unique1 < 1000;', false);

SELECT COUNT(*),AVG(t1.unique1) FROM tenk1 t1
INNER JOIN tenk1 t2 ON t1.unique1 = t2.twenty
WHERE t2.unique1 < 1000;

SELECT explain_memoize('
SELECT COUNT(*),AVG(t2.unique1) FROM tenk1 t1,
LATERAL (SELECT t2.unique1 FROM tenk1 t2
         WHERE t1.twenty = t2.unique1 OFFSET 0) t2
WHERE t1.unique1 < 1000;', false);

SELECT COUNT(*),AVG(t2.unique1) FROM tenk1 t1,
LATERAL (SELECT t2.unique1 FROM tenk1 t2
         WHERE t1.twenty = t2.unique1 OFFSET 0) t2
WHERE t1.unique1 < 1000;

SELECT explain_memoize('
SELECT COUNT(*),AVG(t2.t1two) FROM tenk1 t1 LEFT JOIN
LATERAL (
    SELECT t1.two as t1two, * FROM tenk1 t2 WHERE t2.unique1 < 4 OFFSET 0
) t2
ON t1.two = t2.two
WHERE t1.unique1 < 10;', false);

SELECT COUNT(*),AVG(t2.t1two) FROM tenk1 t1 LEFT JOIN
LATERAL (
    SELECT t1.two as t1two, * FROM tenk1 t2 WHERE t2.unique1 < 4 OFFSET 0
) t2
ON t1.two = t2.two
WHERE t1.unique1 < 10;

SELECT explain_memoize('
SELECT COUNT(*), AVG(t1.twenty) FROM tenk1 t1 LEFT JOIN
LATERAL (SELECT t1.two+1 AS c1, t2.unique1 AS c2 FROM tenk1 t2) s ON TRUE
WHERE s.c1 = s.c2 AND t1.unique1 < 1000;', false);

SELECT COUNT(*), AVG(t1.twenty) FROM tenk1 t1 LEFT JOIN
LATERAL (SELECT t1.two+1 AS c1, t2.unique1 AS c2 FROM tenk1 t2) s ON TRUE
WHERE s.c1 = s.c2 AND t1.unique1 < 1000;

SELECT explain_memoize('
SELECT COUNT(*), AVG(t1.twenty) FROM tenk1 t1 LEFT JOIN
LATERAL (SELECT t1.twenty AS c1, t2.unique1 AS c2, t2.two FROM tenk1 t2) s
ON t1.two = s.two
WHERE s.c1 = s.c2 AND t1.unique1 < 1000;', false);

SELECT COUNT(*), AVG(t1.twenty) FROM tenk1 t1 LEFT JOIN
LATERAL (SELECT t1.twenty AS c1, t2.unique1 AS c2, t2.two FROM tenk1 t2) s
ON t1.two = s.two
WHERE s.c1 = s.c2 AND t1.unique1 < 1000;

SET enable_mergejoin TO off;

CREATE TABLE expr_key (x numeric, t text);

INSERT INTO expr_key (x, t)
SELECT d1::numeric, d1::text FROM (
    SELECT round((d / pi())::numeric, 7) AS d1 FROM generate_series(1, 20) AS d
) t;

INSERT INTO expr_key SELECT * FROM expr_key;

CREATE INDEX expr_key_idx_x_t ON expr_key (x, t);

VACUUM ANALYZE expr_key;

SELECT explain_memoize('
SELECT * FROM expr_key t1 INNER JOIN expr_key t2
ON t1.x = t2.t::numeric AND t1.t::numeric = t2.x;', false);

DROP TABLE expr_key;

SET work_mem TO '64kB';

SET hash_mem_multiplier TO 1.0;

SELECT explain_memoize('
SELECT COUNT(*),AVG(t1.unique1) FROM tenk1 t1
INNER JOIN tenk1 t2 ON t1.unique1 = t2.thousand
WHERE t2.unique1 < 1200;', true);

CREATE TABLE flt (f float);

CREATE INDEX flt_f_idx ON flt (f);

INSERT INTO flt VALUES('-0.0'::float),('+0.0'::float);

ANALYZE flt;

SET enable_seqscan TO off;

SELECT explain_memoize('
SELECT * FROM flt f1 INNER JOIN flt f2 ON f1.f = f2.f;', false);

SELECT explain_memoize('
SELECT * FROM flt f1 INNER JOIN flt f2 ON f1.f >= f2.f;', false);

DROP TABLE flt;

CREATE TABLE strtest (n name, t text);

CREATE INDEX strtest_n_idx ON strtest (n);

CREATE INDEX strtest_t_idx ON strtest (t);

INSERT INTO strtest VALUES('one','one'),('two','two'),('three',repeat(fipshash('three'),100));

INSERT INTO strtest SELECT * FROM strtest;

ANALYZE strtest;

SELECT explain_memoize('
SELECT * FROM strtest s1 INNER JOIN strtest s2 ON s1.n >= s2.n;', false);

SELECT explain_memoize('
SELECT * FROM strtest s1 INNER JOIN strtest s2 ON s1.t >= s2.t;', false);

DROP TABLE strtest;

SET enable_partitionwise_join TO on;

CREATE TABLE prt (a int) PARTITION BY RANGE(a);

CREATE TABLE prt_p1 PARTITION OF prt FOR VALUES FROM (0) TO (10);

CREATE TABLE prt_p2 PARTITION OF prt FOR VALUES FROM (10) TO (20);

INSERT INTO prt VALUES (0), (0), (0), (0);

INSERT INTO prt VALUES (10), (10), (10), (10);

CREATE INDEX iprt_p1_a ON prt_p1 (a);

CREATE INDEX iprt_p2_a ON prt_p2 (a);

ANALYZE prt;

SELECT explain_memoize('
SELECT * FROM prt t1 INNER JOIN prt t2 ON t1.a = t2.a;', false);

SET enable_partitionwise_join TO off;

SELECT explain_memoize('
SELECT * FROM prt_p1 t1 INNER JOIN
(SELECT * FROM prt_p1 UNION ALL SELECT * FROM prt_p2) t2
ON t1.a = t2.a;', false);

DROP TABLE prt;

RESET enable_partitionwise_join;

SELECT unique1 FROM tenk1 t0
WHERE unique1 < 3
  AND EXISTS (
	SELECT 1 FROM tenk1 t1
	INNER JOIN tenk1 t2 ON t1.unique1 = t2.hundred
	WHERE t0.ten = t1.twenty AND t0.two <> t2.four OFFSET 0);

SELECT unique1 FROM tenk1 t0
WHERE unique1 < 3
  AND EXISTS (
	SELECT 1 FROM tenk1 t1
	INNER JOIN tenk1 t2 ON t1.unique1 = t2.hundred
	WHERE t0.ten = t1.twenty AND t0.two <> t2.four OFFSET 0);

RESET enable_seqscan;

RESET enable_mergejoin;

RESET work_mem;

RESET hash_mem_multiplier;

RESET enable_bitmapscan;

RESET enable_hashjoin;

SET min_parallel_table_scan_size TO 0;

SET parallel_setup_cost TO 0;

SET parallel_tuple_cost TO 0;

SET max_parallel_workers_per_gather TO 2;

SELECT COUNT(*),AVG(t2.unique1) FROM tenk1 t1,
LATERAL (SELECT t2.unique1 FROM tenk1 t2 WHERE t1.twenty = t2.unique1) t2
WHERE t1.unique1 < 1000;

SELECT COUNT(*),AVG(t2.unique1) FROM tenk1 t1,
LATERAL (SELECT t2.unique1 FROM tenk1 t2 WHERE t1.twenty = t2.unique1) t2
WHERE t1.unique1 < 1000;

RESET max_parallel_workers_per_gather;

RESET parallel_tuple_cost;

RESET parallel_setup_cost;

RESET min_parallel_table_scan_size;

CREATE TABLE tab_anti (a int, b boolean);

INSERT INTO tab_anti SELECT i%3, false FROM generate_series(1,100)i;

ANALYZE tab_anti;

SELECT explain_memoize('
SELECT COUNT(*) FROM tab_anti t1 LEFT JOIN
LATERAL (SELECT DISTINCT ON (a) a, b, t1.a AS x FROM tab_anti t2) t2
ON t1.a+1 = t2.a
WHERE t2.a IS NULL;', false);

SELECT COUNT(*) FROM tab_anti t1 LEFT JOIN
LATERAL (SELECT DISTINCT ON (a) a, b, t1.a AS x FROM tab_anti t2) t2
ON t1.a+1 = t2.a
WHERE t2.a IS NULL;

SELECT * FROM tab_anti t1 WHERE t1.a IN
 (SELECT a FROM tab_anti t2 WHERE t2.b IN
  (SELECT t1.b FROM tab_anti t3 WHERE t2.a > 1 OFFSET 0));

DROP TABLE tab_anti;
