SET enable_seqscan TO off;

CREATE TABLE tidrangescan(id integer, data text);

SELECT ctid FROM tidrangescan WHERE ctid < '(1, 0)';

SELECT ctid FROM tidrangescan WHERE ctid < '(1, 0)';

SELECT ctid FROM tidrangescan WHERE ctid > '(9, 0)';

SELECT ctid FROM tidrangescan WHERE ctid > '(9, 0)';

INSERT INTO tidrangescan SELECT i,repeat('x', 100) FROM generate_series(1,200) AS s(i);

DELETE FROM tidrangescan
WHERE substring(ctid::text FROM ',(\d+)\)')::integer > 10 OR substring(ctid::text FROM '\((\d+),')::integer > 2;

VACUUM tidrangescan;

SELECT ctid FROM tidrangescan WHERE ctid < '(1,0)';

SELECT ctid FROM tidrangescan WHERE ctid < '(1,0)';

SELECT ctid FROM tidrangescan WHERE ctid <= '(1,5)';

SELECT ctid FROM tidrangescan WHERE ctid <= '(1,5)';

SELECT ctid FROM tidrangescan WHERE ctid < '(0,0)';

SELECT ctid FROM tidrangescan WHERE ctid < '(0,0)';

SELECT ctid FROM tidrangescan WHERE ctid > '(2,8)';

SELECT ctid FROM tidrangescan WHERE ctid > '(2,8)';

SELECT ctid FROM tidrangescan WHERE '(2,8)' < ctid;

SELECT ctid FROM tidrangescan WHERE '(2,8)' < ctid;

SELECT ctid FROM tidrangescan WHERE ctid >= '(2,8)';

SELECT ctid FROM tidrangescan WHERE ctid >= '(2,8)';

SELECT ctid FROM tidrangescan WHERE ctid >= '(100,0)';

SELECT ctid FROM tidrangescan WHERE ctid >= '(100,0)';

SELECT ctid FROM tidrangescan WHERE ctid > '(1,4)' AND '(1,7)' >= ctid;

SELECT ctid FROM tidrangescan WHERE ctid > '(1,4)' AND '(1,7)' >= ctid;

SELECT ctid FROM tidrangescan WHERE '(1,7)' >= ctid AND ctid > '(1,4)';

SELECT ctid FROM tidrangescan WHERE '(1,7)' >= ctid AND ctid > '(1,4)';

SELECT ctid FROM tidrangescan WHERE ctid > '(0,65535)' AND ctid < '(1,0)' LIMIT 1;

SELECT ctid FROM tidrangescan WHERE ctid < '(0,0)' LIMIT 1;

SELECT ctid FROM tidrangescan WHERE ctid > '(4294967295,65535)';

SELECT ctid FROM tidrangescan WHERE ctid < '(0,0)';

SELECT ctid FROM tidrangescan WHERE ctid >= (SELECT NULL::tid);

SELECT t.ctid,t2.c FROM tidrangescan t,
LATERAL (SELECT count(*) c FROM tidrangescan t2 WHERE t2.ctid <= t.ctid) t2
WHERE t.ctid < '(1,0)';

SELECT t.ctid,t2.c FROM tidrangescan t,
LATERAL (SELECT count(*) c FROM tidrangescan t2 WHERE t2.ctid <= t.ctid) t2
WHERE t.ctid < '(1,0)';

EXPLAIN (COSTS OFF)
DECLARE c SCROLL CURSOR FOR SELECT ctid FROM tidrangescan WHERE ctid < '(1,0)';

BEGIN;

DECLARE c SCROLL CURSOR FOR SELECT ctid FROM tidrangescan WHERE ctid < '(1,0)';

FETCH NEXT c;

FETCH NEXT c;

FETCH PRIOR c;

FETCH FIRST c;

FETCH LAST c;

COMMIT;

DROP TABLE tidrangescan;

RESET enable_seqscan;
