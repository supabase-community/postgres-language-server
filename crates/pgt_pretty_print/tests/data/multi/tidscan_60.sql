CREATE TABLE tidscan(id integer);

INSERT INTO tidscan VALUES (1), (2), (3);

SELECT ctid, * FROM tidscan;

SELECT ctid, * FROM tidscan WHERE ctid = '(0,1)';

SELECT ctid, * FROM tidscan WHERE ctid = '(0,1)';

SELECT ctid, * FROM tidscan WHERE '(0,1)' = ctid;

SELECT ctid, * FROM tidscan WHERE '(0,1)' = ctid;

SELECT ctid, * FROM tidscan WHERE ctid = '(0,2)' OR '(0,1)' = ctid;

SELECT ctid, * FROM tidscan WHERE ctid = '(0,2)' OR '(0,1)' = ctid;

SELECT ctid, * FROM tidscan WHERE ctid = ANY(ARRAY['(0,1)', '(0,2)']::tid[]);

SELECT ctid, * FROM tidscan WHERE ctid = ANY(ARRAY['(0,1)', '(0,2)']::tid[]);

SELECT ctid, * FROM tidscan WHERE ctid != ANY(ARRAY['(0,1)', '(0,2)']::tid[]);

SELECT ctid, * FROM tidscan WHERE ctid != ANY(ARRAY['(0,1)', '(0,2)']::tid[]);

SELECT ctid, * FROM tidscan
WHERE (id = 3 AND ctid IN ('(0,2)', '(0,3)')) OR (ctid = '(0,1)' AND id = 1);

SELECT ctid, * FROM tidscan
WHERE (id = 3 AND ctid IN ('(0,2)', '(0,3)')) OR (ctid = '(0,1)' AND id = 1);

SET enable_hashjoin TO off;

SELECT t1.ctid, t1.*, t2.ctid, t2.*
FROM tidscan t1 JOIN tidscan t2 ON t1.ctid = t2.ctid WHERE t1.id = 1;

SELECT t1.ctid, t1.*, t2.ctid, t2.*
FROM tidscan t1 JOIN tidscan t2 ON t1.ctid = t2.ctid WHERE t1.id = 1;

SELECT t1.ctid, t1.*, t2.ctid, t2.*
FROM tidscan t1 LEFT JOIN tidscan t2 ON t1.ctid = t2.ctid WHERE t1.id = 1;

SELECT t1.ctid, t1.*, t2.ctid, t2.*
FROM tidscan t1 LEFT JOIN tidscan t2 ON t1.ctid = t2.ctid WHERE t1.id = 1;

RESET enable_hashjoin;

BEGIN;

DECLARE c CURSOR FOR
SELECT ctid, * FROM tidscan WHERE ctid = ANY(ARRAY['(0,1)', '(0,2)']::tid[]);

FETCH ALL FROM c;

FETCH BACKWARD 1 FROM c;

FETCH FIRST FROM c;

ROLLBACK;

BEGIN;

DECLARE c CURSOR FOR SELECT ctid, * FROM tidscan;

FETCH NEXT FROM c;

FETCH NEXT FROM c;

UPDATE tidscan SET id = -id WHERE CURRENT OF c RETURNING *;

FETCH NEXT FROM c;

UPDATE tidscan SET id = -id WHERE CURRENT OF c RETURNING *;

SELECT * FROM tidscan;

FETCH NEXT FROM c;

UPDATE tidscan SET id = -id WHERE CURRENT OF c RETURNING *;

ROLLBACK;

SELECT count(*) FROM tenk1 t1 JOIN tenk1 t2 ON t1.ctid = t2.ctid;

SELECT count(*) FROM tenk1 t1 JOIN tenk1 t2 ON t1.ctid = t2.ctid;

SET enable_hashjoin TO off;

SELECT count(*) FROM tenk1 t1 JOIN tenk1 t2 ON t1.ctid = t2.ctid;

SELECT count(*) FROM tenk1 t1 JOIN tenk1 t2 ON t1.ctid = t2.ctid;

RESET enable_hashjoin;

BEGIN ISOLATION LEVEL SERIALIZABLE;

SELECT * FROM tidscan WHERE ctid = '(0,1)';

SELECT locktype, mode FROM pg_locks WHERE pid = pg_backend_pid() AND mode = 'SIReadLock';

ROLLBACK;

DROP TABLE tidscan;
