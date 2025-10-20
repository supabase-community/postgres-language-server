SET client_min_messages TO 'warning';

DROP USER IF EXISTS regress_rls_alice;

DROP USER IF EXISTS regress_rls_bob;

DROP USER IF EXISTS regress_rls_carol;

DROP USER IF EXISTS regress_rls_dave;

DROP USER IF EXISTS regress_rls_exempt_user;

DROP ROLE IF EXISTS regress_rls_group1;

DROP ROLE IF EXISTS regress_rls_group2;

DROP SCHEMA IF EXISTS regress_rls_schema CASCADE;

RESET client_min_messages;

CREATE USER regress_rls_alice NOLOGIN;

CREATE USER regress_rls_bob NOLOGIN;

CREATE USER regress_rls_carol NOLOGIN;

CREATE USER regress_rls_dave NOLOGIN;

CREATE USER regress_rls_exempt_user BYPASSRLS NOLOGIN;

CREATE ROLE regress_rls_group1 NOLOGIN;

CREATE ROLE regress_rls_group2 NOLOGIN;

GRANT regress_rls_group1 TO regress_rls_bob;

GRANT regress_rls_group2 TO regress_rls_carol;

CREATE SCHEMA regress_rls_schema;

GRANT ALL ON SCHEMA regress_rls_schema to public;

SET search_path = regress_rls_schema;

CREATE OR REPLACE FUNCTION f_leak(text) RETURNS bool
    COST 0.0000001 LANGUAGE plpgsql
    AS 'BEGIN RAISE NOTICE ''f_leak => %'', $1; RETURN true; END';

GRANT EXECUTE ON FUNCTION f_leak(text) TO public;

SET SESSION AUTHORIZATION regress_rls_alice;

CREATE TABLE uaccount (
    pguser      name primary key,
    seclv       int
);

GRANT SELECT ON uaccount TO public;

INSERT INTO uaccount VALUES
    ('regress_rls_alice', 99),
    ('regress_rls_bob', 1),
    ('regress_rls_carol', 2),
    ('regress_rls_dave', 3);

CREATE TABLE category (
    cid        int primary key,
    cname      text
);

GRANT ALL ON category TO public;

INSERT INTO category VALUES
    (11, 'novel'),
    (22, 'science fiction'),
    (33, 'technology'),
    (44, 'manga');

CREATE TABLE document (
    did         int primary key,
    cid         int references category(cid),
    dlevel      int not null,
    dauthor     name,
    dtitle      text
);

GRANT ALL ON document TO public;

INSERT INTO document VALUES
    ( 1, 11, 1, 'regress_rls_bob', 'my first novel'),
    ( 2, 11, 2, 'regress_rls_bob', 'my second novel'),
    ( 3, 22, 2, 'regress_rls_bob', 'my science fiction'),
    ( 4, 44, 1, 'regress_rls_bob', 'my first manga'),
    ( 5, 44, 2, 'regress_rls_bob', 'my second manga'),
    ( 6, 22, 1, 'regress_rls_carol', 'great science fiction'),
    ( 7, 33, 2, 'regress_rls_carol', 'great technology book'),
    ( 8, 44, 1, 'regress_rls_carol', 'great manga'),
    ( 9, 22, 1, 'regress_rls_dave', 'awesome science fiction'),
    (10, 33, 2, 'regress_rls_dave', 'awesome technology book');

ALTER TABLE document ENABLE ROW LEVEL SECURITY;

CREATE POLICY p1 ON document AS PERMISSIVE
    USING (dlevel <= (SELECT seclv FROM uaccount WHERE pguser = current_user));

CREATE POLICY p2r ON document AS RESTRICTIVE TO regress_rls_dave
    USING (cid <> 44 AND cid < 50);

CREATE POLICY p1r ON document AS RESTRICTIVE TO regress_rls_dave
    USING (cid <> 44);

SELECT * FROM pg_policies WHERE schemaname = 'regress_rls_schema' AND tablename = 'document' ORDER BY policyname;

SET SESSION AUTHORIZATION regress_rls_bob;

SET row_security TO ON;

SELECT * FROM document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM document NATURAL JOIN category WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM document TABLESAMPLE BERNOULLI(50) REPEATABLE(0)
  WHERE f_leak(dtitle) ORDER BY did;

SET SESSION AUTHORIZATION regress_rls_carol;

SELECT * FROM document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM document NATURAL JOIN category WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM document TABLESAMPLE BERNOULLI(50) REPEATABLE(0)
  WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM document WHERE f_leak(dtitle);

SELECT * FROM document NATURAL JOIN category WHERE f_leak(dtitle);

SET SESSION AUTHORIZATION regress_rls_dave;

SELECT * FROM document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM document NATURAL JOIN category WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM document WHERE f_leak(dtitle);

SELECT * FROM document NATURAL JOIN category WHERE f_leak(dtitle);

INSERT INTO document VALUES (100, 44, 1, 'regress_rls_dave', 'testing sorting of policies');

INSERT INTO document VALUES (100, 55, 1, 'regress_rls_dave', 'testing sorting of policies');

ALTER POLICY p1 ON document USING (true);

DROP POLICY p1 ON document;

SET SESSION AUTHORIZATION regress_rls_alice;

ALTER POLICY p1 ON document USING (dauthor = current_user);

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM document NATURAL JOIN category WHERE f_leak(dtitle) ORDER by did;

SET SESSION AUTHORIZATION regress_rls_carol;

SELECT * FROM document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM document NATURAL JOIN category WHERE f_leak(dtitle) ORDER by did;

SELECT * FROM document WHERE f_leak(dtitle);

SELECT * FROM document NATURAL JOIN category WHERE f_leak(dtitle);

SET SESSION AUTHORIZATION regress_rls_alice;

CREATE POLICY p2 ON category
    USING (CASE WHEN current_user = 'regress_rls_bob' THEN cid IN (11, 33)
           WHEN current_user = 'regress_rls_carol' THEN cid IN (22, 44)
           ELSE false END);

ALTER TABLE category ENABLE ROW LEVEL SECURITY;

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM document d FULL OUTER JOIN category c on d.cid = c.cid ORDER BY d.did, c.cid;

DELETE FROM category WHERE cid = 33;

SET SESSION AUTHORIZATION regress_rls_carol;

SELECT * FROM document d FULL OUTER JOIN category c on d.cid = c.cid ORDER BY d.did, c.cid;

INSERT INTO document VALUES (11, 33, 1, current_user, 'hoge');

SET SESSION AUTHORIZATION regress_rls_bob;

INSERT INTO document VALUES (8, 44, 1, 'regress_rls_bob', 'my third manga');

SELECT * FROM document WHERE did = 8;

INSERT INTO document VALUES (8, 44, 1, 'regress_rls_carol', 'my third manga');

UPDATE document SET did = 8, dauthor = 'regress_rls_carol' WHERE did = 5;

RESET SESSION AUTHORIZATION;

SET row_security TO ON;

SELECT * FROM document;

SELECT * FROM category;

RESET SESSION AUTHORIZATION;

SET row_security TO OFF;

SELECT * FROM document;

SELECT * FROM category;

SET SESSION AUTHORIZATION regress_rls_exempt_user;

SET row_security TO OFF;

SELECT * FROM document;

SELECT * FROM category;

SET SESSION AUTHORIZATION regress_rls_alice;

SET row_security TO ON;

SELECT * FROM document;

SELECT * FROM category;

SET SESSION AUTHORIZATION regress_rls_alice;

SET row_security TO OFF;

SELECT * FROM document;

SELECT * FROM category;

SET SESSION AUTHORIZATION regress_rls_alice;

SET row_security TO ON;

CREATE TABLE t1 (id int not null primary key, a int, junk1 text, b text);

ALTER TABLE t1 DROP COLUMN junk1;

GRANT ALL ON t1 TO public;

CREATE TABLE t2 (c float) INHERITS (t1);

GRANT ALL ON t2 TO public;

CREATE TABLE t3 (id int not null primary key, c text, b text, a int);

ALTER TABLE t3 INHERIT t1;

GRANT ALL ON t3 TO public;

CREATE POLICY p1 ON t1 FOR ALL TO PUBLIC USING (a % 2 = 0);

CREATE POLICY p2 ON t2 FOR ALL TO PUBLIC USING (a % 2 = 1);

ALTER TABLE t1 ENABLE ROW LEVEL SECURITY;

ALTER TABLE t2 ENABLE ROW LEVEL SECURITY;

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM t1;

SELECT * FROM t1;

SELECT * FROM t1 WHERE f_leak(b);

SELECT * FROM t1 WHERE f_leak(b);

SELECT tableoid::regclass, * FROM t1;

SELECT *, t1 FROM t1;

SELECT *, t1 FROM t1;

SELECT *, t1 FROM t1;

SELECT * FROM t1 FOR SHARE;

SELECT * FROM t1 FOR SHARE;

SELECT * FROM t1 WHERE f_leak(b) FOR SHARE;

SELECT * FROM t1 WHERE f_leak(b) FOR SHARE;

SELECT a, b, tableoid::regclass FROM t2 UNION ALL SELECT a, b, tableoid::regclass FROM t3;

SELECT a, b, tableoid::regclass FROM t2 UNION ALL SELECT a, b, tableoid::regclass FROM t3;

RESET SESSION AUTHORIZATION;

SET row_security TO OFF;

SELECT * FROM t1 WHERE f_leak(b);

SELECT * FROM t1 WHERE f_leak(b);

SET SESSION AUTHORIZATION regress_rls_exempt_user;

SET row_security TO OFF;

SELECT * FROM t1 WHERE f_leak(b);

SELECT * FROM t1 WHERE f_leak(b);

SET SESSION AUTHORIZATION regress_rls_alice;

CREATE TABLE part_document (
    did         int,
    cid         int,
    dlevel      int not null,
    dauthor     name,
    dtitle      text
) PARTITION BY RANGE (cid);

GRANT ALL ON part_document TO public;

CREATE TABLE part_document_fiction PARTITION OF part_document FOR VALUES FROM (11) to (12);

CREATE TABLE part_document_satire PARTITION OF part_document FOR VALUES FROM (55) to (56);

CREATE TABLE part_document_nonfiction PARTITION OF part_document FOR VALUES FROM (99) to (100);

GRANT ALL ON part_document_fiction TO public;

GRANT ALL ON part_document_satire TO public;

GRANT ALL ON part_document_nonfiction TO public;

INSERT INTO part_document VALUES
    ( 1, 11, 1, 'regress_rls_bob', 'my first novel'),
    ( 2, 11, 2, 'regress_rls_bob', 'my second novel'),
    ( 3, 99, 2, 'regress_rls_bob', 'my science textbook'),
    ( 4, 55, 1, 'regress_rls_bob', 'my first satire'),
    ( 5, 99, 2, 'regress_rls_bob', 'my history book'),
    ( 6, 11, 1, 'regress_rls_carol', 'great science fiction'),
    ( 7, 99, 2, 'regress_rls_carol', 'great technology book'),
    ( 8, 55, 2, 'regress_rls_carol', 'great satire'),
    ( 9, 11, 1, 'regress_rls_dave', 'awesome science fiction'),
    (10, 99, 2, 'regress_rls_dave', 'awesome technology book');

ALTER TABLE part_document ENABLE ROW LEVEL SECURITY;

CREATE POLICY pp1 ON part_document AS PERMISSIVE
    USING (dlevel <= (SELECT seclv FROM uaccount WHERE pguser = current_user));

CREATE POLICY pp1r ON part_document AS RESTRICTIVE TO regress_rls_dave
    USING (cid < 55);

SELECT * FROM pg_policies WHERE schemaname = 'regress_rls_schema' AND tablename like '%part_document%' ORDER BY policyname;

SET SESSION AUTHORIZATION regress_rls_bob;

SET row_security TO ON;

SELECT * FROM part_document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM part_document WHERE f_leak(dtitle);

SET SESSION AUTHORIZATION regress_rls_carol;

SELECT * FROM part_document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM part_document WHERE f_leak(dtitle);

SET SESSION AUTHORIZATION regress_rls_dave;

SELECT * FROM part_document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM part_document WHERE f_leak(dtitle);

INSERT INTO part_document VALUES (100, 11, 5, 'regress_rls_dave', 'testing pp1');

INSERT INTO part_document VALUES (100, 99, 1, 'regress_rls_dave', 'testing pp1r');

INSERT INTO part_document VALUES (100, 55, 1, 'regress_rls_dave', 'testing RLS with partitions');

INSERT INTO part_document_satire VALUES (100, 55, 1, 'regress_rls_dave', 'testing RLS with partitions');

SELECT * FROM part_document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM part_document_satire WHERE f_leak(dtitle) ORDER BY did;

SET SESSION AUTHORIZATION regress_rls_alice;

ALTER TABLE part_document_satire ENABLE ROW LEVEL SECURITY;

CREATE POLICY pp3 ON part_document_satire AS RESTRICTIVE
    USING (cid < 55);

SET SESSION AUTHORIZATION regress_rls_dave;

INSERT INTO part_document_satire VALUES (101, 55, 1, 'regress_rls_dave', 'testing RLS with partitions');

SELECT * FROM part_document_satire WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM part_document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM part_document WHERE f_leak(dtitle);

SET SESSION AUTHORIZATION regress_rls_carol;

SELECT * FROM part_document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM part_document WHERE f_leak(dtitle);

ALTER POLICY pp1 ON part_document USING (true);

DROP POLICY pp1 ON part_document;

SET SESSION AUTHORIZATION regress_rls_alice;

ALTER POLICY pp1 ON part_document USING (dauthor = current_user);

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM part_document WHERE f_leak(dtitle) ORDER BY did;

SET SESSION AUTHORIZATION regress_rls_carol;

SELECT * FROM part_document WHERE f_leak(dtitle) ORDER BY did;

SELECT * FROM part_document WHERE f_leak(dtitle);

RESET SESSION AUTHORIZATION;

SET row_security TO ON;

SELECT * FROM part_document ORDER BY did;

SELECT * FROM part_document_satire ORDER by did;

SET SESSION AUTHORIZATION regress_rls_exempt_user;

SET row_security TO OFF;

SELECT * FROM part_document ORDER BY did;

SELECT * FROM part_document_satire ORDER by did;

SET SESSION AUTHORIZATION regress_rls_alice;

SET row_security TO ON;

SELECT * FROM part_document ORDER by did;

SELECT * FROM part_document_satire ORDER by did;

SET SESSION AUTHORIZATION regress_rls_dave;

SET row_security TO OFF;

SELECT * FROM part_document ORDER by did;

SELECT * FROM part_document_satire ORDER by did;

SET SESSION AUTHORIZATION regress_rls_alice;

SET row_security TO ON;

CREATE POLICY pp3 ON part_document AS RESTRICTIVE
    USING ((SELECT dlevel <= seclv FROM uaccount WHERE pguser = current_user));

SET SESSION AUTHORIZATION regress_rls_carol;

INSERT INTO part_document VALUES (100, 11, 5, 'regress_rls_carol', 'testing pp3');

SET SESSION AUTHORIZATION regress_rls_alice;

SET row_security TO ON;

CREATE TABLE dependee (x integer, y integer);

CREATE TABLE dependent (x integer, y integer);

CREATE POLICY d1 ON dependent FOR ALL
    TO PUBLIC
    USING (x = (SELECT d.x FROM dependee d WHERE d.y = y));

DROP TABLE dependee;

DROP TABLE dependee CASCADE;

SELECT * FROM dependent;

SET SESSION AUTHORIZATION regress_rls_alice;

CREATE TABLE rec1 (x integer, y integer);

CREATE POLICY r1 ON rec1 USING (x = (SELECT r.x FROM rec1 r WHERE y = r.y));

ALTER TABLE rec1 ENABLE ROW LEVEL SECURITY;

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM rec1;

SET SESSION AUTHORIZATION regress_rls_alice;

CREATE TABLE rec2 (a integer, b integer);

ALTER POLICY r1 ON rec1 USING (x = (SELECT a FROM rec2 WHERE b = y));

CREATE POLICY r2 ON rec2 USING (a = (SELECT x FROM rec1 WHERE y = b));

ALTER TABLE rec2 ENABLE ROW LEVEL SECURITY;

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM rec1;

SET SESSION AUTHORIZATION regress_rls_bob;

CREATE VIEW rec1v AS SELECT * FROM rec1;

CREATE VIEW rec2v AS SELECT * FROM rec2;

SET SESSION AUTHORIZATION regress_rls_alice;

ALTER POLICY r1 ON rec1 USING (x = (SELECT a FROM rec2v WHERE b = y));

ALTER POLICY r2 ON rec2 USING (a = (SELECT x FROM rec1v WHERE y = b));

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM rec1;

SET SESSION AUTHORIZATION regress_rls_bob;

DROP VIEW rec1v, rec2v CASCADE;

CREATE VIEW rec1v WITH (security_barrier) AS SELECT * FROM rec1;

CREATE VIEW rec2v WITH (security_barrier) AS SELECT * FROM rec2;

SET SESSION AUTHORIZATION regress_rls_alice;

CREATE POLICY r1 ON rec1 USING (x = (SELECT a FROM rec2v WHERE b = y));

CREATE POLICY r2 ON rec2 USING (a = (SELECT x FROM rec1v WHERE y = b));

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM rec1;

SET SESSION AUTHORIZATION regress_rls_alice;

CREATE TABLE s1 (a int, b text);

INSERT INTO s1 (SELECT x, public.fipshash(x::text) FROM generate_series(-10,10) x);

CREATE TABLE s2 (x int, y text);

INSERT INTO s2 (SELECT x, public.fipshash(x::text) FROM generate_series(-6,6) x);

GRANT SELECT ON s1, s2 TO regress_rls_bob;

CREATE POLICY p1 ON s1 USING (a in (select x from s2 where y like '%2f%'));

CREATE POLICY p2 ON s2 USING (x in (select a from s1 where b like '%22%'));

CREATE POLICY p3 ON s1 FOR INSERT WITH CHECK (a = (SELECT a FROM s1));

ALTER TABLE s1 ENABLE ROW LEVEL SECURITY;

ALTER TABLE s2 ENABLE ROW LEVEL SECURITY;

SET SESSION AUTHORIZATION regress_rls_bob;

CREATE VIEW v2 AS SELECT * FROM s2 WHERE y like '%af%';

SELECT * FROM s1 WHERE f_leak(b);

INSERT INTO s1 VALUES (1, 'foo');

SET SESSION AUTHORIZATION regress_rls_alice;

DROP POLICY p3 on s1;

ALTER POLICY p2 ON s2 USING (x % 2 = 0);

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM s1 WHERE f_leak(b);

SELECT * FROM only s1 WHERE f_leak(b);

SET SESSION AUTHORIZATION regress_rls_alice;

ALTER POLICY p1 ON s1 USING (a in (select x from v2));

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM s1 WHERE f_leak(b);

SELECT * FROM s1 WHERE f_leak(b);

SELECT (SELECT x FROM s1 LIMIT 1) xx, * FROM s2 WHERE y like '%28%';

SELECT (SELECT x FROM s1 LIMIT 1) xx, * FROM s2 WHERE y like '%28%';

SET SESSION AUTHORIZATION regress_rls_alice;

ALTER POLICY p2 ON s2 USING (x in (select a from s1 where b like '%d2%'));

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM s1 WHERE f_leak(b);

PREPARE p1(int) AS SELECT * FROM t1 WHERE a <= $1;

EXECUTE p1(2);

EXPLAIN (COSTS OFF) EXECUTE p1(2);

RESET SESSION AUTHORIZATION;

SET row_security TO OFF;

SELECT * FROM t1 WHERE f_leak(b);

SELECT * FROM t1 WHERE f_leak(b);

EXECUTE p1(2);

EXPLAIN (COSTS OFF) EXECUTE p1(2);

PREPARE p2(int) AS SELECT * FROM t1 WHERE a = $1;

EXECUTE p2(2);

EXPLAIN (COSTS OFF) EXECUTE p2(2);

SET SESSION AUTHORIZATION regress_rls_bob;

SET row_security TO ON;

EXECUTE p2(2);

EXPLAIN (COSTS OFF) EXECUTE p2(2);

SET SESSION AUTHORIZATION regress_rls_bob;

UPDATE t1 SET b = b || b WHERE f_leak(b);

UPDATE t1 SET b = b || b WHERE f_leak(b);

UPDATE only t1 SET b = b || '_updt' WHERE f_leak(b);

UPDATE only t1 SET b = b || '_updt' WHERE f_leak(b);

UPDATE only t1 SET b = b WHERE f_leak(b) RETURNING tableoid::regclass, *, t1;

UPDATE t1 SET b = b WHERE f_leak(b) RETURNING *;

UPDATE t1 SET b = b WHERE f_leak(b) RETURNING tableoid::regclass, *, t1;

UPDATE t2 SET b=t2.b FROM t3
WHERE t2.a = 3 and t3.a = 2 AND f_leak(t2.b) AND f_leak(t3.b);

UPDATE t2 SET b=t2.b FROM t3
WHERE t2.a = 3 and t3.a = 2 AND f_leak(t2.b) AND f_leak(t3.b);

UPDATE t1 SET b=t1.b FROM t2
WHERE t1.a = 3 and t2.a = 3 AND f_leak(t1.b) AND f_leak(t2.b);

UPDATE t1 SET b=t1.b FROM t2
WHERE t1.a = 3 and t2.a = 3 AND f_leak(t1.b) AND f_leak(t2.b);

UPDATE t2 SET b=t2.b FROM t1
WHERE t1.a = 3 and t2.a = 3 AND f_leak(t1.b) AND f_leak(t2.b);

UPDATE t2 SET b=t2.b FROM t1
WHERE t1.a = 3 and t2.a = 3 AND f_leak(t1.b) AND f_leak(t2.b);

UPDATE t2 t2_1 SET b = t2_2.b FROM t2 t2_2
WHERE t2_1.a = 3 AND t2_2.a = t2_1.a AND t2_2.b = t2_1.b
AND f_leak(t2_1.b) AND f_leak(t2_2.b) RETURNING *, t2_1, t2_2;

UPDATE t2 t2_1 SET b = t2_2.b FROM t2 t2_2
WHERE t2_1.a = 3 AND t2_2.a = t2_1.a AND t2_2.b = t2_1.b
AND f_leak(t2_1.b) AND f_leak(t2_2.b) RETURNING *, t2_1, t2_2;

UPDATE t1 t1_1 SET b = t1_2.b FROM t1 t1_2
WHERE t1_1.a = 4 AND t1_2.a = t1_1.a AND t1_2.b = t1_1.b
AND f_leak(t1_1.b) AND f_leak(t1_2.b) RETURNING *, t1_1, t1_2;

UPDATE t1 t1_1 SET b = t1_2.b FROM t1 t1_2
WHERE t1_1.a = 4 AND t1_2.a = t1_1.a AND t1_2.b = t1_1.b
AND f_leak(t1_1.b) AND f_leak(t1_2.b) RETURNING *, t1_1, t1_2;

RESET SESSION AUTHORIZATION;

SET row_security TO OFF;

SELECT * FROM t1 ORDER BY a,b;

SET SESSION AUTHORIZATION regress_rls_bob;

SET row_security TO ON;

DELETE FROM only t1 WHERE f_leak(b);

DELETE FROM t1 WHERE f_leak(b);

DELETE FROM only t1 WHERE f_leak(b) RETURNING tableoid::regclass, *, t1;

DELETE FROM t1 WHERE f_leak(b) RETURNING tableoid::regclass, *, t1;

SET SESSION AUTHORIZATION regress_rls_alice;

CREATE TABLE b1 (a int, b text);

INSERT INTO b1 (SELECT x, public.fipshash(x::text) FROM generate_series(-10,10) x);

CREATE POLICY p1 ON b1 USING (a % 2 = 0);

ALTER TABLE b1 ENABLE ROW LEVEL SECURITY;

GRANT ALL ON b1 TO regress_rls_bob;

SET SESSION AUTHORIZATION regress_rls_bob;

CREATE VIEW bv1 WITH (security_barrier) AS SELECT * FROM b1 WHERE a > 0 WITH CHECK OPTION;

GRANT ALL ON bv1 TO regress_rls_carol;

SET SESSION AUTHORIZATION regress_rls_carol;

SELECT * FROM bv1 WHERE f_leak(b);

SELECT * FROM bv1 WHERE f_leak(b);

INSERT INTO bv1 VALUES (-1, 'xxx');

INSERT INTO bv1 VALUES (11, 'xxx');

INSERT INTO bv1 VALUES (12, 'xxx');

UPDATE bv1 SET b = 'yyy' WHERE a = 4 AND f_leak(b);

UPDATE bv1 SET b = 'yyy' WHERE a = 4 AND f_leak(b);

DELETE FROM bv1 WHERE a = 6 AND f_leak(b);

DELETE FROM bv1 WHERE a = 6 AND f_leak(b);

SET SESSION AUTHORIZATION regress_rls_alice;

SELECT * FROM b1;

SET SESSION AUTHORIZATION regress_rls_alice;

DROP POLICY p1 ON document;

DROP POLICY p1r ON document;

CREATE POLICY p1 ON document FOR SELECT USING (true);

CREATE POLICY p2 ON document FOR INSERT WITH CHECK (dauthor = current_user);

CREATE POLICY p3 ON document FOR UPDATE
  USING (cid = (SELECT cid from category WHERE cname = 'novel'))
  WITH CHECK (dauthor = current_user);

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM document WHERE did = 2;

INSERT INTO document VALUES (2, (SELECT cid from category WHERE cname = 'novel'), 1, 'regress_rls_carol', 'my first novel')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle, dauthor = EXCLUDED.dauthor;

INSERT INTO document VALUES (33, 22, 1, 'regress_rls_bob', 'okay science fiction');

INSERT INTO document VALUES (33, (SELECT cid from category WHERE cname = 'novel'), 1, 'regress_rls_bob', 'Some novel, replaces sci-fi') -- takes UPDATE path
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle;

INSERT INTO document VALUES (2, (SELECT cid from category WHERE cname = 'novel'), 1, 'regress_rls_bob', 'my first novel')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle RETURNING *;

INSERT INTO document VALUES (78, (SELECT cid from category WHERE cname = 'novel'), 1, 'regress_rls_bob', 'some technology novel')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle, cid = 33 RETURNING *;

INSERT INTO document VALUES (78, (SELECT cid from category WHERE cname = 'novel'), 1, 'regress_rls_bob', 'some technology novel')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle, cid = 33 RETURNING *;

INSERT INTO document VALUES (78, (SELECT cid from category WHERE cname = 'novel'), 1, 'regress_rls_bob', 'some technology novel')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle, cid = 33 RETURNING *;

INSERT INTO document VALUES (79, (SELECT cid from category WHERE cname = 'technology'), 1, 'regress_rls_bob', 'technology book, can only insert')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle RETURNING *;

INSERT INTO document VALUES (79, (SELECT cid from category WHERE cname = 'technology'), 1, 'regress_rls_bob', 'technology book, can only insert')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle RETURNING *;

SET SESSION AUTHORIZATION regress_rls_alice;

DROP POLICY p1 ON document;

DROP POLICY p2 ON document;

DROP POLICY p3 ON document;

CREATE POLICY p3_with_default ON document FOR UPDATE
  USING (cid = (SELECT cid from category WHERE cname = 'novel'));

SET SESSION AUTHORIZATION regress_rls_bob;

INSERT INTO document VALUES (79, (SELECT cid from category WHERE cname = 'technology'), 1, 'regress_rls_bob', 'technology book, can only insert')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle RETURNING *;

INSERT INTO document VALUES (2, (SELECT cid from category WHERE cname = 'technology'), 1, 'regress_rls_bob', 'my first novel')
    ON CONFLICT (did) DO UPDATE SET cid = EXCLUDED.cid, dtitle = EXCLUDED.dtitle RETURNING *;

SET SESSION AUTHORIZATION regress_rls_alice;

DROP POLICY p3_with_default ON document;

CREATE POLICY p3_with_all ON document FOR ALL
  USING (cid = (SELECT cid from category WHERE cname = 'novel'))
  WITH CHECK (dauthor = current_user);

SET SESSION AUTHORIZATION regress_rls_bob;

INSERT INTO document VALUES (80, (SELECT cid from category WHERE cname = 'novel'), 1, 'regress_rls_carol', 'my first novel')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle, cid = 33;

INSERT INTO document VALUES (4, (SELECT cid from category WHERE cname = 'novel'), 1, 'regress_rls_bob', 'my first novel')
    ON CONFLICT (did) DO UPDATE SET dtitle = EXCLUDED.dtitle;

INSERT INTO document VALUES (1, (SELECT cid from category WHERE cname = 'novel'), 1, 'regress_rls_bob', 'my first novel')
    ON CONFLICT (did) DO UPDATE SET dauthor = 'regress_rls_carol';

RESET SESSION AUTHORIZATION;

DROP POLICY p3_with_all ON document;

ALTER TABLE document ADD COLUMN dnotes text DEFAULT '';

CREATE POLICY p1 ON document FOR SELECT USING (true);

CREATE POLICY p2 ON document FOR INSERT WITH CHECK (dauthor = current_user);

CREATE POLICY p3 ON document FOR UPDATE
  USING (cid = (SELECT cid from category WHERE cname = 'novel'))
  WITH CHECK (dlevel > 0);

CREATE POLICY p4 ON document FOR DELETE
  USING (cid = (SELECT cid from category WHERE cname = 'manga'));

SELECT * FROM document;

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM document WHERE did = 4;

RESET SESSION AUTHORIZATION;

SET SESSION AUTHORIZATION regress_rls_carol;

RESET SESSION AUTHORIZATION;

SET SESSION AUTHORIZATION regress_rls_bob;

RESET SESSION AUTHORIZATION;

DROP POLICY p1 ON document;

CREATE POLICY p1 ON document FOR SELECT
  USING (cid = (SELECT cid from category WHERE cname = 'novel'));

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM document WHERE did = 13;

RESET SESSION AUTHORIZATION;

DROP POLICY p1 ON document;

SELECT * FROM document;

SET SESSION AUTHORIZATION regress_rls_alice;

CREATE TABLE z1 (a int, b text);

CREATE TABLE z2 (a int, b text);

GRANT SELECT ON z1,z2 TO regress_rls_group1, regress_rls_group2,
    regress_rls_bob, regress_rls_carol;

INSERT INTO z1 VALUES
    (1, 'aba'),
    (2, 'bbb'),
    (3, 'ccc'),
    (4, 'dad');

CREATE POLICY p1 ON z1 TO regress_rls_group1 USING (a % 2 = 0);

CREATE POLICY p2 ON z1 TO regress_rls_group2 USING (a % 2 = 1);

ALTER TABLE z1 ENABLE ROW LEVEL SECURITY;

SET SESSION AUTHORIZATION regress_rls_bob;

SELECT * FROM z1 WHERE f_leak(b);

SELECT * FROM z1 WHERE f_leak(b);

PREPARE plancache_test AS SELECT * FROM z1 WHERE f_leak(b);

EXPLAIN (COSTS OFF) EXECUTE plancache_test;
