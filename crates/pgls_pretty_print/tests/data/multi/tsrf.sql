SELECT generate_series(1, 3);

SELECT generate_series(1, 3), generate_series(3,5);

SELECT generate_series(1, 2), generate_series(1,4);

SELECT generate_series(1, generate_series(1, 3));

SELECT * FROM generate_series(1, generate_series(1, 3));

SELECT generate_series(generate_series(1,3), generate_series(2, 4));

SELECT generate_series(1, generate_series(1, 3)), generate_series(2, 4);

SELECT generate_series(1, generate_series(1, 3)), generate_series(2, 4);

CREATE TABLE few(id int, dataa text, datab text);

INSERT INTO few VALUES(1, 'a', 'foo'),(2, 'a', 'bar'),(3, 'b', 'bar');

SELECT unnest(ARRAY[1, 2]) FROM few WHERE false;

SELECT unnest(ARRAY[1, 2]) FROM few WHERE false;

SELECT * FROM few f1,
  (SELECT unnest(ARRAY[1,2]) FROM few f2 WHERE false OFFSET 0) ss;

SELECT * FROM few f1,
  (SELECT unnest(ARRAY[1,2]) FROM few f2 WHERE false OFFSET 0) ss;

SELECT few.id, generate_series(1,3) g FROM few ORDER BY id DESC;

SELECT few.id, generate_series(1,3) g FROM few ORDER BY id, g DESC;

SELECT few.id, generate_series(1,3) g FROM few ORDER BY id, generate_series(1,3) DESC;

SELECT few.id FROM few ORDER BY id, generate_series(1,3) DESC;

SET enable_hashagg TO 0;

SELECT few.dataa, count(*), min(id), max(id), unnest('{1,1,3}'::int[]) FROM few WHERE few.id = 1 GROUP BY few.dataa;

SELECT few.dataa, count(*), min(id), max(id), unnest('{1,1,3}'::int[]) FROM few WHERE few.id = 1 GROUP BY few.dataa, unnest('{1,1,3}'::int[]);

SELECT few.dataa, count(*), min(id), max(id), unnest('{1,1,3}'::int[]) FROM few WHERE few.id = 1 GROUP BY few.dataa, 5;

RESET enable_hashagg;

SELECT dataa, generate_series(1,1), count(*) FROM few GROUP BY 1 HAVING count(*) > 1;

SELECT dataa, generate_series(1,1), count(*) FROM few GROUP BY 1, 2 HAVING count(*) > 1;

SELECT few.dataa, count(*) FROM few WHERE dataa = 'a' GROUP BY few.dataa ORDER BY 2;

SELECT few.dataa, count(*) FROM few WHERE dataa = 'a' GROUP BY few.dataa, unnest('{1,1,3}'::int[]) ORDER BY 2;

SELECT q1, case when q1 > 0 then generate_series(1,3) else 0 end FROM int8_tbl;

SELECT q1, coalesce(generate_series(1,3), 0) FROM int8_tbl;

SELECT min(generate_series(1, 3)) FROM few;

SELECT sum((3 = ANY(SELECT generate_series(1,4)))::int);

SELECT sum((3 = ANY(SELECT lag(x) over(order by x)
                    FROM generate_series(1,4) x))::int);

SELECT min(generate_series(1, 3)) OVER() FROM few;

SELECT id,lag(id) OVER(), count(*) OVER(), generate_series(1,3) FROM few;

SELECT SUM(count(*)) OVER(PARTITION BY generate_series(1,3) ORDER BY generate_series(1,3)), generate_series(1,3) g FROM few GROUP BY g;

SELECT few.dataa, count(*), min(id), max(id), generate_series(1,3) FROM few GROUP BY few.dataa ORDER BY 5, 1;

set enable_hashagg = false;

SELECT dataa, datab b, generate_series(1,2) g, count(*) FROM few GROUP BY CUBE(dataa, datab);

SELECT dataa, datab b, generate_series(1,2) g, count(*) FROM few GROUP BY CUBE(dataa, datab) ORDER BY dataa;

SELECT dataa, datab b, generate_series(1,2) g, count(*) FROM few GROUP BY CUBE(dataa, datab) ORDER BY g;

SELECT dataa, datab b, generate_series(1,2) g, count(*) FROM few GROUP BY CUBE(dataa, datab, g);

SELECT dataa, datab b, generate_series(1,2) g, count(*) FROM few GROUP BY CUBE(dataa, datab, g) ORDER BY dataa;

SELECT dataa, datab b, generate_series(1,2) g, count(*) FROM few GROUP BY CUBE(dataa, datab, g) ORDER BY g;

reset enable_hashagg;

select 'foo' as f, generate_series(1,2) as g from few order by 1;

select 'foo' as f, generate_series(1,2) as g from few order by 1;

CREATE TABLE fewmore AS SELECT generate_series(1,3) AS data;

INSERT INTO fewmore VALUES(generate_series(4,5));

SELECT * FROM fewmore;

UPDATE fewmore SET data = generate_series(4,9);

INSERT INTO fewmore VALUES(1) RETURNING generate_series(1,3);

VALUES(1, generate_series(1,2));

SELECT int4mul(generate_series(1,2), 10);

SELECT generate_series(1,3) IS DISTINCT FROM 2;

SELECT * FROM int4mul(generate_series(1,2), 10);

SELECT DISTINCT ON (a) a, b, generate_series(1,3) g
FROM (VALUES (3, 2), (3,1), (1,1), (1,4), (5,3), (5,1)) AS t(a, b);

SELECT DISTINCT ON (a) a, b, generate_series(1,3) g
FROM (VALUES (3, 2), (3,1), (1,1), (1,4), (5,3), (5,1)) AS t(a, b)
ORDER BY a, b DESC;

SELECT DISTINCT ON (a) a, b, generate_series(1,3) g
FROM (VALUES (3, 2), (3,1), (1,1), (1,4), (5,3), (5,1)) AS t(a, b)
ORDER BY a, b DESC, g DESC;

SELECT DISTINCT ON (a, b, g) a, b, generate_series(1,3) g
FROM (VALUES (3, 2), (3,1), (1,1), (1,4), (5,3), (5,1)) AS t(a, b)
ORDER BY a, b DESC, g DESC;

SELECT DISTINCT ON (g) a, b, generate_series(1,3) g
FROM (VALUES (3, 2), (3,1), (1,1), (1,4), (5,3), (5,1)) AS t(a, b);

SELECT a, generate_series(1,2) FROM (VALUES(1),(2),(3)) r(a) LIMIT 2 OFFSET 2;

SELECT 1 LIMIT generate_series(1,3);

SELECT (SELECT generate_series(1,3) LIMIT 1 OFFSET few.id) FROM few;

SELECT (SELECT generate_series(1,3) LIMIT 1 OFFSET g.i) FROM generate_series(0,3) g(i);

CREATE OPERATOR |@| (PROCEDURE = unnest, RIGHTARG = ANYARRAY);

SELECT |@|ARRAY[1,2,3];

select generate_series(1,3) as x, generate_series(1,3) + 1 as xp1;

select generate_series(1,3) as x, generate_series(1,3) + 1 as xp1;

select generate_series(1,3)+1 order by generate_series(1,3);

select generate_series(1,3)+1 order by generate_series(1,3);

select generate_series(1,3) as x, generate_series(3,6) + 1 as y;

select generate_series(1,3) as x, generate_series(3,6) + 1 as y;

DROP TABLE few;

DROP TABLE fewmore;
