SELECT DISTINCT ON (string4) string4, two, ten
   FROM onek
   ORDER BY string4 using <, two using >, ten using <;

SELECT DISTINCT ON (string4, ten) string4, two, ten
   FROM onek
   ORDER BY string4 using <, two using <, ten using <;

SELECT DISTINCT ON (string4, ten) string4, ten, two
   FROM onek
   ORDER BY string4 using <, ten using >, two using <;

select distinct on (1) floor(random()) as r, f1 from int4_tbl order by 1,2;

SELECT DISTINCT ON (four) four,two
   FROM tenk1 WHERE four = 0 ORDER BY 1;

SELECT DISTINCT ON (four) four,two
   FROM tenk1 WHERE four = 0 ORDER BY 1;

SELECT DISTINCT ON (four) four,two
   FROM tenk1 WHERE four = 0 ORDER BY 1,2;

SELECT DISTINCT ON (four) four,hundred
   FROM tenk1 WHERE four = 0 ORDER BY 1,2;

CREATE TABLE distinct_on_tbl (x int, y int, z int);

INSERT INTO distinct_on_tbl SELECT i%10, i%10, i%10 FROM generate_series(1, 1000) AS i;

CREATE INDEX distinct_on_tbl_x_y_idx ON distinct_on_tbl (x, y);

ANALYZE distinct_on_tbl;

SET enable_hashagg TO OFF;

SELECT DISTINCT ON (y, x) x, y FROM distinct_on_tbl;

SELECT DISTINCT ON (y, x) x, y FROM distinct_on_tbl;

SELECT DISTINCT ON (y, x) x, y FROM (SELECT * FROM distinct_on_tbl ORDER BY x) s;

SELECT DISTINCT ON (y, x) x, y FROM (SELECT * FROM distinct_on_tbl ORDER BY x) s;

SELECT DISTINCT ON (y, x) x, y FROM distinct_on_tbl ORDER BY y;

SELECT DISTINCT ON (y, x) x, y FROM distinct_on_tbl ORDER BY y;

SELECT DISTINCT ON (y, x) x, y FROM (select * from distinct_on_tbl order by x, z, y) s ORDER BY y, x, z;

SELECT DISTINCT ON (y, x) x, y FROM (select * from distinct_on_tbl order by x, z, y) s ORDER BY y, x, z;

RESET enable_hashagg;

DROP TABLE distinct_on_tbl;
