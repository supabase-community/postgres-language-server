CREATE TEMP TABLE combocidtest (foobar int);

BEGIN;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest VALUES (1);

INSERT INTO combocidtest VALUES (2);

SELECT ctid,cmin,* FROM combocidtest;

SAVEPOINT s1;

UPDATE combocidtest SET foobar = foobar + 10;

SELECT ctid,cmin,* FROM combocidtest;

ROLLBACK TO s1;

SELECT ctid,cmin,* FROM combocidtest;

COMMIT;

SELECT ctid,cmin,* FROM combocidtest;

BEGIN;

INSERT INTO combocidtest VALUES (333);

DECLARE c CURSOR FOR SELECT ctid,cmin,* FROM combocidtest;

DELETE FROM combocidtest;

FETCH ALL FROM c;

ROLLBACK;

SELECT ctid,cmin,* FROM combocidtest;

BEGIN;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest SELECT 1 LIMIT 0;

INSERT INTO combocidtest VALUES (444);

SELECT ctid,cmin,* FROM combocidtest;

SAVEPOINT s1;

SELECT ctid,cmin,* FROM combocidtest FOR UPDATE;

SELECT ctid,cmin,* FROM combocidtest;

UPDATE combocidtest SET foobar = foobar + 10;

SELECT ctid,cmin,* FROM combocidtest;

ROLLBACK TO s1;

SELECT ctid,cmin,* FROM combocidtest;

COMMIT;

SELECT ctid,cmin,* FROM combocidtest;

CREATE TABLE IF NOT EXISTS testcase(
	id int PRIMARY KEY,
	balance numeric
);

INSERT INTO testcase VALUES (1, 0);

BEGIN;

SELECT * FROM testcase WHERE testcase.id = 1 FOR UPDATE;

UPDATE testcase SET balance = balance + 400 WHERE id=1;

SAVEPOINT subxact;

UPDATE testcase SET balance = balance - 100 WHERE id=1;

ROLLBACK TO SAVEPOINT subxact;

SELECT * FROM testcase WHERE id = 1 FOR UPDATE;

ROLLBACK;

DROP TABLE testcase;
