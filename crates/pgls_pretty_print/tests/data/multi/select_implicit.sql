CREATE TABLE test_missing_target (a int, b int, c char(8), d char);

INSERT INTO test_missing_target VALUES (0, 1, 'XXXX', 'A');

INSERT INTO test_missing_target VALUES (1, 2, 'ABAB', 'b');

INSERT INTO test_missing_target VALUES (2, 2, 'ABAB', 'c');

INSERT INTO test_missing_target VALUES (3, 3, 'BBBB', 'D');

INSERT INTO test_missing_target VALUES (4, 3, 'BBBB', 'e');

INSERT INTO test_missing_target VALUES (5, 3, 'bbbb', 'F');

INSERT INTO test_missing_target VALUES (6, 4, 'cccc', 'g');

INSERT INTO test_missing_target VALUES (7, 4, 'cccc', 'h');

INSERT INTO test_missing_target VALUES (8, 4, 'CCCC', 'I');

INSERT INTO test_missing_target VALUES (9, 4, 'CCCC', 'j');

SELECT c, count(*) FROM test_missing_target GROUP BY test_missing_target.c ORDER BY c;

SELECT count(*) FROM test_missing_target GROUP BY test_missing_target.c ORDER BY c;

SELECT count(*) FROM test_missing_target GROUP BY a ORDER BY b;

SELECT count(*) FROM test_missing_target GROUP BY b ORDER BY b;

SELECT test_missing_target.b, count(*)
  FROM test_missing_target GROUP BY b ORDER BY b;

SELECT c FROM test_missing_target ORDER BY a;

SELECT count(*) FROM test_missing_target GROUP BY b ORDER BY b desc;

SELECT count(*) FROM test_missing_target ORDER BY 1 desc;

SELECT c, count(*) FROM test_missing_target GROUP BY 1 ORDER BY 1;

SELECT c, count(*) FROM test_missing_target GROUP BY 3;

SELECT count(*) FROM test_missing_target x, test_missing_target y
	WHERE x.a = y.a
	GROUP BY b ORDER BY b;

SELECT a, a FROM test_missing_target
	ORDER BY a;

SELECT a/2, a/2 FROM test_missing_target
	ORDER BY a/2;

SELECT a/2, a/2 FROM test_missing_target
	GROUP BY a/2 ORDER BY a/2;

SELECT x.b, count(*) FROM test_missing_target x, test_missing_target y
	WHERE x.a = y.a
	GROUP BY x.b ORDER BY x.b;

SELECT count(*) FROM test_missing_target x, test_missing_target y
	WHERE x.a = y.a
	GROUP BY x.b ORDER BY x.b;

CREATE TABLE test_missing_target2 AS
SELECT count(*)
FROM test_missing_target x, test_missing_target y
	WHERE x.a = y.a
	GROUP BY x.b ORDER BY x.b;

SELECT * FROM test_missing_target2;

SELECT a%2, count(b) FROM test_missing_target
GROUP BY test_missing_target.a%2
ORDER BY test_missing_target.a%2;

SELECT count(c) FROM test_missing_target
GROUP BY lower(test_missing_target.c)
ORDER BY lower(test_missing_target.c);

SELECT count(a) FROM test_missing_target GROUP BY a ORDER BY b;

SELECT count(b) FROM test_missing_target GROUP BY b/2 ORDER BY b/2;

SELECT lower(test_missing_target.c), count(c)
  FROM test_missing_target GROUP BY lower(c) ORDER BY lower(c);

SELECT a FROM test_missing_target ORDER BY upper(d);

SELECT count(b) FROM test_missing_target
	GROUP BY (b + 1) / 2 ORDER BY (b + 1) / 2 desc;

SELECT count(x.a) FROM test_missing_target x, test_missing_target y
	WHERE x.a = y.a
	GROUP BY b/2 ORDER BY b/2;

SELECT x.b/2, count(x.b) FROM test_missing_target x, test_missing_target y
	WHERE x.a = y.a
	GROUP BY x.b/2 ORDER BY x.b/2;

SELECT count(b) FROM test_missing_target x, test_missing_target y
	WHERE x.a = y.a
	GROUP BY x.b/2;

CREATE TABLE test_missing_target3 AS
SELECT count(x.b)
FROM test_missing_target x, test_missing_target y
	WHERE x.a = y.a
	GROUP BY x.b/2 ORDER BY x.b/2;

SELECT * FROM test_missing_target3;

DROP TABLE test_missing_target;

DROP TABLE test_missing_target2;

DROP TABLE test_missing_target3;
