SET max_parallel_maintenance_workers = 0;

SET max_parallel_workers = 0;

CREATE TEMP TABLE abbrev_abort_uuids (
    id serial not null,
    abort_increasing uuid,
    abort_decreasing uuid,
    noabort_increasing uuid,
    noabort_decreasing uuid);

INSERT INTO abbrev_abort_uuids (abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing)
    SELECT
        ('00000000-0000-0000-0000-'||to_char(g.i, '000000000000FM'))::uuid abort_increasing,
        ('00000000-0000-0000-0000-'||to_char(20000 - g.i, '000000000000FM'))::uuid abort_decreasing,
        (to_char(g.i % 10009, '00000000FM')||'-0000-0000-0000-'||to_char(g.i, '000000000000FM'))::uuid noabort_increasing,
        (to_char(((20000 - g.i) % 10009), '00000000FM')||'-0000-0000-0000-'||to_char(20000 - g.i, '000000000000FM'))::uuid noabort_decreasing
    FROM generate_series(0, 20000, 1) g(i);

INSERT INTO abbrev_abort_uuids(id) VALUES(0);

INSERT INTO abbrev_abort_uuids DEFAULT VALUES;

INSERT INTO abbrev_abort_uuids DEFAULT VALUES;

INSERT INTO abbrev_abort_uuids (abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing)
    SELECT abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing
    FROM abbrev_abort_uuids
    WHERE (id < 10 OR id > 19990) AND id % 3 = 0 AND abort_increasing is not null;

SELECT abort_increasing, abort_decreasing FROM abbrev_abort_uuids ORDER BY abort_increasing OFFSET 20000 - 4;

SELECT abort_increasing, abort_decreasing FROM abbrev_abort_uuids ORDER BY abort_decreasing NULLS FIRST OFFSET 20000 - 4;

SELECT noabort_increasing, noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_increasing OFFSET 20000 - 4;

SELECT noabort_increasing, noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_decreasing NULLS FIRST OFFSET 20000 - 4;

SELECT abort_increasing, noabort_increasing FROM abbrev_abort_uuids ORDER BY abort_increasing LIMIT 5;

SELECT abort_increasing, noabort_increasing FROM abbrev_abort_uuids ORDER BY noabort_increasing NULLS FIRST LIMIT 5;

CREATE INDEX abbrev_abort_uuids__noabort_increasing_idx ON abbrev_abort_uuids (noabort_increasing);

CREATE INDEX abbrev_abort_uuids__noabort_decreasing_idx ON abbrev_abort_uuids (noabort_decreasing);

SELECT id, noabort_increasing, noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_increasing LIMIT 5;

SELECT id, noabort_increasing, noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_increasing LIMIT 5;

SELECT id, noabort_increasing, noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_decreasing LIMIT 5;

SELECT id, noabort_increasing, noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_decreasing LIMIT 5;

CREATE INDEX abbrev_abort_uuids__abort_increasing_idx ON abbrev_abort_uuids (abort_increasing);

CREATE INDEX abbrev_abort_uuids__abort_decreasing_idx ON abbrev_abort_uuids (abort_decreasing);

SELECT id, abort_increasing, abort_decreasing FROM abbrev_abort_uuids ORDER BY abort_increasing LIMIT 5;

SELECT id, abort_increasing, abort_decreasing FROM abbrev_abort_uuids ORDER BY abort_increasing LIMIT 5;

SELECT id, abort_increasing, abort_decreasing FROM abbrev_abort_uuids ORDER BY abort_decreasing LIMIT 5;

SELECT id, abort_increasing, abort_decreasing FROM abbrev_abort_uuids ORDER BY abort_decreasing LIMIT 5;

BEGIN;

SET LOCAL enable_indexscan = false;

CLUSTER abbrev_abort_uuids USING abbrev_abort_uuids__abort_increasing_idx;

SELECT id, abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing
FROM abbrev_abort_uuids
ORDER BY ctid LIMIT 5;

SELECT id, abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing
FROM abbrev_abort_uuids
ORDER BY ctid DESC LIMIT 5;

ROLLBACK;

BEGIN;

SET LOCAL enable_indexscan = false;

CLUSTER abbrev_abort_uuids USING abbrev_abort_uuids__abort_decreasing_idx;

SELECT id, abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing
FROM abbrev_abort_uuids
ORDER BY ctid LIMIT 5;

SELECT id, abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing
FROM abbrev_abort_uuids
ORDER BY ctid DESC LIMIT 5;

ROLLBACK;

BEGIN;

SET LOCAL enable_indexscan = false;

CLUSTER abbrev_abort_uuids USING abbrev_abort_uuids__noabort_increasing_idx;

SELECT id, abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing
FROM abbrev_abort_uuids
ORDER BY ctid LIMIT 5;

SELECT id, abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing
FROM abbrev_abort_uuids
ORDER BY ctid DESC LIMIT 5;

ROLLBACK;

BEGIN;

SET LOCAL enable_indexscan = false;

CLUSTER abbrev_abort_uuids USING abbrev_abort_uuids__noabort_decreasing_idx;

SELECT id, abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing
FROM abbrev_abort_uuids
ORDER BY ctid LIMIT 5;

SELECT id, abort_increasing, abort_decreasing, noabort_increasing, noabort_decreasing
FROM abbrev_abort_uuids
ORDER BY ctid DESC LIMIT 5;

ROLLBACK;

SELECT LEFT(a,10),b FROM
    (VALUES(REPEAT('a', 512 * 1024),1),(REPEAT('b', 512 * 1024),2)) v(a,b)
ORDER BY v.a DESC;

BEGIN;

SET LOCAL enable_indexscan = false;

EXPLAIN (COSTS OFF) DECLARE c SCROLL CURSOR FOR SELECT noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_decreasing;

DECLARE c SCROLL CURSOR FOR SELECT noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_decreasing;

FETCH NEXT FROM c;

FETCH NEXT FROM c;

FETCH BACKWARD FROM c;

FETCH BACKWARD FROM c;

FETCH BACKWARD FROM c;

FETCH BACKWARD FROM c;

FETCH NEXT FROM c;

FETCH LAST FROM c;

FETCH BACKWARD FROM c;

FETCH NEXT FROM c;

FETCH NEXT FROM c;

FETCH NEXT FROM c;

FETCH BACKWARD FROM c;

FETCH NEXT FROM c;

COMMIT;

BEGIN;

SET LOCAL enable_indexscan = false;

SET LOCAL work_mem = '100kB';

EXPLAIN (COSTS OFF) DECLARE c SCROLL CURSOR FOR SELECT noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_decreasing;

DECLARE c SCROLL CURSOR FOR SELECT noabort_decreasing FROM abbrev_abort_uuids ORDER BY noabort_decreasing;

FETCH NEXT FROM c;

FETCH NEXT FROM c;

FETCH BACKWARD FROM c;

FETCH BACKWARD FROM c;

FETCH BACKWARD FROM c;

FETCH BACKWARD FROM c;

FETCH NEXT FROM c;

FETCH LAST FROM c;

FETCH BACKWARD FROM c;

FETCH NEXT FROM c;

FETCH NEXT FROM c;

FETCH NEXT FROM c;

FETCH BACKWARD FROM c;

FETCH NEXT FROM c;

COMMIT;

SELECT
    -- fixed-width by-value datum
    (array_agg(id ORDER BY id DESC NULLS FIRST))[0:5],
    -- fixed-width by-ref datum
    (array_agg(abort_increasing ORDER BY abort_increasing DESC NULLS LAST))[0:5],
    -- variable-width datum
    (array_agg(id::text ORDER BY id::text DESC NULLS LAST))[0:5],
    -- fixed width by-value datum tuplesort
    percentile_disc(0.99) WITHIN GROUP (ORDER BY id),
    -- ensure state is shared
    percentile_disc(0.01) WITHIN GROUP (ORDER BY id),
    -- fixed width by-ref datum tuplesort
    percentile_disc(0.8) WITHIN GROUP (ORDER BY abort_increasing),
    -- variable width by-ref datum tuplesort
    percentile_disc(0.2) WITHIN GROUP (ORDER BY id::text),
    -- multi-column tuplesort
    rank('00000000-0000-0000-0000-000000000000', '2', '2') WITHIN GROUP (ORDER BY noabort_increasing, id, id::text)
FROM (
    SELECT * FROM abbrev_abort_uuids
    UNION ALL
    SELECT NULL, NULL, NULL, NULL, NULL) s;

BEGIN;

SET LOCAL work_mem = '100kB';

SELECT
    (array_agg(id ORDER BY id DESC NULLS FIRST))[0:5],
    (array_agg(abort_increasing ORDER BY abort_increasing DESC NULLS LAST))[0:5],
    (array_agg(id::text ORDER BY id::text DESC NULLS LAST))[0:5],
    percentile_disc(0.99) WITHIN GROUP (ORDER BY id),
    percentile_disc(0.01) WITHIN GROUP (ORDER BY id),
    percentile_disc(0.8) WITHIN GROUP (ORDER BY abort_increasing),
    percentile_disc(0.2) WITHIN GROUP (ORDER BY id::text),
    rank('00000000-0000-0000-0000-000000000000', '2', '2') WITHIN GROUP (ORDER BY noabort_increasing, id, id::text)
FROM (
    SELECT * FROM abbrev_abort_uuids
    UNION ALL
    SELECT NULL, NULL, NULL, NULL, NULL) s;

ROLLBACK;

CREATE TEMP TABLE test_mark_restore(col1 int, col2 int, col12 int);

INSERT INTO test_mark_restore(col1, col2, col12)
   SELECT a.i, b.i, a.i * b.i FROM generate_series(1, 500) a(i), generate_series(1, 5) b(i);

BEGIN;

SET LOCAL enable_nestloop = off;

SET LOCAL enable_hashjoin = off;

SET LOCAL enable_material = off;

SELECT $$
    SELECT col12, count(distinct a.col1), count(distinct a.col2), count(distinct b.col1), count(distinct b.col2), count(*)
    FROM test_mark_restore a
        JOIN test_mark_restore b USING(col12)
    GROUP BY 1
    HAVING count(*) > 1
    ORDER BY 2 DESC, 1 DESC, 3 DESC, 4 DESC, 5 DESC, 6 DESC
    LIMIT 10
$$ AS qry ;

SET LOCAL work_mem = '100kB';

COMMIT;
