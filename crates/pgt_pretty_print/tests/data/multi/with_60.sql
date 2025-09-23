WITH q1(x,y) AS (SELECT 1,2)
SELECT * FROM q1, q1 AS q2;

SELECT count(*) FROM (
  WITH q1(x) AS (SELECT random() FROM generate_series(1, 5))
    SELECT * FROM q1
  UNION
    SELECT * FROM q1
) ss;

WITH RECURSIVE t(n) AS (
    VALUES (1)
UNION ALL
    SELECT n+1 FROM t WHERE n < 100
)
SELECT sum(n) FROM t;

WITH RECURSIVE t(n) AS (
    SELECT (VALUES(1))
UNION ALL
    SELECT n+1 FROM t WHERE n < 5
)
SELECT * FROM t;

WITH RECURSIVE t(n) AS (
    VALUES ('01'::varbit)
UNION
    SELECT n || '10'::varbit FROM t WHERE n < '100'::varbit
)
SELECT n FROM t;

CREATE RECURSIVE VIEW nums (n) AS
    VALUES (1)
UNION ALL
    SELECT n+1 FROM nums WHERE n < 5;

SELECT * FROM nums;

CREATE OR REPLACE RECURSIVE VIEW nums (n) AS
    VALUES (1)
UNION ALL
    SELECT n+1 FROM nums WHERE n < 6;

SELECT * FROM nums;

WITH RECURSIVE t(n) AS (
    SELECT 1
UNION
    SELECT 10-n FROM t)
SELECT * FROM t;

WITH RECURSIVE t(n) AS (
    VALUES (1)
UNION ALL
    SELECT n+1 FROM t)
SELECT * FROM t LIMIT 10;

WITH RECURSIVE t(n) AS (
    SELECT 1
UNION
    SELECT n+1 FROM t)
SELECT * FROM t LIMIT 10;

WITH q AS (SELECT 'foo' AS x)
SELECT x, pg_typeof(x) FROM q;

WITH RECURSIVE t(n) AS (
    SELECT 'foo'
UNION ALL
    SELECT n || ' bar' FROM t WHERE length(n) < 20
)
SELECT n, pg_typeof(n) FROM t;

WITH RECURSIVE t(n) AS (
    SELECT '7'
UNION ALL
    SELECT n+1 FROM t WHERE n < 10
)
SELECT n, pg_typeof(n) FROM t;

WITH RECURSIVE w1(c1) AS
 (WITH w2(c2) AS
  (WITH w3(c3) AS
   (WITH w4(c4) AS
    (WITH w5(c5) AS
     (WITH RECURSIVE w6(c6) AS
      (WITH w6(c6) AS
       (WITH w8(c8) AS
        (SELECT 1)
        SELECT * FROM w8)
       SELECT * FROM w6)
      SELECT * FROM w6)
     SELECT * FROM w5)
    SELECT * FROM w4)
   SELECT * FROM w3)
  SELECT * FROM w2)
SELECT * FROM w1;

WITH RECURSIVE outermost(x) AS (
 SELECT 1
 UNION (WITH innermost1 AS (
  SELECT 2
  UNION (WITH innermost2 AS (
   SELECT 3
   UNION (WITH innermost3 AS (
    SELECT 4
    UNION (WITH innermost4 AS (
     SELECT 5
     UNION (WITH innermost5 AS (
      SELECT 6
      UNION (WITH innermost6 AS
       (SELECT 7)
       SELECT * FROM innermost6))
      SELECT * FROM innermost5))
     SELECT * FROM innermost4))
    SELECT * FROM innermost3))
   SELECT * FROM innermost2))
  SELECT * FROM outermost
  UNION SELECT * FROM innermost1)
 )
 SELECT * FROM outermost ORDER BY 1;

CREATE TEMP TABLE department (
	id INTEGER PRIMARY KEY,  -- department ID
	parent_department INTEGER REFERENCES department, -- upper department ID
	name TEXT -- department name
);

INSERT INTO department VALUES (0, NULL, 'ROOT');

INSERT INTO department VALUES (1, 0, 'A');

INSERT INTO department VALUES (2, 1, 'B');

INSERT INTO department VALUES (3, 2, 'C');

INSERT INTO department VALUES (4, 2, 'D');

INSERT INTO department VALUES (5, 0, 'E');

INSERT INTO department VALUES (6, 4, 'F');

INSERT INTO department VALUES (7, 5, 'G');

WITH RECURSIVE subdepartment AS
(
	-- non recursive term
	SELECT name as root_name, * FROM department WHERE name = 'A'

	UNION ALL

	-- recursive term
	SELECT sd.root_name, d.* FROM department AS d, subdepartment AS sd
		WHERE d.parent_department = sd.id
)
SELECT * FROM subdepartment ORDER BY name;

WITH RECURSIVE subdepartment(level, id, parent_department, name) AS
(
	-- non recursive term
	SELECT 1, * FROM department WHERE name = 'A'

	UNION ALL

	-- recursive term
	SELECT sd.level + 1, d.* FROM department AS d, subdepartment AS sd
		WHERE d.parent_department = sd.id
)
SELECT * FROM subdepartment ORDER BY name;

WITH RECURSIVE subdepartment(level, id, parent_department, name) AS
(
	-- non recursive term
	SELECT 1, * FROM department WHERE name = 'A'

	UNION ALL

	-- recursive term
	SELECT sd.level + 1, d.* FROM department AS d, subdepartment AS sd
		WHERE d.parent_department = sd.id
)
SELECT * FROM subdepartment WHERE level >= 2 ORDER BY name;

WITH RECURSIVE subdepartment AS
(
	-- note lack of recursive UNION structure
	SELECT * FROM department WHERE name = 'A'
)
SELECT * FROM subdepartment ORDER BY name;

WITH RECURSIVE subdepartment AS
(
	-- select all columns to prevent projection
	SELECT id, parent_department, name FROM department WHERE name = 'A'

	UNION

	-- joins do projection
	SELECT d.id, d.parent_department, d.name FROM department AS d
	INNER JOIN subdepartment AS sd ON d.parent_department = sd.id
)
SELECT * FROM subdepartment ORDER BY name;

SELECT count(*) FROM (
    WITH RECURSIVE t(n) AS (
        SELECT 1 UNION ALL SELECT n + 1 FROM t WHERE n < 500
    )
    SELECT * FROM t) AS t WHERE n < (
        SELECT count(*) FROM (
            WITH RECURSIVE t(n) AS (
                   SELECT 1 UNION ALL SELECT n + 1 FROM t WHERE n < 100
                )
            SELECT * FROM t WHERE n < 50000
         ) AS t WHERE n < 100);

WITH q1(x,y) AS (
    SELECT hundred, sum(ten) FROM tenk1 GROUP BY hundred
  )
SELECT count(*) FROM q1 WHERE y > (SELECT sum(y)/100 FROM q1 qsub);

SELECT * FROM subdepartment;

SELECT * FROM vsubdepartment ORDER BY name;

SELECT pg_get_viewdef('vsubdepartment'::regclass);

SELECT pg_get_viewdef('vsubdepartment'::regclass, true);

SELECT sum(n) FROM t;

with recursive q as (
      select * from department
    union all
      (with x as (select * from q)
       select * from x)
    )
select * from q limit 24;

with recursive q as (
      select * from department
    union all
      (with recursive x as (
           select * from department
         union all
           (select * from q union all select * from x)
        )
       select * from x)
    )
select * from q limit 32;

SELECT * FROM t;

CREATE TEMPORARY TABLE tree(
    id INTEGER PRIMARY KEY,
    parent_id INTEGER REFERENCES tree(id)
);

INSERT INTO tree
VALUES (1, NULL), (2, 1), (3,1), (4,2), (5,2), (6,2), (7,3), (8,3),
       (9,4), (10,4), (11,7), (12,7), (13,7), (14, 9), (15,11), (16,11);

WITH RECURSIVE t(id, path) AS (
    VALUES(1,ARRAY[]::integer[])
UNION ALL
    SELECT tree.id, t.path || tree.id
    FROM tree JOIN t ON (tree.parent_id = t.id)
)
SELECT t1.*, t2.* FROM t AS t1 JOIN t AS t2 ON
	(t1.path[1] = t2.path[1] AND
	array_upper(t1.path,1) = 1 AND
	array_upper(t2.path,1) > 1)
	ORDER BY t1.id, t2.id;

WITH RECURSIVE t(id, path) AS (
    VALUES(1,ARRAY[]::integer[])
UNION ALL
    SELECT tree.id, t.path || tree.id
    FROM tree JOIN t ON (tree.parent_id = t.id)
)
SELECT t1.id, count(t2.*) FROM t AS t1 JOIN t AS t2 ON
	(t1.path[1] = t2.path[1] AND
	array_upper(t1.path,1) = 1 AND
	array_upper(t2.path,1) > 1)
	GROUP BY t1.id
	ORDER BY t1.id;

WITH RECURSIVE t(id, path) AS (
    VALUES(1,ARRAY[]::integer[])
UNION ALL
    SELECT tree.id, t.path || tree.id
    FROM tree JOIN t ON (tree.parent_id = t.id)
)
SELECT t1.id, t2.path, t2 FROM t AS t1 JOIN t AS t2 ON
(t1.id=t2.id);

CREATE TEMP TABLE duplicates (a INT NOT NULL);

INSERT INTO duplicates VALUES(1), (1);

WITH RECURSIVE cte (a) as (
	SELECT a FROM duplicates
	UNION
	SELECT a FROM cte
)
SELECT a FROM cte;
