SELECT DISTINCT two FROM onek ORDER BY 1;

SELECT DISTINCT ten FROM onek ORDER BY 1;

SELECT DISTINCT string4 FROM onek ORDER BY 1;

SELECT DISTINCT two, string4, ten
   FROM onek
   ORDER BY two using <, string4 using <, ten using <;

SELECT DISTINCT p.age FROM person* p ORDER BY age using >;

SELECT count(*) FROM
  (SELECT DISTINCT two, four, two FROM tenk1) ss;

SELECT count(*) FROM
  (SELECT DISTINCT two, four, two FROM tenk1) ss;

SET work_mem='64kB';

SET enable_hashagg=FALSE;

SET jit_above_cost=0;

SELECT DISTINCT g%1000 FROM generate_series(0,9999) g;

CREATE TABLE distinct_group_1 AS
SELECT DISTINCT g%1000 FROM generate_series(0,9999) g;

SET jit_above_cost TO DEFAULT;

CREATE TABLE distinct_group_2 AS
SELECT DISTINCT (g%1000)::text FROM generate_series(0,9999) g;

SET enable_seqscan = 0;

SELECT DISTINCT hundred, two FROM tenk1;

RESET enable_seqscan;

SET enable_hashagg=TRUE;

SET enable_sort=FALSE;

SET jit_above_cost=0;

SELECT DISTINCT g%1000 FROM generate_series(0,9999) g;

CREATE TABLE distinct_hash_1 AS
SELECT DISTINCT g%1000 FROM generate_series(0,9999) g;

SET jit_above_cost TO DEFAULT;

CREATE TABLE distinct_hash_2 AS
SELECT DISTINCT (g%1000)::text FROM generate_series(0,9999) g;

SET enable_sort=TRUE;

SET work_mem TO DEFAULT;

(SELECT * FROM distinct_hash_1 EXCEPT SELECT * FROM distinct_group_1)
  UNION ALL
(SELECT * FROM distinct_group_1 EXCEPT SELECT * FROM distinct_hash_1);

(SELECT * FROM distinct_hash_1 EXCEPT SELECT * FROM distinct_group_1)
  UNION ALL
(SELECT * FROM distinct_group_1 EXCEPT SELECT * FROM distinct_hash_1);

DROP TABLE distinct_hash_1;

DROP TABLE distinct_hash_2;

DROP TABLE distinct_group_1;

DROP TABLE distinct_group_2;

SET parallel_tuple_cost=0;

SET parallel_setup_cost=0;

SET min_parallel_table_scan_size=0;

SET max_parallel_workers_per_gather=2;

SELECT DISTINCT four FROM tenk1;

SELECT DISTINCT four FROM tenk1;

CREATE OR REPLACE FUNCTION distinct_func(a INT) RETURNS INT AS $$
  BEGIN
    RETURN a;
  END;
$$ LANGUAGE plpgsql PARALLEL UNSAFE;

SELECT DISTINCT distinct_func(1) FROM tenk1;

CREATE OR REPLACE FUNCTION distinct_func(a INT) RETURNS INT AS $$
  BEGIN
    RETURN a;
  END;
$$ LANGUAGE plpgsql PARALLEL SAFE;

SELECT DISTINCT distinct_func(1) FROM tenk1;

RESET max_parallel_workers_per_gather;

RESET min_parallel_table_scan_size;

RESET parallel_setup_cost;

RESET parallel_tuple_cost;

SELECT DISTINCT four FROM tenk1 WHERE four = 0;

SELECT DISTINCT four FROM tenk1 WHERE four = 0;

SELECT DISTINCT four FROM tenk1 WHERE four = 0 AND two <> 0;

SELECT DISTINCT four FROM tenk1 WHERE four = 0 AND two <> 0;

SELECT DISTINCT four,1,2,3 FROM tenk1 WHERE four = 0;

SELECT DISTINCT four,1,2,3 FROM tenk1 WHERE four = 0;

SET parallel_setup_cost=0;

SET min_parallel_table_scan_size=0;

SET max_parallel_workers_per_gather=2;

SELECT DISTINCT four FROM tenk1 WHERE four = 10;

RESET max_parallel_workers_per_gather;

RESET min_parallel_table_scan_size;

RESET parallel_setup_cost;

CREATE TEMP TABLE disttable (f1 integer);

INSERT INTO DISTTABLE VALUES(1);

INSERT INTO DISTTABLE VALUES(2);

INSERT INTO DISTTABLE VALUES(3);

INSERT INTO DISTTABLE VALUES(NULL);

SELECT f1, f1 IS DISTINCT FROM 2 as "not 2" FROM disttable;

SELECT f1, f1 IS DISTINCT FROM NULL as "not null" FROM disttable;

SELECT f1, f1 IS DISTINCT FROM f1 as "false" FROM disttable;

SELECT f1, f1 IS DISTINCT FROM f1+1 as "not null" FROM disttable;

SELECT 1 IS DISTINCT FROM 2 as "yes";

SELECT 2 IS DISTINCT FROM 2 as "no";

SELECT 2 IS DISTINCT FROM null as "yes";

SELECT null IS DISTINCT FROM null as "no";

SELECT 1 IS NOT DISTINCT FROM 2 as "no";

SELECT 2 IS NOT DISTINCT FROM 2 as "yes";

SELECT 2 IS NOT DISTINCT FROM null as "no";

SELECT null IS NOT DISTINCT FROM null as "yes";

CREATE TABLE distinct_tbl (x int, y int);

INSERT INTO distinct_tbl SELECT i%10, i%10 FROM generate_series(1, 1000) AS i;

CREATE INDEX distinct_tbl_x_y_idx ON distinct_tbl (x, y);

ANALYZE distinct_tbl;

SET enable_hashagg TO OFF;

SELECT DISTINCT y, x FROM distinct_tbl;

SELECT DISTINCT y, x FROM distinct_tbl;

SELECT DISTINCT y, x FROM (SELECT * FROM distinct_tbl ORDER BY x) s;

SELECT DISTINCT y, x FROM (SELECT * FROM distinct_tbl ORDER BY x) s;

SET parallel_tuple_cost=0;

SET parallel_setup_cost=0;

SET min_parallel_table_scan_size=0;

SET min_parallel_index_scan_size=0;

SET max_parallel_workers_per_gather=2;

SELECT DISTINCT y, x FROM distinct_tbl limit 10;

SELECT DISTINCT y, x FROM distinct_tbl limit 10;

RESET max_parallel_workers_per_gather;

RESET min_parallel_index_scan_size;

RESET min_parallel_table_scan_size;

RESET parallel_setup_cost;

RESET parallel_tuple_cost;

SELECT DISTINCT y, x FROM distinct_tbl ORDER BY y;

SELECT DISTINCT y, x FROM distinct_tbl ORDER BY y;

RESET enable_hashagg;

DROP TABLE distinct_tbl;
