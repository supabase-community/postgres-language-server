SET client_min_messages TO 'warning';

DROP ROLE IF EXISTS regress_priv_group1;

DROP ROLE IF EXISTS regress_priv_group2;

DROP ROLE IF EXISTS regress_priv_user1;

DROP ROLE IF EXISTS regress_priv_user2;

DROP ROLE IF EXISTS regress_priv_user3;

DROP ROLE IF EXISTS regress_priv_user4;

DROP ROLE IF EXISTS regress_priv_user5;

DROP ROLE IF EXISTS regress_priv_user6;

DROP ROLE IF EXISTS regress_priv_user7;

SELECT lo_unlink(oid) FROM pg_largeobject_metadata WHERE oid >= 1000 AND oid < 3000 ORDER BY oid;

RESET client_min_messages;

CREATE USER regress_priv_user1;

CREATE USER regress_priv_user2;

CREATE USER regress_priv_user3;

CREATE USER regress_priv_user4;

CREATE USER regress_priv_user5;

CREATE USER regress_priv_user5;

CREATE USER regress_priv_user6;

CREATE USER regress_priv_user7;

CREATE USER regress_priv_user8;

CREATE USER regress_priv_user9;

CREATE USER regress_priv_user10;

CREATE ROLE regress_priv_role;

GRANT regress_priv_user1 TO regress_priv_user2 WITH ADMIN OPTION;

GRANT regress_priv_user1 TO regress_priv_user3 WITH ADMIN OPTION GRANTED BY regress_priv_user2;

GRANT regress_priv_user1 TO regress_priv_user2 WITH ADMIN OPTION GRANTED BY regress_priv_user3;

REVOKE ADMIN OPTION FOR regress_priv_user1 FROM regress_priv_user2;

REVOKE regress_priv_user1 FROM regress_priv_user2;

SELECT member::regrole, admin_option FROM pg_auth_members WHERE roleid = 'regress_priv_user1'::regrole;

BEGIN;

REVOKE ADMIN OPTION FOR regress_priv_user1 FROM regress_priv_user2 CASCADE;

SELECT member::regrole, admin_option FROM pg_auth_members WHERE roleid = 'regress_priv_user1'::regrole;

ROLLBACK;

REVOKE regress_priv_user1 FROM regress_priv_user2 CASCADE;

SELECT member::regrole, admin_option FROM pg_auth_members WHERE roleid = 'regress_priv_user1'::regrole;

GRANT regress_priv_user1 TO regress_priv_user2 WITH ADMIN OPTION;

GRANT regress_priv_user2 TO regress_priv_user3;

SET ROLE regress_priv_user3;

GRANT regress_priv_user1 TO regress_priv_user4;

SELECT grantor::regrole FROM pg_auth_members WHERE roleid = 'regress_priv_user1'::regrole and member = 'regress_priv_user4'::regrole;

RESET ROLE;

REVOKE regress_priv_user2 FROM regress_priv_user3;

REVOKE regress_priv_user1 FROM regress_priv_user2 CASCADE;

GRANT regress_priv_user1 TO regress_priv_user2 WITH ADMIN OPTION;

GRANT regress_priv_user1 TO regress_priv_user3 GRANTED BY regress_priv_user2;

DROP ROLE regress_priv_user2;

REASSIGN OWNED BY regress_priv_user2 TO regress_priv_user4;

DROP ROLE regress_priv_user2;

DROP OWNED BY regress_priv_user2;

DROP ROLE regress_priv_user2;

GRANT regress_priv_user1 TO regress_priv_user3 WITH ADMIN OPTION;

GRANT regress_priv_user1 TO regress_priv_user4 GRANTED BY regress_priv_user3;

DROP ROLE regress_priv_user3;

DROP ROLE regress_priv_user4;

DROP ROLE regress_priv_user3;

GRANT regress_priv_user1 TO regress_priv_user5 WITH ADMIN OPTION;

GRANT regress_priv_user1 TO regress_priv_user6 GRANTED BY regress_priv_user5;

DROP ROLE regress_priv_user5;

DROP ROLE regress_priv_user1, regress_priv_user5;

CREATE USER regress_priv_user1;

CREATE USER regress_priv_user2;

CREATE USER regress_priv_user3;

CREATE USER regress_priv_user4;

CREATE USER regress_priv_user5;

GRANT pg_read_all_data TO regress_priv_user6;

GRANT pg_write_all_data TO regress_priv_user7;

GRANT pg_read_all_settings TO regress_priv_user8 WITH ADMIN OPTION;

GRANT regress_priv_user9 TO regress_priv_user8;

SET SESSION AUTHORIZATION regress_priv_user8;

GRANT pg_read_all_settings TO regress_priv_user9 WITH ADMIN OPTION;

SET SESSION AUTHORIZATION regress_priv_user9;

GRANT pg_read_all_settings TO regress_priv_user10;

SET SESSION AUTHORIZATION regress_priv_user8;

REVOKE pg_read_all_settings FROM regress_priv_user10 GRANTED BY regress_priv_user9;

REVOKE ADMIN OPTION FOR pg_read_all_settings FROM regress_priv_user9;

REVOKE pg_read_all_settings FROM regress_priv_user9;

RESET SESSION AUTHORIZATION;

REVOKE regress_priv_user9 FROM regress_priv_user8;

REVOKE ADMIN OPTION FOR pg_read_all_settings FROM regress_priv_user8;

SET SESSION AUTHORIZATION regress_priv_user8;

SET ROLE pg_read_all_settings;

RESET ROLE;

RESET SESSION AUTHORIZATION;

REVOKE SET OPTION FOR pg_read_all_settings FROM regress_priv_user8;

GRANT pg_read_all_stats TO regress_priv_user8 WITH SET FALSE;

SET SESSION AUTHORIZATION regress_priv_user8;

SET ROLE pg_read_all_settings;

SET ROLE pg_read_all_stats;

RESET ROLE;

RESET SESSION AUTHORIZATION;

GRANT regress_priv_user9 TO regress_priv_user8;

SET SESSION AUTHORIZATION regress_priv_user8;

SET ROLE regress_priv_user9;

SET debug_parallel_query = 0;

SELECT session_user, current_role, current_user, current_setting('role') as role;

SET debug_parallel_query = 1;

SELECT session_user, current_role, current_user, current_setting('role') as role;

BEGIN;

SET SESSION AUTHORIZATION regress_priv_user10;

SET debug_parallel_query = 0;

SELECT session_user, current_role, current_user, current_setting('role') as role;

SET debug_parallel_query = 1;

SELECT session_user, current_role, current_user, current_setting('role') as role;

ROLLBACK;

SET debug_parallel_query = 0;

SELECT session_user, current_role, current_user, current_setting('role') as role;

SET debug_parallel_query = 1;

SELECT session_user, current_role, current_user, current_setting('role') as role;

RESET SESSION AUTHORIZATION;

SET debug_parallel_query = 0;

SELECT session_user = current_role as c_r_ok, session_user = current_user as c_u_ok, current_setting('role') as role;

SET debug_parallel_query = 1;

SELECT session_user = current_role as c_r_ok, session_user = current_user as c_u_ok, current_setting('role') as role;

RESET debug_parallel_query;

REVOKE pg_read_all_settings FROM regress_priv_user8;

DROP USER regress_priv_user10;

DROP USER regress_priv_user9;

DROP USER regress_priv_user8;

CREATE GROUP regress_priv_group1;

CREATE GROUP regress_priv_group2 WITH ADMIN regress_priv_user1 USER regress_priv_user2;

ALTER GROUP regress_priv_group1 ADD USER regress_priv_user4;

GRANT regress_priv_group2 TO regress_priv_user2 GRANTED BY regress_priv_user1;

SET SESSION AUTHORIZATION regress_priv_user3;

ALTER GROUP regress_priv_group2 ADD USER regress_priv_user2;

ALTER GROUP regress_priv_group2 DROP USER regress_priv_user2;

SET SESSION AUTHORIZATION regress_priv_user1;

ALTER GROUP regress_priv_group2 ADD USER regress_priv_user2;

ALTER GROUP regress_priv_group2 ADD USER regress_priv_user2;

ALTER GROUP regress_priv_group2 DROP USER regress_priv_user2;

ALTER USER regress_priv_user2 PASSWORD 'verysecret';

RESET SESSION AUTHORIZATION;

ALTER GROUP regress_priv_group2 DROP USER regress_priv_user2;

REVOKE ADMIN OPTION FOR regress_priv_group2 FROM regress_priv_user1;

GRANT regress_priv_group2 TO regress_priv_user4 WITH ADMIN OPTION;

CREATE FUNCTION leak(integer,integer) RETURNS boolean
  AS 'int4lt'
  LANGUAGE internal IMMUTABLE STRICT;

ALTER FUNCTION leak(integer,integer) OWNER TO regress_priv_user1;

GRANT regress_priv_role TO regress_priv_user1 WITH ADMIN OPTION GRANTED BY regress_priv_role;

GRANT regress_priv_role TO regress_priv_user1 WITH ADMIN OPTION GRANTED BY CURRENT_ROLE;

REVOKE ADMIN OPTION FOR regress_priv_role FROM regress_priv_user1 GRANTED BY foo;

REVOKE ADMIN OPTION FOR regress_priv_role FROM regress_priv_user1 GRANTED BY regress_priv_user2;

REVOKE ADMIN OPTION FOR regress_priv_role FROM regress_priv_user1 GRANTED BY CURRENT_USER;

REVOKE regress_priv_role FROM regress_priv_user1 GRANTED BY CURRENT_ROLE;

DROP ROLE regress_priv_role;

SET SESSION AUTHORIZATION regress_priv_user1;

SELECT session_user, current_user;

CREATE TABLE atest1 ( a int, b text );

SELECT * FROM atest1;

INSERT INTO atest1 VALUES (1, 'one');

DELETE FROM atest1;

UPDATE atest1 SET a = 1 WHERE b = 'blech';

TRUNCATE atest1;

BEGIN;

LOCK atest1 IN ACCESS EXCLUSIVE MODE;

COMMIT;

REVOKE ALL ON atest1 FROM PUBLIC;

SELECT * FROM atest1;

GRANT ALL ON atest1 TO regress_priv_user2;

GRANT SELECT ON atest1 TO regress_priv_user3, regress_priv_user4;

SELECT * FROM atest1;

CREATE TABLE atest2 (col1 varchar(10), col2 boolean);

SELECT pg_get_acl('pg_class'::regclass, 'atest2'::regclass::oid, 0);

GRANT SELECT ON atest2 TO regress_priv_user2;

GRANT UPDATE ON atest2 TO regress_priv_user3;

GRANT INSERT ON atest2 TO regress_priv_user4 GRANTED BY CURRENT_USER;

GRANT TRUNCATE ON atest2 TO regress_priv_user5 GRANTED BY CURRENT_ROLE;

SELECT unnest(pg_get_acl('pg_class'::regclass, 'atest2'::regclass::oid, 0));

SELECT pg_get_acl('pg_class'::regclass, 0, 0);

SELECT pg_get_acl(0, 0, 0);

GRANT TRUNCATE ON atest2 TO regress_priv_user4 GRANTED BY regress_priv_user5;

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT session_user, current_user;

SELECT * FROM atest1;

SELECT * FROM atest2;

INSERT INTO atest1 VALUES (2, 'two');

INSERT INTO atest2 VALUES ('foo', true);

INSERT INTO atest1 SELECT 1, b FROM atest1;

UPDATE atest1 SET a = 1 WHERE a = 2;

UPDATE atest2 SET col2 = NOT col2;

SELECT * FROM atest1 FOR UPDATE;

SELECT * FROM atest2 FOR UPDATE;

DELETE FROM atest2;

TRUNCATE atest2;

BEGIN;

LOCK atest2 IN ACCESS EXCLUSIVE MODE;

COMMIT;

SELECT * FROM atest1 WHERE ( b IN ( SELECT col1 FROM atest2 ) );

SELECT * FROM atest2 WHERE ( col1 IN ( SELECT b FROM atest1 ) );

SET SESSION AUTHORIZATION regress_priv_user6;

SELECT * FROM atest1;

SELECT * FROM atest2;

INSERT INTO atest2 VALUES ('foo', true);

SET SESSION AUTHORIZATION regress_priv_user7;

SELECT * FROM atest1;

SELECT * FROM atest2;

INSERT INTO atest2 VALUES ('foo', true);

UPDATE atest2 SET col2 = true;

DELETE FROM atest2;

UPDATE pg_catalog.pg_class SET relname = '123';

DELETE FROM pg_catalog.pg_class;

UPDATE pg_toast.pg_toast_1213 SET chunk_id = 1;

SET SESSION AUTHORIZATION regress_priv_user3;

SELECT session_user, current_user;

SELECT * FROM atest1;

SELECT * FROM atest2;

INSERT INTO atest1 VALUES (2, 'two');

INSERT INTO atest2 VALUES ('foo', true);

INSERT INTO atest1 SELECT 1, b FROM atest1;

UPDATE atest1 SET a = 1 WHERE a = 2;

UPDATE atest2 SET col2 = NULL;

UPDATE atest2 SET col2 = NOT col2;

UPDATE atest2 SET col2 = true FROM atest1 WHERE atest1.a = 5;

SELECT * FROM atest1 FOR UPDATE;

SELECT * FROM atest2 FOR UPDATE;

DELETE FROM atest2;

TRUNCATE atest2;

BEGIN;

LOCK atest2 IN ACCESS EXCLUSIVE MODE;

COMMIT;

SELECT * FROM atest1 WHERE ( b IN ( SELECT col1 FROM atest2 ) );

SELECT * FROM atest2 WHERE ( col1 IN ( SELECT b FROM atest1 ) );

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT * FROM atest1;

SET SESSION AUTHORIZATION regress_priv_user1;

CREATE TABLE atest12 as
  SELECT x AS a, 10001 - x AS b FROM generate_series(1,10000) x;

CREATE INDEX ON atest12 (a);

CREATE INDEX ON atest12 (abs(a));

ALTER TABLE atest12 SET (autovacuum_enabled = off);

SET default_statistics_target = 10000;

VACUUM ANALYZE atest12;

RESET default_statistics_target;

CREATE OPERATOR <<< (procedure = leak, leftarg = integer, rightarg = integer,
                     restrict = scalarltsel);

CREATE VIEW atest12v AS
  SELECT * FROM atest12 WHERE b <<< 5;

CREATE VIEW atest12sbv WITH (security_barrier=true) AS
  SELECT * FROM atest12 WHERE b <<< 5;

SELECT * FROM atest12v x, atest12v y WHERE x.a = y.b;

SELECT * FROM atest12 x, atest12 y
  WHERE x.a = y.b and abs(y.a) <<< 5;

SELECT * FROM atest12sbv x, atest12sbv y WHERE x.a = y.b;

SET SESSION AUTHORIZATION regress_priv_user2;

CREATE FUNCTION leak2(integer,integer) RETURNS boolean
  AS $$begin raise notice 'leak % %', $1, $2; return $1 > $2; end$$
  LANGUAGE plpgsql immutable;

CREATE OPERATOR >>> (procedure = leak2, leftarg = integer, rightarg = integer,
                     restrict = scalargtsel);

SELECT * FROM atest12 WHERE a >>> 0;

SELECT * FROM atest12v WHERE a >>> 0;

SELECT * FROM atest12sbv WHERE a >>> 0;

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT SELECT ON atest12v TO PUBLIC;

GRANT SELECT ON atest12sbv TO PUBLIC;

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT * FROM atest12v x, atest12v y WHERE x.a = y.b;

SELECT * FROM atest12sbv x, atest12sbv y WHERE x.a = y.b;

SELECT * FROM atest12v x, atest12v y
  WHERE x.a = y.b and abs(y.a) <<< 5;

SELECT * FROM atest12sbv x, atest12sbv y
  WHERE x.a = y.b and abs(y.a) <<< 5;

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT SELECT (a, b) ON atest12 TO PUBLIC;

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT * FROM atest12v x, atest12v y WHERE x.a = y.b;

SELECT * FROM atest12 x, atest12 y
  WHERE x.a = y.b and abs(y.a) <<< 5;

DROP FUNCTION leak2(integer, integer) CASCADE;

SET SESSION AUTHORIZATION regress_priv_user3;

CREATE TABLE atest3 (one int, two int, three int);

GRANT DELETE ON atest3 TO GROUP regress_priv_group2;

SET SESSION AUTHORIZATION regress_priv_user1;

SELECT * FROM atest3;

DELETE FROM atest3;

BEGIN;

RESET SESSION AUTHORIZATION;

ALTER ROLE regress_priv_user1 NOINHERIT;

SET SESSION AUTHORIZATION regress_priv_user1;

SAVEPOINT s1;

DELETE FROM atest3;

ROLLBACK TO s1;

RESET SESSION AUTHORIZATION;

GRANT regress_priv_group2 TO regress_priv_user1 WITH INHERIT FALSE;

SET SESSION AUTHORIZATION regress_priv_user1;

DELETE FROM atest3;

ROLLBACK TO s1;

RESET SESSION AUTHORIZATION;

REVOKE INHERIT OPTION FOR regress_priv_group2 FROM regress_priv_user1;

SET SESSION AUTHORIZATION regress_priv_user1;

DELETE FROM atest3;

ROLLBACK;

SET SESSION AUTHORIZATION regress_priv_user3;

CREATE VIEW atestv1 AS SELECT * FROM atest1;

CREATE VIEW atestv2 AS SELECT * FROM atest2;

CREATE VIEW atestv3 AS SELECT * FROM atest3;

CREATE VIEW atestv0 AS SELECT 0 as x WHERE false;

SELECT * FROM atestv1;

SELECT * FROM atestv2;

GRANT SELECT ON atestv1, atestv3 TO regress_priv_user4;

GRANT SELECT ON atestv2 TO regress_priv_user2;

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT * FROM atestv1;

SELECT * FROM atestv2;

SELECT * FROM atestv3;

SELECT * FROM atestv0;

select * from
  ((select a.q1 as x from int8_tbl a offset 0)
   union all
   (select b.q2 as x from int8_tbl b offset 0)) ss
where false;

set constraint_exclusion = on;

select * from
  ((select a.q1 as x, random() from int8_tbl a where q1 > 0)
   union all
   (select b.q2 as x, random() from int8_tbl b where q2 > 0)) ss
where x < 0;

reset constraint_exclusion;

CREATE VIEW atestv4 AS SELECT * FROM atestv3;

SELECT * FROM atestv4;

GRANT SELECT ON atestv4 TO regress_priv_user2;

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT * FROM atestv3;

SELECT * FROM atestv4;

SELECT * FROM atest2;

SELECT * FROM atestv2;

SET SESSION AUTHORIZATION regress_priv_user1;

CREATE TABLE atest5 (one int, two int unique, three int, four int unique);

CREATE TABLE atest6 (one int, two int, blue int);

GRANT SELECT (one), INSERT (two), UPDATE (three) ON atest5 TO regress_priv_user4;

GRANT ALL (one) ON atest5 TO regress_priv_user3;

SELECT unnest(pg_get_acl('pg_class'::regclass, 'atest5'::regclass::oid, 1));

SELECT unnest(pg_get_acl('pg_class'::regclass, 'atest5'::regclass::oid, 2));

SELECT unnest(pg_get_acl('pg_class'::regclass, 'atest5'::regclass::oid, 3));

SELECT unnest(pg_get_acl('pg_class'::regclass, 'atest5'::regclass::oid, 4));

INSERT INTO atest5 VALUES (1,2,3);

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT * FROM atest5;

SELECT one FROM atest5;

COPY atest5 (one) TO stdout;

SELECT two FROM atest5;

COPY atest5 (two) TO stdout;

SELECT atest5 FROM atest5;

COPY atest5 (one,two) TO stdout;

SELECT 1 FROM atest5;

SELECT 1 FROM atest5 a JOIN atest5 b USING (one);

SELECT 1 FROM atest5 a JOIN atest5 b USING (two);

SELECT 1 FROM atest5 a NATURAL JOIN atest5 b;

SELECT * FROM (atest5 a JOIN atest5 b USING (one)) j;

SELECT j.* FROM (atest5 a JOIN atest5 b USING (one)) j;

SELECT (j.*) IS NULL FROM (atest5 a JOIN atest5 b USING (one)) j;

SELECT one FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one)) j;

SELECT j.one FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one)) j;

SELECT two FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one)) j;

SELECT j.two FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one)) j;

SELECT y FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one)) j;

SELECT j.y FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one)) j;

SELECT * FROM (atest5 a JOIN atest5 b USING (one));

SELECT a.* FROM (atest5 a JOIN atest5 b USING (one));

SELECT (a.*) IS NULL FROM (atest5 a JOIN atest5 b USING (one));

SELECT two FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one));

SELECT a.two FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one));

SELECT y FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one));

SELECT b.y FROM (atest5 a JOIN atest5 b(one,x,y,z) USING (one));

SELECT y FROM (atest5 a LEFT JOIN atest5 b(one,x,y,z) USING (one));

SELECT b.y FROM (atest5 a LEFT JOIN atest5 b(one,x,y,z) USING (one));

SELECT y FROM (atest5 a FULL JOIN atest5 b(one,x,y,z) USING (one));

SELECT b.y FROM (atest5 a FULL JOIN atest5 b(one,x,y,z) USING (one));

SELECT 1 FROM atest5 WHERE two = 2;

SELECT * FROM atest1, atest5;

SELECT atest1.* FROM atest1, atest5;

SELECT atest1.*,atest5.one FROM atest1, atest5;

SELECT atest1.*,atest5.one FROM atest1 JOIN atest5 ON (atest1.a = atest5.two);

SELECT atest1.*,atest5.one FROM atest1 JOIN atest5 ON (atest1.a = atest5.one);

SELECT one, two FROM atest5;

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT SELECT (one,two) ON atest6 TO regress_priv_user4;

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT one, two FROM atest5 NATURAL JOIN atest6;

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT SELECT (two) ON atest5 TO regress_priv_user4;

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT one, two FROM atest5 NATURAL JOIN atest6;

INSERT INTO atest5 (two) VALUES (3);

INSERT INTO atest5 (three) VALUES (4);

INSERT INTO atest5 VALUES (5,5,5);

UPDATE atest5 SET three = 10;

UPDATE atest5 SET one = 8;

UPDATE atest5 SET three = 5, one = 2;

INSERT INTO atest5(two) VALUES (6) ON CONFLICT (two) DO UPDATE set three = 10;

INSERT INTO atest5(two) VALUES (6) ON CONFLICT (two) DO UPDATE set three = 10 RETURNING atest5.three;

INSERT INTO atest5(two) VALUES (6) ON CONFLICT (two) DO UPDATE set three = 10 RETURNING atest5.one;

INSERT INTO atest5(two) VALUES (6) ON CONFLICT (two) DO UPDATE set three = EXCLUDED.one;

INSERT INTO atest5(two) VALUES (6) ON CONFLICT (two) DO UPDATE set three = EXCLUDED.three;

INSERT INTO atest5(two) VALUES (6) ON CONFLICT (two) DO UPDATE set one = 8;

INSERT INTO atest5(three) VALUES (4) ON CONFLICT (two) DO UPDATE set three = 10;

INSERT INTO atest5(four) VALUES (4);

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT INSERT (four) ON atest5 TO regress_priv_user4;

SET SESSION AUTHORIZATION regress_priv_user4;

INSERT INTO atest5(four) VALUES (4) ON CONFLICT (four) DO UPDATE set three = 3;

INSERT INTO atest5(four) VALUES (4) ON CONFLICT ON CONSTRAINT atest5_four_key DO UPDATE set three = 3;

INSERT INTO atest5(four) VALUES (4);

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT SELECT (four) ON atest5 TO regress_priv_user4;

SET SESSION AUTHORIZATION regress_priv_user4;

INSERT INTO atest5(four) VALUES (4) ON CONFLICT (four) DO UPDATE set three = 3;

INSERT INTO atest5(four) VALUES (4) ON CONFLICT ON CONSTRAINT atest5_four_key DO UPDATE set three = 3;

SET SESSION AUTHORIZATION regress_priv_user1;

REVOKE ALL (one) ON atest5 FROM regress_priv_user4;

GRANT SELECT (one,two,blue) ON atest6 TO regress_priv_user4;

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT one FROM atest5;

UPDATE atest5 SET one = 1;

SELECT atest6 FROM atest6;

COPY atest6 TO stdout;

SET SESSION AUTHORIZATION regress_priv_user1;

CREATE TABLE mtarget (a int, b text);

CREATE TABLE msource (a int, b text);

INSERT INTO mtarget VALUES (1, 'init1'), (2, 'init2');

INSERT INTO msource VALUES (1, 'source1'), (2, 'source2'), (3, 'source3');

GRANT SELECT (a) ON msource TO regress_priv_user4;

GRANT SELECT (a) ON mtarget TO regress_priv_user4;

GRANT INSERT (a,b) ON mtarget TO regress_priv_user4;

GRANT UPDATE (b) ON mtarget TO regress_priv_user4;

SET SESSION AUTHORIZATION regress_priv_user4;

BEGIN;

ROLLBACK;

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT SELECT (b) ON msource TO regress_priv_user4;

SET SESSION AUTHORIZATION regress_priv_user4;

BEGIN;

ROLLBACK;

BEGIN;

ROLLBACK;

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT DELETE ON mtarget TO regress_priv_user4;

BEGIN;

ROLLBACK;

SET SESSION AUTHORIZATION regress_priv_user1;

CREATE TABLE t1 (c1 int, c2 int, c3 int check (c3 < 5), primary key (c1, c2));

GRANT SELECT (c1) ON t1 TO regress_priv_user2;

GRANT INSERT (c1, c2, c3) ON t1 TO regress_priv_user2;

GRANT UPDATE (c1, c2, c3) ON t1 TO regress_priv_user2;

INSERT INTO t1 VALUES (1, 1, 1);

INSERT INTO t1 VALUES (1, 2, 1);

INSERT INTO t1 VALUES (2, 1, 2);

INSERT INTO t1 VALUES (2, 2, 2);

INSERT INTO t1 VALUES (3, 1, 3);

SET SESSION AUTHORIZATION regress_priv_user2;

INSERT INTO t1 (c1, c2) VALUES (1, 1);

UPDATE t1 SET c2 = 1;

INSERT INTO t1 (c1, c2) VALUES (null, null);

INSERT INTO t1 (c3) VALUES (null);

INSERT INTO t1 (c1) VALUES (5);

UPDATE t1 SET c3 = 10;

SET SESSION AUTHORIZATION regress_priv_user1;

DROP TABLE t1;

CREATE TABLE errtst(a text, b text NOT NULL, c text, secret1 text, secret2 text) PARTITION BY LIST (a);

CREATE TABLE errtst_part_1(secret2 text, c text, a text, b text NOT NULL, secret1 text);

CREATE TABLE errtst_part_2(secret1 text, secret2 text, a text, c text, b text NOT NULL);

ALTER TABLE errtst ATTACH PARTITION errtst_part_1 FOR VALUES IN ('aaa');

ALTER TABLE errtst ATTACH PARTITION errtst_part_2 FOR VALUES IN ('aaaa');

GRANT SELECT (a, b, c) ON TABLE errtst TO regress_priv_user2;

GRANT UPDATE (a, b, c) ON TABLE errtst TO regress_priv_user2;

GRANT INSERT (a, b, c) ON TABLE errtst TO regress_priv_user2;

INSERT INTO errtst_part_1 (a, b, c, secret1, secret2)
VALUES ('aaa', 'bbb', 'ccc', 'the body', 'is in the attic');

SET SESSION AUTHORIZATION regress_priv_user2;

INSERT INTO errtst (a, b) VALUES ('aaa', NULL);

UPDATE errtst SET b = NULL;

UPDATE errtst SET a = 'aaa', b = NULL;

UPDATE errtst SET a = 'aaaa', b = NULL;

UPDATE errtst SET a = 'aaaa', b = NULL WHERE a = 'aaa';

SET SESSION AUTHORIZATION regress_priv_user1;

DROP TABLE errtst;

SET SESSION AUTHORIZATION regress_priv_user1;

ALTER TABLE atest6 ADD COLUMN three integer;

GRANT DELETE ON atest5 TO regress_priv_user3;

GRANT SELECT (two) ON atest5 TO regress_priv_user3;

REVOKE ALL (one) ON atest5 FROM regress_priv_user3;

GRANT SELECT (one) ON atest5 TO regress_priv_user4;

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT atest6 FROM atest6;

SELECT one FROM atest5 NATURAL JOIN atest6;

SET SESSION AUTHORIZATION regress_priv_user1;

ALTER TABLE atest6 DROP COLUMN three;

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT atest6 FROM atest6;

SELECT one FROM atest5 NATURAL JOIN atest6;

SET SESSION AUTHORIZATION regress_priv_user1;

ALTER TABLE atest6 DROP COLUMN two;

REVOKE SELECT (one,blue) ON atest6 FROM regress_priv_user4;

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT * FROM atest6;

SELECT 1 FROM atest6;

SET SESSION AUTHORIZATION regress_priv_user3;

DELETE FROM atest5 WHERE one = 1;

DELETE FROM atest5 WHERE two = 2;

SET SESSION AUTHORIZATION regress_priv_user1;

CREATE TABLE atestp1 (f1 int, f2 int);

CREATE TABLE atestp2 (fx int, fy int);

CREATE TABLE atestc (fz int) INHERITS (atestp1, atestp2);

GRANT SELECT(fx,fy,tableoid) ON atestp2 TO regress_priv_user2;

GRANT SELECT(fx) ON atestc TO regress_priv_user2;

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT fx FROM atestp2;

SELECT fy FROM atestp2;

SELECT atestp2 FROM atestp2;

SELECT tableoid FROM atestp2;

SELECT fy FROM atestc;

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT SELECT(fy,tableoid) ON atestc TO regress_priv_user2;

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT fx FROM atestp2;

SELECT fy FROM atestp2;

SELECT atestp2 FROM atestp2;

SELECT tableoid FROM atestp2;

SET SESSION AUTHORIZATION regress_priv_user1;

REVOKE ALL ON atestc FROM regress_priv_user2;

GRANT ALL ON atestp1 TO regress_priv_user2;

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT f2 FROM atestp1;

SELECT f2 FROM atestc;

DELETE FROM atestp1;

DELETE FROM atestc;

UPDATE atestp1 SET f1 = 1;

UPDATE atestc SET f1 = 1;

TRUNCATE atestp1;

TRUNCATE atestc;

BEGIN;

LOCK atestp1;

END;

BEGIN;

LOCK atestc;

END;

REVOKE ALL PRIVILEGES ON LANGUAGE sql FROM PUBLIC;

GRANT USAGE ON LANGUAGE sql TO regress_priv_user1;

GRANT USAGE ON LANGUAGE c TO PUBLIC;

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT USAGE ON LANGUAGE sql TO regress_priv_user2;

CREATE FUNCTION priv_testfunc1(int) RETURNS int AS 'select 2 * $1;' LANGUAGE sql;

CREATE FUNCTION priv_testfunc2(int) RETURNS int AS 'select 3 * $1;' LANGUAGE sql;

CREATE AGGREGATE priv_testagg1(int) (sfunc = int4pl, stype = int4);

CREATE PROCEDURE priv_testproc1(int) AS 'select $1;' LANGUAGE sql;

REVOKE ALL ON FUNCTION priv_testfunc1(int), priv_testfunc2(int), priv_testagg1(int) FROM PUBLIC;

GRANT EXECUTE ON FUNCTION priv_testfunc1(int), priv_testfunc2(int), priv_testagg1(int) TO regress_priv_user2;

REVOKE ALL ON FUNCTION priv_testproc1(int) FROM PUBLIC;

REVOKE ALL ON PROCEDURE priv_testproc1(int) FROM PUBLIC;

GRANT EXECUTE ON PROCEDURE priv_testproc1(int) TO regress_priv_user2;

GRANT USAGE ON FUNCTION priv_testfunc1(int) TO regress_priv_user3;

GRANT USAGE ON FUNCTION priv_testagg1(int) TO regress_priv_user3;

GRANT USAGE ON PROCEDURE priv_testproc1(int) TO regress_priv_user3;

GRANT ALL PRIVILEGES ON FUNCTION priv_testfunc1(int) TO regress_priv_user4;

GRANT ALL PRIVILEGES ON FUNCTION priv_testfunc_nosuch(int) TO regress_priv_user4;

GRANT ALL PRIVILEGES ON FUNCTION priv_testagg1(int) TO regress_priv_user4;

GRANT ALL PRIVILEGES ON PROCEDURE priv_testproc1(int) TO regress_priv_user4;

CREATE FUNCTION priv_testfunc4(boolean) RETURNS text
  AS 'select col1 from atest2 where col2 = $1;'
  LANGUAGE sql SECURITY DEFINER;

GRANT EXECUTE ON FUNCTION priv_testfunc4(boolean) TO regress_priv_user3;

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT priv_testfunc1(5), priv_testfunc2(5);

CREATE FUNCTION priv_testfunc3(int) RETURNS int AS 'select 2 * $1;' LANGUAGE sql;

SELECT priv_testagg1(x) FROM (VALUES (1), (2), (3)) _(x);

CALL priv_testproc1(6);

SET SESSION AUTHORIZATION regress_priv_user3;

SELECT priv_testfunc1(5);

SELECT priv_testagg1(x) FROM (VALUES (1), (2), (3)) _(x);

CALL priv_testproc1(6);

SELECT col1 FROM atest2 WHERE col2 = true;

SELECT priv_testfunc4(true);

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT priv_testfunc1(5);

SELECT priv_testagg1(x) FROM (VALUES (1), (2), (3)) _(x);

CALL priv_testproc1(6);

DROP FUNCTION priv_testfunc1(int);

DROP AGGREGATE priv_testagg1(int);

DROP PROCEDURE priv_testproc1(int);

DROP FUNCTION priv_testfunc1(int);

GRANT ALL PRIVILEGES ON LANGUAGE sql TO PUBLIC;

BEGIN;

SELECT '{1}'::int4[]::int8[];

REVOKE ALL ON FUNCTION int8(integer) FROM PUBLIC;

SELECT '{1}'::int4[]::int8[];

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT '{1}'::int4[]::int8[];

ROLLBACK;

CREATE TYPE priv_testtype1 AS (a int, b text);

REVOKE USAGE ON TYPE priv_testtype1 FROM PUBLIC;

GRANT USAGE ON TYPE priv_testtype1 TO regress_priv_user2;

GRANT USAGE ON TYPE _priv_testtype1 TO regress_priv_user2;

GRANT USAGE ON DOMAIN priv_testtype1 TO regress_priv_user2;

CREATE DOMAIN priv_testdomain1 AS int;

REVOKE USAGE on DOMAIN priv_testdomain1 FROM PUBLIC;

GRANT USAGE ON DOMAIN priv_testdomain1 TO regress_priv_user2;

GRANT USAGE ON TYPE priv_testdomain1 TO regress_priv_user2;

SET SESSION AUTHORIZATION regress_priv_user1;

CREATE AGGREGATE priv_testagg1a(priv_testdomain1) (sfunc = int4_sum, stype = bigint);

CREATE DOMAIN priv_testdomain2a AS priv_testdomain1;

CREATE DOMAIN priv_testdomain3a AS int;

CREATE FUNCTION castfunc(int) RETURNS priv_testdomain3a AS $$ SELECT $1::priv_testdomain3a $$ LANGUAGE SQL;

CREATE CAST (priv_testdomain1 AS priv_testdomain3a) WITH FUNCTION castfunc(int);

DROP FUNCTION castfunc(int) CASCADE;

DROP DOMAIN priv_testdomain3a;

CREATE FUNCTION priv_testfunc5a(a priv_testdomain1) RETURNS int LANGUAGE SQL AS $$ SELECT $1 $$;

CREATE FUNCTION priv_testfunc6a(b int) RETURNS priv_testdomain1 LANGUAGE SQL AS $$ SELECT $1::priv_testdomain1 $$;

CREATE OPERATOR !+! (PROCEDURE = int4pl, LEFTARG = priv_testdomain1, RIGHTARG = priv_testdomain1);

CREATE TABLE test5a (a int, b priv_testdomain1);

CREATE TABLE test6a OF priv_testtype1;

CREATE TABLE test10a (a int[], b priv_testtype1[]);

CREATE TABLE test9a (a int, b int);

ALTER TABLE test9a ADD COLUMN c priv_testdomain1;

ALTER TABLE test9a ALTER COLUMN b TYPE priv_testdomain1;

CREATE TYPE test7a AS (a int, b priv_testdomain1);

CREATE TYPE test8a AS (a int, b int);

ALTER TYPE test8a ADD ATTRIBUTE c priv_testdomain1;

ALTER TYPE test8a ALTER ATTRIBUTE b TYPE priv_testdomain1;

CREATE TABLE test11a AS (SELECT 1::priv_testdomain1 AS a);

REVOKE ALL ON TYPE priv_testtype1 FROM PUBLIC;

SET SESSION AUTHORIZATION regress_priv_user2;

CREATE AGGREGATE priv_testagg1b(priv_testdomain1) (sfunc = int4_sum, stype = bigint);

CREATE DOMAIN priv_testdomain2b AS priv_testdomain1;

CREATE DOMAIN priv_testdomain3b AS int;

CREATE FUNCTION castfunc(int) RETURNS priv_testdomain3b AS $$ SELECT $1::priv_testdomain3b $$ LANGUAGE SQL;

CREATE CAST (priv_testdomain1 AS priv_testdomain3b) WITH FUNCTION castfunc(int);

CREATE FUNCTION priv_testfunc5b(a priv_testdomain1) RETURNS int LANGUAGE SQL AS $$ SELECT $1 $$;

CREATE FUNCTION priv_testfunc6b(b int) RETURNS priv_testdomain1 LANGUAGE SQL AS $$ SELECT $1::priv_testdomain1 $$;

CREATE OPERATOR !! (PROCEDURE = priv_testfunc5b, RIGHTARG = priv_testdomain1);

CREATE TABLE test5b (a int, b priv_testdomain1);

CREATE TABLE test6b OF priv_testtype1;

CREATE TABLE test10b (a int[], b priv_testtype1[]);

CREATE TABLE test9b (a int, b int);

ALTER TABLE test9b ADD COLUMN c priv_testdomain1;

ALTER TABLE test9b ALTER COLUMN b TYPE priv_testdomain1;

CREATE TYPE test7b AS (a int, b priv_testdomain1);

CREATE TYPE test8b AS (a int, b int);

ALTER TYPE test8b ADD ATTRIBUTE c priv_testdomain1;

ALTER TYPE test8b ALTER ATTRIBUTE b TYPE priv_testdomain1;

CREATE TABLE test11b AS (SELECT 1::priv_testdomain1 AS a);

REVOKE ALL ON TYPE priv_testtype1 FROM PUBLIC;

DROP AGGREGATE priv_testagg1b(priv_testdomain1);

DROP DOMAIN priv_testdomain2b;

DROP OPERATOR !! (NONE, priv_testdomain1);

DROP FUNCTION priv_testfunc5b(a priv_testdomain1);

DROP FUNCTION priv_testfunc6b(b int);

DROP TABLE test5b;

DROP TABLE test6b;

DROP TABLE test9b;

DROP TABLE test10b;

DROP TYPE test7b;

DROP TYPE test8b;

DROP CAST (priv_testdomain1 AS priv_testdomain3b);

DROP FUNCTION castfunc(int) CASCADE;

DROP DOMAIN priv_testdomain3b;

DROP TABLE test11b;

DROP TYPE priv_testtype1;

DROP DOMAIN priv_testdomain1;

SET SESSION AUTHORIZATION regress_priv_user5;

TRUNCATE atest2;

TRUNCATE atest3;

select has_table_privilege(NULL,'pg_authid','select');

select has_table_privilege('pg_shad','select');

select has_table_privilege('nosuchuser','pg_authid','select');

select has_table_privilege('pg_authid','sel');

select has_table_privilege(-999999,'pg_authid','update');

select has_table_privilege(1,'select');

select has_table_privilege(current_user,'pg_authid','select');

select has_table_privilege(current_user,'pg_authid','insert');

select has_table_privilege(t2.oid,'pg_authid','update')
from (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege(t2.oid,'pg_authid','delete')
from (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege(current_user,t1.oid,'references')
from (select oid from pg_class where relname = 'pg_authid') as t1;

select has_table_privilege(t2.oid,t1.oid,'select')
from (select oid from pg_class where relname = 'pg_authid') as t1,
  (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege(t2.oid,t1.oid,'insert')
from (select oid from pg_class where relname = 'pg_authid') as t1,
  (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege('pg_authid','update');

select has_table_privilege('pg_authid','delete');

select has_table_privilege('pg_authid','truncate');

select has_table_privilege(t1.oid,'select')
from (select oid from pg_class where relname = 'pg_authid') as t1;

select has_table_privilege(t1.oid,'trigger')
from (select oid from pg_class where relname = 'pg_authid') as t1;

SET SESSION AUTHORIZATION regress_priv_user3;

select has_table_privilege(current_user,'pg_class','select');

select has_table_privilege(current_user,'pg_class','insert');

select has_table_privilege(t2.oid,'pg_class','update')
from (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege(t2.oid,'pg_class','delete')
from (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege(current_user,t1.oid,'references')
from (select oid from pg_class where relname = 'pg_class') as t1;

select has_table_privilege(t2.oid,t1.oid,'select')
from (select oid from pg_class where relname = 'pg_class') as t1,
  (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege(t2.oid,t1.oid,'insert')
from (select oid from pg_class where relname = 'pg_class') as t1,
  (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege('pg_class','update');

select has_table_privilege('pg_class','delete');

select has_table_privilege('pg_class','truncate');

select has_table_privilege(t1.oid,'select')
from (select oid from pg_class where relname = 'pg_class') as t1;

select has_table_privilege(t1.oid,'trigger')
from (select oid from pg_class where relname = 'pg_class') as t1;

select has_table_privilege(current_user,'atest1','select');

select has_table_privilege(current_user,'atest1','insert');

select has_table_privilege(t2.oid,'atest1','update')
from (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege(t2.oid,'atest1','delete')
from (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege(current_user,t1.oid,'references')
from (select oid from pg_class where relname = 'atest1') as t1;

select has_table_privilege(t2.oid,t1.oid,'select')
from (select oid from pg_class where relname = 'atest1') as t1,
  (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege(t2.oid,t1.oid,'insert')
from (select oid from pg_class where relname = 'atest1') as t1,
  (select oid from pg_roles where rolname = current_user) as t2;

select has_table_privilege('atest1','update');

select has_table_privilege('atest1','delete');

select has_table_privilege('atest1','truncate');

select has_table_privilege(t1.oid,'select')
from (select oid from pg_class where relname = 'atest1') as t1;

select has_table_privilege(t1.oid,'trigger')
from (select oid from pg_class where relname = 'atest1') as t1;

select has_column_privilege('pg_authid',NULL,'select');

select has_column_privilege('pg_authid','nosuchcol','select');

select has_column_privilege(9999,'nosuchcol','select');

select has_column_privilege(9999,99::int2,'select');

select has_column_privilege('pg_authid',99::int2,'select');

select has_column_privilege(9999,99::int2,'select');

create temp table mytable(f1 int, f2 int, f3 int);

alter table mytable drop column f2;

select has_column_privilege('mytable','f2','select');

select has_column_privilege('mytable','........pg.dropped.2........','select');

select has_column_privilege('mytable',2::int2,'select');

select has_column_privilege('mytable',99::int2,'select');

revoke select on table mytable from regress_priv_user3;

select has_column_privilege('mytable',2::int2,'select');

select has_column_privilege('mytable',99::int2,'select');

drop table mytable;

SET SESSION AUTHORIZATION regress_priv_user1;

CREATE TABLE atest4 (a int);

GRANT SELECT ON atest4 TO regress_priv_user2 WITH GRANT OPTION;

GRANT UPDATE ON atest4 TO regress_priv_user2;

GRANT SELECT ON atest4 TO GROUP regress_priv_group1 WITH GRANT OPTION;

SET SESSION AUTHORIZATION regress_priv_user2;

GRANT SELECT ON atest4 TO regress_priv_user3;

GRANT UPDATE ON atest4 TO regress_priv_user3;

SET SESSION AUTHORIZATION regress_priv_user1;

REVOKE SELECT ON atest4 FROM regress_priv_user3;

SELECT has_table_privilege('regress_priv_user3', 'atest4', 'SELECT');

REVOKE SELECT ON atest4 FROM regress_priv_user2;

REVOKE GRANT OPTION FOR SELECT ON atest4 FROM regress_priv_user2 CASCADE;

SELECT has_table_privilege('regress_priv_user2', 'atest4', 'SELECT');

SELECT has_table_privilege('regress_priv_user3', 'atest4', 'SELECT');

SELECT has_table_privilege('regress_priv_user1', 'atest4', 'SELECT WITH GRANT OPTION');

CREATE ROLE regress_sro_user;

CREATE FUNCTION sro_ifun(int) RETURNS int AS $$
BEGIN
	-- Below we set the table's owner to regress_sro_user
	ASSERT current_user = 'regress_sro_user',
		format('sro_ifun(%s) called by %s', $1, current_user);
	RETURN $1;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE TABLE sro_tab (a int);

ALTER TABLE sro_tab OWNER TO regress_sro_user;

INSERT INTO sro_tab VALUES (1), (2), (3);

CREATE INDEX sro_idx ON sro_tab ((sro_ifun(a) + sro_ifun(0)))
	WHERE sro_ifun(a + 10) > sro_ifun(10);

DROP INDEX sro_idx;

CREATE INDEX CONCURRENTLY sro_idx ON sro_tab ((sro_ifun(a) + sro_ifun(0)))
	WHERE sro_ifun(a + 10) > sro_ifun(10);

REINDEX TABLE sro_tab;

REINDEX INDEX sro_idx;

REINDEX TABLE CONCURRENTLY sro_tab;

DROP INDEX sro_idx;

CREATE INDEX sro_cluster_idx ON sro_tab ((sro_ifun(a) + sro_ifun(0)));

CLUSTER sro_tab USING sro_cluster_idx;

DROP INDEX sro_cluster_idx;

CREATE INDEX sro_brin ON sro_tab USING brin ((sro_ifun(a) + sro_ifun(0)));

SELECT brin_desummarize_range('sro_brin', 0);

SELECT brin_summarize_range('sro_brin', 0);

DROP TABLE sro_tab;

CREATE TABLE sro_ptab (a int) PARTITION BY RANGE (a);

ALTER TABLE sro_ptab OWNER TO regress_sro_user;

CREATE TABLE sro_part PARTITION OF sro_ptab FOR VALUES FROM (1) TO (10);

ALTER TABLE sro_part OWNER TO regress_sro_user;

INSERT INTO sro_ptab VALUES (1), (2), (3);

CREATE INDEX sro_pidx ON sro_ptab ((sro_ifun(a) + sro_ifun(0)))
	WHERE sro_ifun(a + 10) > sro_ifun(10);

REINDEX TABLE sro_ptab;

REINDEX INDEX CONCURRENTLY sro_pidx;

SET SESSION AUTHORIZATION regress_sro_user;

CREATE FUNCTION unwanted_grant() RETURNS void LANGUAGE sql AS
	'GRANT regress_priv_group2 TO regress_sro_user';

CREATE FUNCTION mv_action() RETURNS bool LANGUAGE sql AS
	'DECLARE c CURSOR WITH HOLD FOR SELECT public.unwanted_grant(); SELECT true';

CREATE MATERIALIZED VIEW sro_mv AS SELECT mv_action() WITH NO DATA;

REFRESH MATERIALIZED VIEW sro_mv;

REFRESH MATERIALIZED VIEW sro_mv;

SET SESSION AUTHORIZATION regress_sro_user;

CREATE TABLE sro_trojan_table ();

CREATE FUNCTION sro_trojan() RETURNS trigger LANGUAGE plpgsql AS
	'BEGIN PERFORM public.unwanted_grant(); RETURN NULL; END';

CREATE CONSTRAINT TRIGGER t AFTER INSERT ON sro_trojan_table
    INITIALLY DEFERRED FOR EACH ROW EXECUTE PROCEDURE sro_trojan();

CREATE OR REPLACE FUNCTION mv_action() RETURNS bool LANGUAGE sql AS
	'INSERT INTO public.sro_trojan_table DEFAULT VALUES; SELECT true';

REFRESH MATERIALIZED VIEW sro_mv;

REFRESH MATERIALIZED VIEW sro_mv;

BEGIN;

SET CONSTRAINTS ALL IMMEDIATE;

REFRESH MATERIALIZED VIEW sro_mv;

COMMIT;

SET SESSION AUTHORIZATION regress_sro_user;

CREATE FUNCTION unwanted_grant_nofail(int) RETURNS int
	IMMUTABLE LANGUAGE plpgsql AS $$
BEGIN
	PERFORM public.unwanted_grant();
	RAISE WARNING 'owned';
	RETURN 1;
EXCEPTION WHEN OTHERS THEN
	RETURN 2;
END$$;

CREATE MATERIALIZED VIEW sro_index_mv AS SELECT 1 AS c;

CREATE UNIQUE INDEX ON sro_index_mv (c) WHERE unwanted_grant_nofail(1) > 0;

REFRESH MATERIALIZED VIEW CONCURRENTLY sro_index_mv;

REFRESH MATERIALIZED VIEW sro_index_mv;

DROP OWNED BY regress_sro_user;

DROP ROLE regress_sro_user;

SET SESSION AUTHORIZATION regress_priv_user4;

CREATE FUNCTION dogrant_ok() RETURNS void LANGUAGE sql SECURITY DEFINER AS
	'GRANT regress_priv_group2 TO regress_priv_user5';

GRANT regress_priv_group2 TO regress_priv_user5;

SET ROLE regress_priv_group2;

GRANT regress_priv_group2 TO regress_priv_user5;

SET SESSION AUTHORIZATION regress_priv_user1;

GRANT regress_priv_group2 TO regress_priv_user5;

SELECT dogrant_ok();

SET ROLE regress_priv_group2;

GRANT regress_priv_group2 TO regress_priv_user5;

SET SESSION AUTHORIZATION regress_priv_group2;

GRANT regress_priv_group2 TO regress_priv_user5;

SET SESSION AUTHORIZATION regress_priv_user4;

DROP FUNCTION dogrant_ok();

REVOKE regress_priv_group2 FROM regress_priv_user5;

CREATE SEQUENCE x_seq;

GRANT USAGE on x_seq to regress_priv_user2;

SELECT has_sequence_privilege('regress_priv_user1', 'atest1', 'SELECT');

SELECT has_sequence_privilege('regress_priv_user1', 'x_seq', 'INSERT');

SELECT has_sequence_privilege('regress_priv_user1', 'x_seq', 'SELECT');

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT has_sequence_privilege('x_seq', 'USAGE');

SET SESSION AUTHORIZATION regress_priv_user1;

SELECT lo_create(1001);

SELECT lo_create(1002);

SELECT lo_create(1003);

SELECT lo_create(1004);

SELECT lo_create(1005);

GRANT ALL ON LARGE OBJECT 1001 TO PUBLIC;

GRANT SELECT ON LARGE OBJECT 1003 TO regress_priv_user2;

GRANT SELECT,UPDATE ON LARGE OBJECT 1004 TO regress_priv_user2;

GRANT ALL ON LARGE OBJECT 1005 TO regress_priv_user2;

GRANT SELECT ON LARGE OBJECT 1005 TO regress_priv_user2 WITH GRANT OPTION;

GRANT SELECT, INSERT ON LARGE OBJECT 1001 TO PUBLIC;

GRANT SELECT, UPDATE ON LARGE OBJECT 1001 TO nosuchuser;

GRANT SELECT, UPDATE ON LARGE OBJECT  999 TO PUBLIC;

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT lo_create(2001);

SELECT lo_create(2002);

SELECT loread(lo_open(1001, x'20000'::int), 32);

SELECT lowrite(lo_open(1001, x'40000'::int), 'abcd');

SELECT loread(lo_open(1001, x'40000'::int), 32);

SELECT loread(lo_open(1002, x'40000'::int), 32);

SELECT loread(lo_open(1003, x'40000'::int), 32);

SELECT loread(lo_open(1004, x'40000'::int), 32);

SELECT lowrite(lo_open(1001, x'20000'::int), 'abcd');

SELECT lowrite(lo_open(1002, x'20000'::int), 'abcd');

SELECT lowrite(lo_open(1003, x'20000'::int), 'abcd');

SELECT lowrite(lo_open(1004, x'20000'::int), 'abcd');

GRANT SELECT ON LARGE OBJECT 1005 TO regress_priv_user3;

GRANT UPDATE ON LARGE OBJECT 1006 TO regress_priv_user3;

REVOKE ALL ON LARGE OBJECT 2001, 2002 FROM PUBLIC;

GRANT ALL ON LARGE OBJECT 2001 TO regress_priv_user3;

SELECT lo_unlink(1001);

SELECT lo_unlink(2002);

SELECT oid, pg_get_userbyid(lomowner) ownername, lomacl FROM pg_largeobject_metadata WHERE oid >= 1000 AND oid < 3000 ORDER BY oid;

SET SESSION AUTHORIZATION regress_priv_user3;

SELECT loread(lo_open(1001, x'40000'::int), 32);

SELECT loread(lo_open(1003, x'40000'::int), 32);

SELECT loread(lo_open(1005, x'40000'::int), 32);

SELECT lo_truncate(lo_open(1005, x'20000'::int), 10);

SELECT lo_truncate(lo_open(2001, x'20000'::int), 10);

SELECT has_largeobject_privilege(1001, 'SELECT');

SELECT has_largeobject_privilege(1002, 'SELECT');

SELECT has_largeobject_privilege(1003, 'SELECT');

SELECT has_largeobject_privilege(1004, 'SELECT');

SELECT has_largeobject_privilege(1001, 'UPDATE');

SELECT has_largeobject_privilege(1002, 'UPDATE');

SELECT has_largeobject_privilege(1003, 'UPDATE');

SELECT has_largeobject_privilege(1004, 'UPDATE');

SELECT has_largeobject_privilege(9999, 'SELECT');

SET SESSION AUTHORIZATION regress_priv_user2;

SELECT has_largeobject_privilege(1001, 'SELECT');

SELECT has_largeobject_privilege(1002, 'SELECT');

SELECT has_largeobject_privilege(1003, 'SELECT');

SELECT has_largeobject_privilege(1004, 'SELECT');

SELECT has_largeobject_privilege(1001, 'UPDATE');

SELECT has_largeobject_privilege(1002, 'UPDATE');

SELECT has_largeobject_privilege(1003, 'UPDATE');

SELECT has_largeobject_privilege(1004, 'UPDATE');

SELECT has_largeobject_privilege('regress_priv_user3', 1001, 'SELECT');

SELECT has_largeobject_privilege('regress_priv_user3', 1003, 'SELECT');

SELECT has_largeobject_privilege('regress_priv_user3', 1005, 'SELECT');

SELECT has_largeobject_privilege('regress_priv_user3', 1005, 'UPDATE');

SELECT has_largeobject_privilege('regress_priv_user3', 2001, 'UPDATE');

SET lo_compat_privileges = false;

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT has_largeobject_privilege(1002, 'SELECT');

SELECT has_largeobject_privilege(1002, 'UPDATE');

SELECT loread(lo_open(1002, x'40000'::int), 32);

SELECT lowrite(lo_open(1002, x'20000'::int), 'abcd');

SELECT lo_truncate(lo_open(1002, x'20000'::int), 10);

SELECT lo_put(1002, 1, 'abcd');

SELECT lo_unlink(1002);

SELECT lo_export(1001, '/dev/null');

SELECT lo_import('/dev/null');

SELECT lo_import('/dev/null', 2003);

SET lo_compat_privileges = true;

SET SESSION AUTHORIZATION regress_priv_user4;

SELECT has_largeobject_privilege(1002, 'SELECT');

SELECT has_largeobject_privilege(1002, 'UPDATE');

SELECT loread(lo_open(1002, x'40000'::int), 32);

SELECT lowrite(lo_open(1002, x'20000'::int), 'abcd');

SELECT lo_truncate(lo_open(1002, x'20000'::int), 10);

SELECT lo_unlink(1002);

SELECT lo_export(1001, '/dev/null');

SELECT * FROM pg_largeobject LIMIT 0;

SET SESSION AUTHORIZATION regress_priv_user1;

SELECT * FROM pg_largeobject LIMIT 0;

RESET SESSION AUTHORIZATION;

BEGIN;

CREATE OR REPLACE FUNCTION terminate_nothrow(pid int) RETURNS bool
	LANGUAGE plpgsql SECURITY DEFINER SET client_min_messages = error AS $$
BEGIN
	RETURN pg_terminate_backend($1);
EXCEPTION WHEN OTHERS THEN
	RETURN false;
END$$;

ALTER FUNCTION terminate_nothrow OWNER TO pg_signal_backend;

SELECT backend_type FROM pg_stat_activity
WHERE CASE WHEN COALESCE(usesysid, 10) = 10 THEN terminate_nothrow(pid) END;

ROLLBACK;

RESET SESSION AUTHORIZATION;

GRANT pg_database_owner TO regress_priv_user1;

GRANT regress_priv_user1 TO pg_database_owner;

CREATE TABLE datdba_only ();

ALTER TABLE datdba_only OWNER TO pg_database_owner;

REVOKE DELETE ON datdba_only FROM pg_database_owner;

SELECT
	pg_has_role('regress_priv_user1', 'pg_database_owner', 'USAGE') as priv,
	pg_has_role('regress_priv_user1', 'pg_database_owner', 'MEMBER') as mem,
	pg_has_role('regress_priv_user1', 'pg_database_owner',
				'MEMBER WITH ADMIN OPTION') as admin;

BEGIN;

DO $$BEGIN EXECUTE format(
	'ALTER DATABASE %I OWNER TO regress_priv_group2', current_catalog); END$$;

SELECT
	pg_has_role('regress_priv_user1', 'pg_database_owner', 'USAGE') as priv,
	pg_has_role('regress_priv_user1', 'pg_database_owner', 'MEMBER') as mem,
	pg_has_role('regress_priv_user1', 'pg_database_owner',
				'MEMBER WITH ADMIN OPTION') as admin;

SET SESSION AUTHORIZATION regress_priv_user1;

TABLE information_schema.enabled_roles ORDER BY role_name COLLATE "C";

TABLE information_schema.applicable_roles ORDER BY role_name COLLATE "C";

INSERT INTO datdba_only DEFAULT VALUES;

SAVEPOINT q;

DELETE FROM datdba_only;

ROLLBACK TO q;

SET SESSION AUTHORIZATION regress_priv_user2;

TABLE information_schema.enabled_roles;

INSERT INTO datdba_only DEFAULT VALUES;

ROLLBACK;

CREATE SCHEMA testns;

GRANT ALL ON SCHEMA testns TO regress_priv_user1;

CREATE TABLE testns.acltest1 (x int);

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'SELECT');

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'INSERT');

ALTER DEFAULT PRIVILEGES IN SCHEMA testns,testns GRANT SELECT ON TABLES TO public,public;

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'SELECT');

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'INSERT');

DROP TABLE testns.acltest1;

CREATE TABLE testns.acltest1 (x int);

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'SELECT');

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'INSERT');

ALTER DEFAULT PRIVILEGES IN SCHEMA testns GRANT INSERT ON TABLES TO regress_priv_user1;

DROP TABLE testns.acltest1;

CREATE TABLE testns.acltest1 (x int);

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'SELECT');

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'INSERT');

ALTER DEFAULT PRIVILEGES IN SCHEMA testns REVOKE INSERT ON TABLES FROM regress_priv_user1;

DROP TABLE testns.acltest1;

CREATE TABLE testns.acltest1 (x int);

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'SELECT');

SELECT has_table_privilege('regress_priv_user1', 'testns.acltest1', 'INSERT');

ALTER DEFAULT PRIVILEGES FOR ROLE regress_priv_user1 REVOKE EXECUTE ON FUNCTIONS FROM public;

ALTER DEFAULT PRIVILEGES IN SCHEMA testns GRANT USAGE ON SCHEMAS TO regress_priv_user2;

SELECT makeaclitem('regress_priv_user1'::regrole, 'regress_priv_user2'::regrole,
	'SELECT', TRUE);

SELECT makeaclitem('regress_priv_user1'::regrole, 'regress_priv_user2'::regrole,
	'SELECT, INSERT,  UPDATE , DELETE  ', FALSE);

SELECT makeaclitem('regress_priv_user1'::regrole, 'regress_priv_user2'::regrole,
	'SELECT, fake_privilege', FALSE);

CREATE ROLE "regress_""quoted";

SELECT makeaclitem('regress_"quoted'::regrole, 'regress_"quoted'::regrole,
                   'SELECT', TRUE);

SELECT '"regress_""quoted"=r*/"regress_""quoted"'::aclitem;

SELECT '""=r*/""'::aclitem;

DROP ROLE "regress_""quoted";

SELECT pg_input_is_valid('regress_priv_user1=r/regress_priv_user2', 'aclitem');

SELECT pg_input_is_valid('regress_priv_user1=r/', 'aclitem');

SELECT * FROM pg_input_error_info('regress_priv_user1=r/', 'aclitem');

SELECT pg_input_is_valid('regress_priv_user1=r/regress_no_such_user', 'aclitem');

SELECT * FROM pg_input_error_info('regress_priv_user1=r/regress_no_such_user', 'aclitem');

SELECT pg_input_is_valid('regress_priv_user1=rY', 'aclitem');

SELECT * FROM pg_input_error_info('regress_priv_user1=rY', 'aclitem');

BEGIN;

ALTER DEFAULT PRIVILEGES GRANT USAGE ON SCHEMAS TO regress_priv_user2;

CREATE SCHEMA testns2;

SELECT has_schema_privilege('regress_priv_user2', 'testns2', 'USAGE');

SELECT has_schema_privilege('regress_priv_user6', 'testns2', 'USAGE');

SELECT has_schema_privilege('regress_priv_user2', 'testns2', 'CREATE');

ALTER DEFAULT PRIVILEGES REVOKE USAGE ON SCHEMAS FROM regress_priv_user2;

CREATE SCHEMA testns3;

SELECT has_schema_privilege('regress_priv_user2', 'testns3', 'USAGE');

SELECT has_schema_privilege('regress_priv_user2', 'testns3', 'CREATE');

ALTER DEFAULT PRIVILEGES GRANT ALL ON SCHEMAS TO regress_priv_user2;

CREATE SCHEMA testns4;

SELECT has_schema_privilege('regress_priv_user2', 'testns4', 'USAGE');

SELECT has_schema_privilege('regress_priv_user2', 'testns4', 'CREATE');

ALTER DEFAULT PRIVILEGES REVOKE ALL ON SCHEMAS FROM regress_priv_user2;

COMMIT;

BEGIN;

SELECT lo_create(1007);

SELECT has_largeobject_privilege('regress_priv_user2', 1007, 'SELECT');

SELECT has_largeobject_privilege('regress_priv_user2', 1007, 'UPDATE');

SELECT lo_create(1008);

SELECT has_largeobject_privilege('regress_priv_user2', 1008, 'SELECT');

SELECT has_largeobject_privilege('regress_priv_user6', 1008, 'SELECT');

SELECT has_largeobject_privilege('regress_priv_user2', 1008, 'UPDATE');

SELECT lo_create(1009);

SELECT has_largeobject_privilege('regress_priv_user2', 1009, 'SELECT');

SELECT has_largeobject_privilege('regress_priv_user2', 1009, 'UPDATE');

SELECT lo_create(1010);

SELECT has_largeobject_privilege('regress_priv_user2', 1010, 'SELECT');

SELECT has_largeobject_privilege('regress_priv_user2', 1010, 'UPDATE');

ROLLBACK;

BEGIN;

ALTER DEFAULT PRIVILEGES GRANT ALL ON FUNCTIONS TO regress_priv_user2;

ALTER DEFAULT PRIVILEGES GRANT ALL ON SCHEMAS TO regress_priv_user2;

ALTER DEFAULT PRIVILEGES GRANT ALL ON SEQUENCES TO regress_priv_user2;

ALTER DEFAULT PRIVILEGES GRANT ALL ON TABLES TO regress_priv_user2;

ALTER DEFAULT PRIVILEGES GRANT ALL ON TYPES TO regress_priv_user2;

SELECT count(*) FROM pg_shdepend
  WHERE deptype = 'a' AND
        refobjid = 'regress_priv_user2'::regrole AND
	classid = 'pg_default_acl'::regclass;

DROP OWNED BY regress_priv_user2, regress_priv_user2;

SELECT count(*) FROM pg_shdepend
  WHERE deptype = 'a' AND
        refobjid = 'regress_priv_user2'::regrole AND
	classid = 'pg_default_acl'::regclass;

ROLLBACK;

CREATE SCHEMA testns5;

SELECT has_schema_privilege('regress_priv_user2', 'testns5', 'USAGE');

SELECT has_schema_privilege('regress_priv_user2', 'testns5', 'CREATE');

SET ROLE regress_priv_user1;

CREATE FUNCTION testns.foo() RETURNS int AS 'select 1' LANGUAGE sql;

CREATE AGGREGATE testns.agg1(int) (sfunc = int4pl, stype = int4);

CREATE PROCEDURE testns.bar() AS 'select 1' LANGUAGE sql;

SELECT has_function_privilege('regress_priv_user2', 'testns.foo()', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user2', 'testns.agg1(int)', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user2', 'testns.bar()', 'EXECUTE');

ALTER DEFAULT PRIVILEGES IN SCHEMA testns GRANT EXECUTE ON ROUTINES to public;

DROP FUNCTION testns.foo();

CREATE FUNCTION testns.foo() RETURNS int AS 'select 1' LANGUAGE sql;

DROP AGGREGATE testns.agg1(int);

CREATE AGGREGATE testns.agg1(int) (sfunc = int4pl, stype = int4);

DROP PROCEDURE testns.bar();

CREATE PROCEDURE testns.bar() AS 'select 1' LANGUAGE sql;

SELECT has_function_privilege('regress_priv_user2', 'testns.foo()', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user2', 'testns.agg1(int)', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user2', 'testns.bar()', 'EXECUTE');

DROP FUNCTION testns.foo();

DROP AGGREGATE testns.agg1(int);

DROP PROCEDURE testns.bar();

ALTER DEFAULT PRIVILEGES FOR ROLE regress_priv_user1 REVOKE USAGE ON TYPES FROM public;

CREATE DOMAIN testns.priv_testdomain1 AS int;

SELECT has_type_privilege('regress_priv_user2', 'testns.priv_testdomain1', 'USAGE');

ALTER DEFAULT PRIVILEGES IN SCHEMA testns GRANT USAGE ON TYPES to public;

DROP DOMAIN testns.priv_testdomain1;

CREATE DOMAIN testns.priv_testdomain1 AS int;

SELECT has_type_privilege('regress_priv_user2', 'testns.priv_testdomain1', 'USAGE');

DROP DOMAIN testns.priv_testdomain1;

RESET ROLE;

SELECT count(*)
  FROM pg_default_acl d LEFT JOIN pg_namespace n ON defaclnamespace = n.oid
  WHERE nspname = 'testns';

DROP SCHEMA testns CASCADE;

DROP SCHEMA testns2 CASCADE;

DROP SCHEMA testns3 CASCADE;

DROP SCHEMA testns4 CASCADE;

DROP SCHEMA testns5 CASCADE;

SELECT d.*     -- check that entries went away
  FROM pg_default_acl d LEFT JOIN pg_namespace n ON defaclnamespace = n.oid
  WHERE nspname IS NULL AND defaclnamespace != 0;

CREATE SCHEMA testns;

CREATE TABLE testns.t1 (f1 int);

CREATE TABLE testns.t2 (f1 int);

SELECT has_table_privilege('regress_priv_user1', 'testns.t1', 'SELECT');

GRANT ALL ON ALL TABLES IN SCHEMA testns TO regress_priv_user1;

SELECT has_table_privilege('regress_priv_user1', 'testns.t1', 'SELECT');

SELECT has_table_privilege('regress_priv_user1', 'testns.t2', 'SELECT');

REVOKE ALL ON ALL TABLES IN SCHEMA testns FROM regress_priv_user1;

SELECT has_table_privilege('regress_priv_user1', 'testns.t1', 'SELECT');

SELECT has_table_privilege('regress_priv_user1', 'testns.t2', 'SELECT');

CREATE FUNCTION testns.priv_testfunc(int) RETURNS int AS 'select 3 * $1;' LANGUAGE sql;

CREATE AGGREGATE testns.priv_testagg(int) (sfunc = int4pl, stype = int4);

CREATE PROCEDURE testns.priv_testproc(int) AS 'select 3' LANGUAGE sql;

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testfunc(int)', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testagg(int)', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testproc(int)', 'EXECUTE');

REVOKE ALL ON ALL FUNCTIONS IN SCHEMA testns FROM PUBLIC;

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testfunc(int)', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testagg(int)', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testproc(int)', 'EXECUTE');

REVOKE ALL ON ALL PROCEDURES IN SCHEMA testns FROM PUBLIC;

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testproc(int)', 'EXECUTE');

GRANT ALL ON ALL ROUTINES IN SCHEMA testns TO PUBLIC;

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testfunc(int)', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testagg(int)', 'EXECUTE');

SELECT has_function_privilege('regress_priv_user1', 'testns.priv_testproc(int)', 'EXECUTE');

DROP SCHEMA testns CASCADE;

CREATE ROLE regress_schemauser1 superuser login;

CREATE ROLE regress_schemauser2 superuser login;

SET SESSION ROLE regress_schemauser1;

CREATE SCHEMA testns;

SELECT nspname, rolname FROM pg_namespace, pg_roles WHERE pg_namespace.nspname = 'testns' AND pg_namespace.nspowner = pg_roles.oid;

ALTER SCHEMA testns OWNER TO regress_schemauser2;

ALTER ROLE regress_schemauser2 RENAME TO regress_schemauser_renamed;

SELECT nspname, rolname FROM pg_namespace, pg_roles WHERE pg_namespace.nspname = 'testns' AND pg_namespace.nspowner = pg_roles.oid;

set session role regress_schemauser_renamed;

DROP SCHEMA testns CASCADE;

DROP ROLE regress_schemauser1;

DROP ROLE regress_schemauser_renamed;

set session role regress_priv_user1;

create table dep_priv_test (a int);

grant select on dep_priv_test to regress_priv_user2 with grant option;

grant select on dep_priv_test to regress_priv_user3 with grant option;

set session role regress_priv_user2;

grant select on dep_priv_test to regress_priv_user4 with grant option;

set session role regress_priv_user3;

grant select on dep_priv_test to regress_priv_user4 with grant option;

set session role regress_priv_user4;

grant select on dep_priv_test to regress_priv_user5;

set session role regress_priv_user2;

revoke select on dep_priv_test from regress_priv_user4 cascade;

set session role regress_priv_user3;

revoke select on dep_priv_test from regress_priv_user4 cascade;

set session role regress_priv_user1;

drop table dep_priv_test;

drop sequence x_seq;

DROP AGGREGATE priv_testagg1(int);

DROP FUNCTION priv_testfunc2(int);

DROP FUNCTION priv_testfunc4(boolean);

DROP PROCEDURE priv_testproc1(int);

DROP VIEW atestv0;

DROP VIEW atestv1;

DROP VIEW atestv2;

DROP VIEW atestv3 CASCADE;

DROP VIEW atestv4;

DROP TABLE atest1;

DROP TABLE atest2;

DROP TABLE atest3;

DROP TABLE atest4;

DROP TABLE atest5;

DROP TABLE atest6;

DROP TABLE atestc;

DROP TABLE atestp1;

DROP TABLE atestp2;

SELECT lo_unlink(oid) FROM pg_largeobject_metadata WHERE oid >= 1000 AND oid < 3000 ORDER BY oid;

DROP GROUP regress_priv_group1;

DROP GROUP regress_priv_group2;

REVOKE USAGE ON LANGUAGE sql FROM regress_priv_user1;

DROP OWNED BY regress_priv_user1;

DROP USER regress_priv_user1;

DROP USER regress_priv_user2;

DROP USER regress_priv_user3;

DROP USER regress_priv_user4;

DROP USER regress_priv_user5;

DROP USER regress_priv_user6;

DROP USER regress_priv_user7;

DROP USER regress_priv_user8;

ALTER DEFAULT PRIVILEGES FOR ROLE pg_signal_backend
	REVOKE USAGE ON TYPES FROM pg_signal_backend;

ALTER DEFAULT PRIVILEGES FOR ROLE pg_read_all_settings
	REVOKE USAGE ON TYPES FROM pg_read_all_settings;

CREATE USER regress_locktable_user;

CREATE TABLE lock_table (a int);

GRANT SELECT ON lock_table TO regress_locktable_user;

SET SESSION AUTHORIZATION regress_locktable_user;

BEGIN;

LOCK TABLE lock_table IN ACCESS SHARE MODE;

COMMIT;

BEGIN;

LOCK TABLE lock_table IN ROW EXCLUSIVE MODE;

ROLLBACK;

BEGIN;

LOCK TABLE lock_table IN ACCESS EXCLUSIVE MODE;

ROLLBACK;

REVOKE SELECT ON lock_table FROM regress_locktable_user;

GRANT INSERT ON lock_table TO regress_locktable_user;

SET SESSION AUTHORIZATION regress_locktable_user;

BEGIN;

LOCK TABLE lock_table IN ACCESS SHARE MODE;

ROLLBACK;

BEGIN;

LOCK TABLE lock_table IN ROW EXCLUSIVE MODE;

COMMIT;

BEGIN;

LOCK TABLE lock_table IN ACCESS EXCLUSIVE MODE;

ROLLBACK;

REVOKE INSERT ON lock_table FROM regress_locktable_user;

GRANT UPDATE ON lock_table TO regress_locktable_user;

SET SESSION AUTHORIZATION regress_locktable_user;

BEGIN;

LOCK TABLE lock_table IN ACCESS SHARE MODE;

ROLLBACK;

BEGIN;

LOCK TABLE lock_table IN ROW EXCLUSIVE MODE;

COMMIT;

BEGIN;

LOCK TABLE lock_table IN ACCESS EXCLUSIVE MODE;

COMMIT;

REVOKE UPDATE ON lock_table FROM regress_locktable_user;

GRANT DELETE ON lock_table TO regress_locktable_user;

SET SESSION AUTHORIZATION regress_locktable_user;

BEGIN;

LOCK TABLE lock_table IN ACCESS SHARE MODE;

ROLLBACK;

BEGIN;

LOCK TABLE lock_table IN ROW EXCLUSIVE MODE;

COMMIT;

BEGIN;

LOCK TABLE lock_table IN ACCESS EXCLUSIVE MODE;

COMMIT;

REVOKE DELETE ON lock_table FROM regress_locktable_user;

GRANT TRUNCATE ON lock_table TO regress_locktable_user;

SET SESSION AUTHORIZATION regress_locktable_user;

BEGIN;

LOCK TABLE lock_table IN ACCESS SHARE MODE;

ROLLBACK;

BEGIN;

LOCK TABLE lock_table IN ROW EXCLUSIVE MODE;

COMMIT;

BEGIN;

LOCK TABLE lock_table IN ACCESS EXCLUSIVE MODE;

COMMIT;

REVOKE TRUNCATE ON lock_table FROM regress_locktable_user;

GRANT MAINTAIN ON lock_table TO regress_locktable_user;

SET SESSION AUTHORIZATION regress_locktable_user;

BEGIN;

LOCK TABLE lock_table IN ACCESS SHARE MODE;

ROLLBACK;

BEGIN;

LOCK TABLE lock_table IN ROW EXCLUSIVE MODE;

COMMIT;

BEGIN;

LOCK TABLE lock_table IN ACCESS EXCLUSIVE MODE;

COMMIT;

REVOKE MAINTAIN ON lock_table FROM regress_locktable_user;

DROP TABLE lock_table;

DROP USER regress_locktable_user;

CREATE ROLE regress_readallstats;

SELECT has_table_privilege('regress_readallstats','pg_aios','SELECT');

SELECT has_table_privilege('regress_readallstats','pg_backend_memory_contexts','SELECT');

SELECT has_table_privilege('regress_readallstats','pg_shmem_allocations','SELECT');

SELECT has_table_privilege('regress_readallstats','pg_shmem_allocations_numa','SELECT');

SELECT has_table_privilege('regress_readallstats','pg_dsm_registry_allocations','SELECT');

GRANT pg_read_all_stats TO regress_readallstats;

SELECT has_table_privilege('regress_readallstats','pg_aios','SELECT');

SELECT has_table_privilege('regress_readallstats','pg_backend_memory_contexts','SELECT');

SELECT has_table_privilege('regress_readallstats','pg_shmem_allocations','SELECT');

SELECT has_table_privilege('regress_readallstats','pg_shmem_allocations_numa','SELECT');

SELECT has_table_privilege('regress_readallstats','pg_dsm_registry_allocations','SELECT');

SET ROLE regress_readallstats;

SELECT COUNT(*) >= 0 AS ok FROM pg_aios;

SELECT COUNT(*) >= 0 AS ok FROM pg_backend_memory_contexts;

SELECT COUNT(*) >= 0 AS ok FROM pg_shmem_allocations;

RESET ROLE;

DROP ROLE regress_readallstats;

CREATE ROLE regress_group;

CREATE ROLE regress_group_direct_manager;

CREATE ROLE regress_group_indirect_manager;

CREATE ROLE regress_group_member;

GRANT regress_group TO regress_group_direct_manager WITH INHERIT FALSE, ADMIN TRUE;

GRANT regress_group_direct_manager TO regress_group_indirect_manager;

SET SESSION AUTHORIZATION regress_group_direct_manager;

GRANT regress_group TO regress_group_member;

SELECT member::regrole::text, CASE WHEN grantor = 10 THEN 'BOOTSTRAP SUPERUSER' ELSE grantor::regrole::text END FROM pg_auth_members WHERE roleid = 'regress_group'::regrole ORDER BY 1, 2;

REVOKE regress_group FROM regress_group_member;

SET SESSION AUTHORIZATION regress_group_indirect_manager;

GRANT regress_group TO regress_group_member;

SELECT member::regrole::text, CASE WHEN grantor = 10 THEN 'BOOTSTRAP SUPERUSER' ELSE grantor::regrole::text END FROM pg_auth_members WHERE roleid = 'regress_group'::regrole ORDER BY 1, 2;

REVOKE regress_group FROM regress_group_member;

RESET SESSION AUTHORIZATION;

DROP ROLE regress_group;

DROP ROLE regress_group_direct_manager;

DROP ROLE regress_group_indirect_manager;

DROP ROLE regress_group_member;

CREATE ROLE regress_roleoption_protagonist;

CREATE ROLE regress_roleoption_donor;

CREATE ROLE regress_roleoption_recipient;

CREATE SCHEMA regress_roleoption;

GRANT CREATE, USAGE ON SCHEMA regress_roleoption TO PUBLIC;

GRANT regress_roleoption_donor TO regress_roleoption_protagonist WITH INHERIT TRUE, SET FALSE;

GRANT regress_roleoption_recipient TO regress_roleoption_protagonist WITH INHERIT FALSE, SET TRUE;

SET SESSION AUTHORIZATION regress_roleoption_protagonist;

CREATE TABLE regress_roleoption.t1 (a int);

CREATE TABLE regress_roleoption.t2 (a int);

SET SESSION AUTHORIZATION regress_roleoption_donor;

CREATE TABLE regress_roleoption.t3 (a int);

SET SESSION AUTHORIZATION regress_roleoption_recipient;

CREATE TABLE regress_roleoption.t4 (a int);

SET SESSION AUTHORIZATION regress_roleoption_protagonist;

ALTER TABLE regress_roleoption.t1 OWNER TO regress_roleoption_donor;

ALTER TABLE regress_roleoption.t2 OWNER TO regress_roleoption_recipient;

ALTER TABLE regress_roleoption.t3 OWNER TO regress_roleoption_protagonist;

ALTER TABLE regress_roleoption.t4 OWNER TO regress_roleoption_protagonist;

RESET SESSION AUTHORIZATION;

DROP TABLE regress_roleoption.t1;

DROP TABLE regress_roleoption.t2;

DROP TABLE regress_roleoption.t3;

DROP TABLE regress_roleoption.t4;

DROP SCHEMA regress_roleoption;

DROP ROLE regress_roleoption_protagonist;

DROP ROLE regress_roleoption_donor;

DROP ROLE regress_roleoption_recipient;

CREATE ROLE regress_no_maintain;

CREATE ROLE regress_maintain;

CREATE ROLE regress_maintain_all IN ROLE pg_maintain;

CREATE TABLE maintain_test (a INT);

CREATE INDEX ON maintain_test (a);

GRANT MAINTAIN ON maintain_test TO regress_maintain;

CREATE MATERIALIZED VIEW refresh_test AS SELECT 1;

GRANT MAINTAIN ON refresh_test TO regress_maintain;

CREATE SCHEMA reindex_test;

SET ROLE regress_no_maintain;

VACUUM maintain_test;

ANALYZE maintain_test;

VACUUM (ANALYZE) maintain_test;

CLUSTER maintain_test USING maintain_test_a_idx;

REFRESH MATERIALIZED VIEW refresh_test;

REINDEX TABLE maintain_test;

REINDEX INDEX maintain_test_a_idx;

REINDEX SCHEMA reindex_test;

RESET ROLE;

SET ROLE regress_maintain;

VACUUM maintain_test;

ANALYZE maintain_test;

VACUUM (ANALYZE) maintain_test;

CLUSTER maintain_test USING maintain_test_a_idx;

REFRESH MATERIALIZED VIEW refresh_test;

REINDEX TABLE maintain_test;

REINDEX INDEX maintain_test_a_idx;

REINDEX SCHEMA reindex_test;

RESET ROLE;

SET ROLE regress_maintain_all;

VACUUM maintain_test;

ANALYZE maintain_test;

VACUUM (ANALYZE) maintain_test;

CLUSTER maintain_test USING maintain_test_a_idx;

REFRESH MATERIALIZED VIEW refresh_test;

REINDEX TABLE maintain_test;

REINDEX INDEX maintain_test_a_idx;

REINDEX SCHEMA reindex_test;

RESET ROLE;

DROP TABLE maintain_test;

DROP MATERIALIZED VIEW refresh_test;

DROP SCHEMA reindex_test;

DROP ROLE regress_no_maintain;

DROP ROLE regress_maintain;

DROP ROLE regress_maintain_all;

CREATE ROLE regress_grantor1;

CREATE ROLE regress_grantor2 ROLE regress_grantor1;

CREATE ROLE regress_grantor3;

CREATE TABLE grantor_test1 ();

CREATE TABLE grantor_test2 ();

CREATE TABLE grantor_test3 ();

GRANT SELECT ON grantor_test2 TO regress_grantor1 WITH GRANT OPTION;

GRANT SELECT, UPDATE ON grantor_test3 TO regress_grantor2 WITH GRANT OPTION;

SET ROLE regress_grantor1;

GRANT SELECT, UPDATE ON grantor_test1 TO regress_grantor3;

GRANT SELECT, UPDATE ON grantor_test2 TO regress_grantor3;

GRANT SELECT, UPDATE ON grantor_test3 TO regress_grantor3;

RESET ROLE;

SELECT * FROM information_schema.table_privileges t
	WHERE grantor LIKE 'regress_grantor%' ORDER BY ROW(t.*);

DROP TABLE grantor_test1, grantor_test2, grantor_test3;

DROP ROLE regress_grantor1, regress_grantor2, regress_grantor3;
