CREATE ROLE regress_publication_user LOGIN SUPERUSER;

CREATE ROLE regress_publication_user2;

CREATE ROLE regress_publication_user_dummy LOGIN NOSUPERUSER;

SET SESSION AUTHORIZATION 'regress_publication_user';

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_default;

RESET client_min_messages;

COMMENT ON PUBLICATION testpub_default IS 'test publication';

SELECT obj_description(p.oid, 'pg_publication') FROM pg_publication p;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_ins_trunct WITH (publish = insert);

RESET client_min_messages;

ALTER PUBLICATION testpub_default SET (publish = update);

CREATE PUBLICATION testpub_xxx WITH (foo);

CREATE PUBLICATION testpub_xxx WITH (publish = 'cluster, vacuum');

CREATE PUBLICATION testpub_xxx WITH (publish_via_partition_root = 'true', publish_via_partition_root = '0');

CREATE PUBLICATION testpub_xxx WITH (publish_generated_columns = stored, publish_generated_columns = none);

CREATE PUBLICATION testpub_xxx WITH (publish_generated_columns = foo);

CREATE PUBLICATION testpub_xxx WITH (publish_generated_columns);

ALTER PUBLICATION testpub_default SET (publish = 'insert, update, delete');

CREATE SCHEMA pub_test;

CREATE TABLE testpub_tbl1 (id serial primary key, data text);

CREATE TABLE pub_test.testpub_nopk (foo int, bar int);

CREATE VIEW testpub_view AS SELECT 1;

CREATE TABLE testpub_parted (a int) PARTITION BY LIST (a);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_foralltables FOR ALL TABLES WITH (publish = 'insert');

RESET client_min_messages;

ALTER PUBLICATION testpub_foralltables SET (publish = 'insert, update');

CREATE TABLE testpub_tbl2 (id serial primary key, data text);

ALTER PUBLICATION testpub_foralltables ADD TABLE testpub_tbl2;

ALTER PUBLICATION testpub_foralltables DROP TABLE testpub_tbl2;

ALTER PUBLICATION testpub_foralltables SET TABLE pub_test.testpub_nopk;

ALTER PUBLICATION testpub_foralltables ADD TABLES IN SCHEMA pub_test;

ALTER PUBLICATION testpub_foralltables DROP TABLES IN SCHEMA pub_test;

ALTER PUBLICATION testpub_foralltables SET TABLES IN SCHEMA pub_test;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_fortable FOR TABLE testpub_tbl1;

RESET client_min_messages;

ALTER PUBLICATION testpub_fortable ADD TABLES IN SCHEMA pub_test;

ALTER PUBLICATION testpub_fortable DROP TABLES IN SCHEMA pub_test;

ALTER PUBLICATION testpub_fortable SET TABLES IN SCHEMA pub_test;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_forschema FOR TABLES IN SCHEMA pub_test;

CREATE PUBLICATION testpub_for_tbl_schema FOR TABLES IN SCHEMA pub_test, TABLE pub_test.testpub_nopk;

RESET client_min_messages;

ALTER PUBLICATION testpub_forschema ADD TABLE pub_test.testpub_nopk;

ALTER PUBLICATION testpub_forschema DROP TABLE pub_test.testpub_nopk;

ALTER PUBLICATION testpub_forschema DROP TABLE pub_test.testpub_nopk;

ALTER PUBLICATION testpub_forschema SET TABLE pub_test.testpub_nopk;

SELECT pubname, puballtables FROM pg_publication WHERE pubname = 'testpub_foralltables';

DROP TABLE testpub_tbl2;

DROP PUBLICATION testpub_foralltables, testpub_fortable, testpub_forschema, testpub_for_tbl_schema;

CREATE TABLE testpub_tbl3 (a int);

CREATE TABLE testpub_tbl3a (b text) INHERITS (testpub_tbl3);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub3 FOR TABLE testpub_tbl3;

CREATE PUBLICATION testpub4 FOR TABLE ONLY testpub_tbl3;

RESET client_min_messages;

DROP TABLE testpub_tbl3, testpub_tbl3a;

DROP PUBLICATION testpub3, testpub4;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_forparted;

CREATE PUBLICATION testpub_forparted1;

RESET client_min_messages;

CREATE TABLE testpub_parted1 (LIKE testpub_parted);

CREATE TABLE testpub_parted2 (LIKE testpub_parted);

ALTER PUBLICATION testpub_forparted1 SET (publish='insert');

ALTER TABLE testpub_parted ATTACH PARTITION testpub_parted1 FOR VALUES IN (1);

ALTER TABLE testpub_parted ATTACH PARTITION testpub_parted2 FOR VALUES IN (2);

UPDATE testpub_parted1 SET a = 1;

ALTER PUBLICATION testpub_forparted ADD TABLE testpub_parted;

UPDATE testpub_parted SET a = 1 WHERE false;

UPDATE testpub_parted1 SET a = 1;

ALTER TABLE testpub_parted DETACH PARTITION testpub_parted1;

UPDATE testpub_parted1 SET a = 1;

ALTER PUBLICATION testpub_forparted SET (publish_via_partition_root = true);

UPDATE testpub_parted2 SET a = 2;

ALTER PUBLICATION testpub_forparted DROP TABLE testpub_parted;

UPDATE testpub_parted2 SET a = 2;

DROP TABLE testpub_parted1, testpub_parted2;

DROP PUBLICATION testpub_forparted, testpub_forparted1;

CREATE TABLE testpub_rf_tbl1 (a integer, b text);

CREATE TABLE testpub_rf_tbl2 (c text, d integer);

CREATE TABLE testpub_rf_tbl3 (e integer);

CREATE TABLE testpub_rf_tbl4 (g text);

CREATE TABLE testpub_rf_tbl5 (a xml);

CREATE SCHEMA testpub_rf_schema1;

CREATE TABLE testpub_rf_schema1.testpub_rf_tbl5 (h integer);

CREATE SCHEMA testpub_rf_schema2;

CREATE TABLE testpub_rf_schema2.testpub_rf_tbl6 (i integer);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub5 FOR TABLE testpub_rf_tbl1, testpub_rf_tbl2 WHERE (c <> 'test' AND d < 5) WITH (publish = 'insert');

RESET client_min_messages;

ALTER PUBLICATION testpub5 ADD TABLE testpub_rf_tbl3 WHERE (e > 1000 AND e < 2000);

ALTER PUBLICATION testpub5 DROP TABLE testpub_rf_tbl2;

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl3 WHERE (e > 300 AND e < 500);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_rf_yes FOR TABLE testpub_rf_tbl1 WHERE (a > 1) WITH (publish = 'insert');

CREATE PUBLICATION testpub_rf_no FOR TABLE testpub_rf_tbl1;

RESET client_min_messages;

DROP PUBLICATION testpub_rf_yes, testpub_rf_no;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_syntax1 FOR TABLE testpub_rf_tbl1, ONLY testpub_rf_tbl3 WHERE (e < 999) WITH (publish = 'insert');

RESET client_min_messages;

DROP PUBLICATION testpub_syntax1;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_syntax2 FOR TABLE testpub_rf_tbl1, testpub_rf_schema1.testpub_rf_tbl5 WHERE (h < 999) WITH (publish = 'insert');

RESET client_min_messages;

DROP PUBLICATION testpub_syntax2;

SET client_min_messages = 'ERROR';

RESET client_min_messages;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_dups FOR TABLE testpub_rf_tbl1 WHERE (a = 1), testpub_rf_tbl1 WITH (publish = 'insert');

CREATE PUBLICATION testpub_dups FOR TABLE testpub_rf_tbl1, testpub_rf_tbl1 WHERE (a = 2) WITH (publish = 'insert');

RESET client_min_messages;

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl3 WHERE (1234);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl3 WHERE (e < AVG(e));

CREATE FUNCTION testpub_rf_func1(integer, integer) RETURNS boolean AS $$ SELECT hashint4($1) > $2 $$ LANGUAGE SQL;

CREATE OPERATOR =#> (PROCEDURE = testpub_rf_func1, LEFTARG = integer, RIGHTARG = integer);

CREATE PUBLICATION testpub6 FOR TABLE testpub_rf_tbl3 WHERE (e =#> 27);

CREATE FUNCTION testpub_rf_func2() RETURNS integer IMMUTABLE AS $$ BEGIN RETURN 123; END; $$ LANGUAGE plpgsql;

ALTER PUBLICATION testpub5 ADD TABLE testpub_rf_tbl1 WHERE (a >= testpub_rf_func2());

ALTER PUBLICATION testpub5 ADD TABLE testpub_rf_tbl1 WHERE (a < random());

CREATE COLLATION user_collation FROM "C";

ALTER PUBLICATION testpub5 ADD TABLE testpub_rf_tbl1 WHERE (b < '2' COLLATE user_collation);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (NULLIF(1,2) = a);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (a IS NULL);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE ((a > 5) IS FALSE);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (a IS DISTINCT FROM 5);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE ((a, a + 1) < (2, 3));

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (b::varchar < '2');

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl4 WHERE (length(g) < 6);

CREATE TYPE rf_bug_status AS ENUM ('new', 'open', 'closed');

CREATE TABLE rf_bug (id serial, description text, status rf_bug_status);

CREATE PUBLICATION testpub6 FOR TABLE rf_bug WHERE (status = 'open') WITH (publish = 'insert');

DROP TABLE rf_bug;

DROP TYPE rf_bug_status;

CREATE PUBLICATION testpub6 FOR TABLE testpub_rf_tbl1 WHERE (a IN (SELECT generate_series(1,5)));

CREATE PUBLICATION testpub6 FOR TABLE testpub_rf_tbl1 WHERE ('(0,1)'::tid = ctid);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl5 WHERE (a IS DOCUMENT);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl5 WHERE (xmlexists('//foo[text() = ''bar'']' PASSING BY VALUE a));

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (NULLIF(1, 2) = a);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (CASE a WHEN 5 THEN true ELSE false END);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (COALESCE(b, 'foo') = 'foo');

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (GREATEST(a, 10) > 10);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (a IN (2, 4, 6));

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (ARRAY[a] <@ ARRAY[2, 4, 6]);

ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (ROW(a, 2) IS NULL);

ALTER PUBLICATION testpub5 DROP TABLE testpub_rf_tbl1 WHERE (e < 27);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub6 FOR TABLES IN SCHEMA testpub_rf_schema2;

ALTER PUBLICATION testpub6 SET TABLES IN SCHEMA testpub_rf_schema2, TABLE testpub_rf_schema2.testpub_rf_tbl6 WHERE (i < 99);

RESET client_min_messages;

CREATE PUBLICATION testpub7 FOR TABLE testpub_rf_tbl6 WHERE (y > 100);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub8 FOR TABLE testpub_rf_tbl7 WHERE (y > 100);

ALTER TABLE testpub_rf_tbl7 ALTER COLUMN y SET EXPRESSION AS (x * testpub_rf_func2());

RESET client_min_messages;

DROP TABLE testpub_rf_tbl1;

DROP TABLE testpub_rf_tbl2;

DROP TABLE testpub_rf_tbl3;

DROP TABLE testpub_rf_tbl4;

DROP TABLE testpub_rf_tbl5;

DROP TABLE testpub_rf_schema1.testpub_rf_tbl5;

DROP TABLE testpub_rf_schema2.testpub_rf_tbl6;

DROP SCHEMA testpub_rf_schema1;

DROP SCHEMA testpub_rf_schema2;

DROP PUBLICATION testpub5;

DROP PUBLICATION testpub6;

DROP PUBLICATION testpub8;

DROP TABLE testpub_rf_tbl7;

DROP OPERATOR =#>(integer, integer);

DROP FUNCTION testpub_rf_func1(integer, integer);

DROP FUNCTION testpub_rf_func2();

DROP COLLATION user_collation;

CREATE TABLE rf_tbl_abcd_nopk(a int, b int, c int, d int);

CREATE TABLE rf_tbl_abcd_pk(a int, b int, c int, d int, PRIMARY KEY(a,b));

CREATE TABLE rf_tbl_abcd_part_pk (a int PRIMARY KEY, b int) PARTITION by RANGE (a);

CREATE TABLE rf_tbl_abcd_part_pk_1 (b int, a int PRIMARY KEY);

ALTER TABLE rf_tbl_abcd_part_pk ATTACH PARTITION rf_tbl_abcd_part_pk_1 FOR VALUES FROM (1) TO (10);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub6 FOR TABLE rf_tbl_abcd_pk WHERE (a > 99);

RESET client_min_messages;

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk WHERE (b > 99);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk WHERE (c > 99);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk WHERE (d > 99);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk WHERE (a > 99);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER TABLE rf_tbl_abcd_pk REPLICA IDENTITY FULL;

ALTER TABLE rf_tbl_abcd_nopk REPLICA IDENTITY FULL;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk WHERE (c > 99);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk WHERE (a > 99);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER TABLE rf_tbl_abcd_pk REPLICA IDENTITY NOTHING;

ALTER TABLE rf_tbl_abcd_nopk REPLICA IDENTITY NOTHING;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk WHERE (a > 99);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk WHERE (c > 99);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk WHERE (a > 99);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER TABLE rf_tbl_abcd_pk ALTER COLUMN c SET NOT NULL;

CREATE UNIQUE INDEX idx_abcd_pk_c ON rf_tbl_abcd_pk(c);

ALTER TABLE rf_tbl_abcd_pk REPLICA IDENTITY USING INDEX idx_abcd_pk_c;

ALTER TABLE rf_tbl_abcd_nopk ALTER COLUMN c SET NOT NULL;

CREATE UNIQUE INDEX idx_abcd_nopk_c ON rf_tbl_abcd_nopk(c);

ALTER TABLE rf_tbl_abcd_nopk REPLICA IDENTITY USING INDEX idx_abcd_nopk_c;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk WHERE (a > 99);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk WHERE (c > 99);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk WHERE (a > 99);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk WHERE (c > 99);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=0);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk WHERE (a > 99);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk_1 WHERE (a > 99);

UPDATE rf_tbl_abcd_part_pk SET a = 1;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=1);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk WHERE (a > 99);

UPDATE rf_tbl_abcd_part_pk SET a = 1;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=0);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=0);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk_1 WHERE (b > 99);

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=0);

UPDATE rf_tbl_abcd_part_pk SET a = 1;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=1);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk WHERE (b > 99);

UPDATE rf_tbl_abcd_part_pk SET a = 1;

DROP PUBLICATION testpub6;

DROP TABLE rf_tbl_abcd_pk;

DROP TABLE rf_tbl_abcd_nopk;

DROP TABLE rf_tbl_abcd_part_pk;

SET client_min_messages = 'ERROR';

CREATE TABLE testpub_gencol (a INT, b INT GENERATED ALWAYS AS (a + 1) STORED NOT NULL);

CREATE UNIQUE INDEX testpub_gencol_idx ON testpub_gencol (b);

ALTER TABLE testpub_gencol REPLICA IDENTITY USING index testpub_gencol_idx;

CREATE PUBLICATION pub_gencol FOR TABLE testpub_gencol;

UPDATE testpub_gencol SET a = 100 WHERE a = 1;

ALTER TABLE testpub_gencol REPLICA IDENTITY FULL;

UPDATE testpub_gencol SET a = 100 WHERE a = 1;

DROP PUBLICATION pub_gencol;

CREATE PUBLICATION pub_gencol FOR TABLE testpub_gencol with (publish_generated_columns = stored);

UPDATE testpub_gencol SET a = 100 WHERE a = 1;

DROP PUBLICATION pub_gencol;

DROP TABLE testpub_gencol;

CREATE PUBLICATION pub_gencol FOR TABLE testpub_gencol;

ALTER TABLE testpub_gencol REPLICA IDENTITY FULL;

UPDATE testpub_gencol SET a = 100 WHERE a = 1;

DROP PUBLICATION pub_gencol;

CREATE PUBLICATION pub_gencol FOR TABLE testpub_gencol with (publish_generated_columns = stored);

UPDATE testpub_gencol SET a = 100 WHERE a = 1;

DROP PUBLICATION pub_gencol;

DROP TABLE testpub_gencol;

RESET client_min_messages;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_dups FOR TABLE testpub_tbl1 (a), testpub_tbl1 WITH (publish = 'insert');

CREATE PUBLICATION testpub_dups FOR TABLE testpub_tbl1, testpub_tbl1 (a) WITH (publish = 'insert');

RESET client_min_messages;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_fortable FOR TABLE testpub_tbl1;

CREATE PUBLICATION testpub_fortable_insert WITH (publish = 'insert');

RESET client_min_messages;

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl5 (a, x);

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl5 (b, c);

UPDATE testpub_tbl5 SET a = 1;

ALTER PUBLICATION testpub_fortable DROP TABLE testpub_tbl5;

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl5 (a, ctid);

ALTER PUBLICATION testpub_fortable SET TABLE testpub_tbl1 (id, ctid);

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl5 (a, a);

ALTER PUBLICATION testpub_fortable SET TABLE testpub_tbl5 (a, a);

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl5 (a, c);

ALTER TABLE testpub_tbl5 DROP COLUMN c;

ALTER PUBLICATION testpub_fortable_insert ADD TABLE testpub_tbl5 (b, c);

CREATE UNIQUE INDEX testpub_tbl5_b_key ON testpub_tbl5 (b, c);

ALTER TABLE testpub_tbl5 ALTER b SET NOT NULL, ALTER c SET NOT NULL;

ALTER TABLE testpub_tbl5 REPLICA IDENTITY USING INDEX testpub_tbl5_b_key;

UPDATE testpub_tbl5 SET a = 1;

ALTER PUBLICATION testpub_fortable DROP TABLE testpub_tbl5;

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl5 (a, d);

ALTER PUBLICATION testpub_fortable DROP TABLE testpub_tbl5;

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl5 (a, e);

ALTER TABLE testpub_tbl5 REPLICA IDENTITY USING INDEX testpub_tbl5_b_key;

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl5 (a, c);

UPDATE testpub_tbl5 SET a = 1;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_table_ins WITH (publish = 'insert, truncate');

RESET client_min_messages;

ALTER PUBLICATION testpub_table_ins ADD TABLE testpub_tbl5 (a);

CREATE TABLE testpub_tbl5d (a int PRIMARY KEY DEFERRABLE);

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl5d;

UPDATE testpub_tbl5d SET a = 1;

ALTER TABLE testpub_tbl5d REPLICA IDENTITY FULL;

UPDATE testpub_tbl5d SET a = 1;

DROP TABLE testpub_tbl5d;

CREATE TABLE testpub_tbl6 (a int, b text, c text);

ALTER TABLE testpub_tbl6 REPLICA IDENTITY FULL;

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl6 (a, b, c);

UPDATE testpub_tbl6 SET a = 1;

ALTER PUBLICATION testpub_fortable DROP TABLE testpub_tbl6;

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl6;

UPDATE testpub_tbl6 SET a = 1;

CREATE TABLE testpub_tbl7 (a int primary key, b text, c text);

ALTER PUBLICATION testpub_fortable ADD TABLE testpub_tbl7 (a, b);

ALTER PUBLICATION testpub_fortable SET TABLE testpub_tbl7 (a, b);

ALTER PUBLICATION testpub_fortable SET TABLE testpub_tbl7 (a, c);

CREATE TABLE testpub_tbl8 (a int, b text, c text) PARTITION BY HASH (a);

CREATE TABLE testpub_tbl8_0 PARTITION OF testpub_tbl8 FOR VALUES WITH (modulus 2, remainder 0);

ALTER TABLE testpub_tbl8_0 ADD PRIMARY KEY (a);

ALTER TABLE testpub_tbl8_0 REPLICA IDENTITY USING INDEX testpub_tbl8_0_pkey;

CREATE TABLE testpub_tbl8_1 PARTITION OF testpub_tbl8 FOR VALUES WITH (modulus 2, remainder 1);

ALTER TABLE testpub_tbl8_1 ADD PRIMARY KEY (b);

ALTER TABLE testpub_tbl8_1 REPLICA IDENTITY USING INDEX testpub_tbl8_1_pkey;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_col_list FOR TABLE testpub_tbl8 (a, b) WITH (publish_via_partition_root = 'true');

RESET client_min_messages;

ALTER PUBLICATION testpub_col_list DROP TABLE testpub_tbl8;

ALTER PUBLICATION testpub_col_list ADD TABLE testpub_tbl8 (a, b);

UPDATE testpub_tbl8 SET a = 1;

ALTER PUBLICATION testpub_col_list DROP TABLE testpub_tbl8;

ALTER PUBLICATION testpub_col_list ADD TABLE testpub_tbl8 (a, c);

UPDATE testpub_tbl8 SET a = 1;

ALTER PUBLICATION testpub_col_list DROP TABLE testpub_tbl8;

ALTER TABLE testpub_tbl8_1 REPLICA IDENTITY FULL;

ALTER PUBLICATION testpub_col_list ADD TABLE testpub_tbl8 (a, c);

UPDATE testpub_tbl8 SET a = 1;

ALTER PUBLICATION testpub_col_list DROP TABLE testpub_tbl8;

ALTER TABLE testpub_tbl8_1 REPLICA IDENTITY USING INDEX testpub_tbl8_1_pkey;

ALTER PUBLICATION testpub_col_list ADD TABLE testpub_tbl8 (a, b);

ALTER TABLE testpub_tbl8_1 REPLICA IDENTITY FULL;

UPDATE testpub_tbl8 SET a = 1;

ALTER TABLE testpub_tbl8_1 DROP CONSTRAINT testpub_tbl8_1_pkey;

ALTER TABLE testpub_tbl8_1 ADD PRIMARY KEY (c);

ALTER TABLE testpub_tbl8_1 REPLICA IDENTITY USING INDEX testpub_tbl8_1_pkey;

UPDATE testpub_tbl8 SET a = 1;

DROP TABLE testpub_tbl8;

CREATE TABLE testpub_tbl8 (a int, b text, c text) PARTITION BY HASH (a);

ALTER PUBLICATION testpub_col_list ADD TABLE testpub_tbl8 (a, b);

CREATE TABLE testpub_tbl8_0 (a int, b text, c text);

ALTER TABLE testpub_tbl8_0 ADD PRIMARY KEY (a);

ALTER TABLE testpub_tbl8_0 REPLICA IDENTITY USING INDEX testpub_tbl8_0_pkey;

CREATE TABLE testpub_tbl8_1 (a int, b text, c text);

ALTER TABLE testpub_tbl8_1 ADD PRIMARY KEY (c);

ALTER TABLE testpub_tbl8_1 REPLICA IDENTITY USING INDEX testpub_tbl8_1_pkey;

UPDATE testpub_tbl8 SET a = 1;

ALTER TABLE testpub_tbl8_0 REPLICA IDENTITY FULL;

UPDATE testpub_tbl8 SET a = 1;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_tbl9 FOR TABLES IN SCHEMA public, TABLE public.testpub_tbl7(a);

CREATE PUBLICATION testpub_tbl9 FOR TABLES IN SCHEMA public;

ALTER PUBLICATION testpub_tbl9 ADD TABLE public.testpub_tbl7(a);

ALTER PUBLICATION testpub_tbl9 SET TABLE public.testpub_tbl7(a);

ALTER PUBLICATION testpub_tbl9 ADD TABLES IN SCHEMA public;

ALTER PUBLICATION testpub_tbl9 SET TABLES IN SCHEMA public, TABLE public.testpub_tbl7(a);

ALTER PUBLICATION testpub_tbl9 DROP TABLE public.testpub_tbl7;

ALTER PUBLICATION testpub_tbl9 ADD TABLES IN SCHEMA public, TABLE public.testpub_tbl7(a);

RESET client_min_messages;

DROP TABLE testpub_tbl5, testpub_tbl6, testpub_tbl7, testpub_tbl8, testpub_tbl8_1;

DROP PUBLICATION testpub_table_ins, testpub_fortable, testpub_fortable_insert, testpub_col_list, testpub_tbl9;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_both_filters;

RESET client_min_messages;

CREATE TABLE testpub_tbl_both_filters (a int, b int, c int, PRIMARY KEY (a,c));

ALTER TABLE testpub_tbl_both_filters REPLICA IDENTITY USING INDEX testpub_tbl_both_filters_pkey;

ALTER PUBLICATION testpub_both_filters ADD TABLE testpub_tbl_both_filters (a,c) WHERE (c != 1);

DROP TABLE testpub_tbl_both_filters;

DROP PUBLICATION testpub_both_filters;

CREATE TABLE rf_tbl_abcd_nopk(a int, b int, c int, d int);

CREATE TABLE rf_tbl_abcd_pk(a int, b int, c int, d int, PRIMARY KEY(a,b));

CREATE TABLE rf_tbl_abcd_part_pk (a int PRIMARY KEY, b int) PARTITION by RANGE (a);

CREATE TABLE rf_tbl_abcd_part_pk_1 (b int, a int PRIMARY KEY);

ALTER TABLE rf_tbl_abcd_part_pk ATTACH PARTITION rf_tbl_abcd_part_pk_1 FOR VALUES FROM (1) TO (10);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub6 FOR TABLE rf_tbl_abcd_pk (a, b);

RESET client_min_messages;

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk (a, b, c);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk (a);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk (b);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk (a);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER TABLE rf_tbl_abcd_pk REPLICA IDENTITY FULL;

ALTER TABLE rf_tbl_abcd_nopk REPLICA IDENTITY FULL;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk (c);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk (a, b, c, d);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER TABLE rf_tbl_abcd_pk REPLICA IDENTITY NOTHING;

ALTER TABLE rf_tbl_abcd_nopk REPLICA IDENTITY NOTHING;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk (a);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk (a, b, c, d);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk (d);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER TABLE rf_tbl_abcd_pk ALTER COLUMN c SET NOT NULL;

CREATE UNIQUE INDEX idx_abcd_pk_c ON rf_tbl_abcd_pk(c);

ALTER TABLE rf_tbl_abcd_pk REPLICA IDENTITY USING INDEX idx_abcd_pk_c;

ALTER TABLE rf_tbl_abcd_nopk ALTER COLUMN c SET NOT NULL;

CREATE UNIQUE INDEX idx_abcd_nopk_c ON rf_tbl_abcd_nopk(c);

ALTER TABLE rf_tbl_abcd_nopk REPLICA IDENTITY USING INDEX idx_abcd_nopk_c;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk (a);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_pk (c);

UPDATE rf_tbl_abcd_pk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk (a);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_nopk (c);

UPDATE rf_tbl_abcd_nopk SET a = 1;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=0);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk (a);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk_1 (a);

UPDATE rf_tbl_abcd_part_pk SET a = 1;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=1);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk (a);

UPDATE rf_tbl_abcd_part_pk SET a = 1;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=0);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=0);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk_1 (b);

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=0);

UPDATE rf_tbl_abcd_part_pk SET a = 1;

ALTER PUBLICATION testpub6 SET (PUBLISH_VIA_PARTITION_ROOT=1);

ALTER PUBLICATION testpub6 SET TABLE rf_tbl_abcd_part_pk (b);

UPDATE rf_tbl_abcd_part_pk SET a = 1;

DROP PUBLICATION testpub6;

DROP TABLE rf_tbl_abcd_pk;

DROP TABLE rf_tbl_abcd_nopk;

DROP TABLE rf_tbl_abcd_part_pk;

SET client_min_messages = 'ERROR';

CREATE TABLE testpub_tbl4(a int);

INSERT INTO testpub_tbl4 values(1);

UPDATE testpub_tbl4 set a = 2;

CREATE PUBLICATION testpub_foralltables FOR ALL TABLES;

RESET client_min_messages;

UPDATE testpub_tbl4 set a = 3;

DROP PUBLICATION testpub_foralltables;

UPDATE testpub_tbl4 set a = 3;

DROP TABLE testpub_tbl4;

CREATE PUBLICATION testpub_fortbl FOR TABLE testpub_view;

CREATE TEMPORARY TABLE testpub_temptbl(a int);

CREATE PUBLICATION testpub_fortemptbl FOR TABLE testpub_temptbl;

DROP TABLE testpub_temptbl;

CREATE UNLOGGED TABLE testpub_unloggedtbl(a int);

CREATE PUBLICATION testpub_forunloggedtbl FOR TABLE testpub_unloggedtbl;

DROP TABLE testpub_unloggedtbl;

CREATE PUBLICATION testpub_forsystemtbl FOR TABLE pg_publication;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_fortbl FOR TABLE testpub_tbl1, pub_test.testpub_nopk;

RESET client_min_messages;

ALTER PUBLICATION testpub_fortbl ADD TABLE testpub_tbl1;

CREATE PUBLICATION testpub_fortbl FOR TABLE testpub_tbl1;

ALTER PUBLICATION testpub_default ADD TABLE testpub_view;

ALTER PUBLICATION testpub_default ADD TABLE testpub_tbl1;

ALTER PUBLICATION testpub_default SET TABLE testpub_tbl1;

ALTER PUBLICATION testpub_default ADD TABLE pub_test.testpub_nopk;

ALTER PUBLICATION testpub_ins_trunct ADD TABLE pub_test.testpub_nopk, testpub_tbl1;

ALTER PUBLICATION testpub_default DROP TABLE testpub_tbl1, pub_test.testpub_nopk;

ALTER PUBLICATION testpub_default DROP TABLE pub_test.testpub_nopk;

CREATE TABLE pub_test.testpub_addpk (id int not null, data int);

ALTER PUBLICATION testpub_default ADD TABLE pub_test.testpub_addpk;

INSERT INTO pub_test.testpub_addpk VALUES(1, 11);

CREATE UNIQUE INDEX testpub_addpk_id_idx ON pub_test.testpub_addpk(id);

UPDATE pub_test.testpub_addpk SET id = 2;

ALTER TABLE pub_test.testpub_addpk ADD PRIMARY KEY USING INDEX testpub_addpk_id_idx;

UPDATE pub_test.testpub_addpk SET id = 2;

DROP TABLE pub_test.testpub_addpk;

SET ROLE regress_publication_user2;

CREATE PUBLICATION testpub2;

SET ROLE regress_publication_user;

GRANT CREATE ON DATABASE regression TO regress_publication_user2;

SET ROLE regress_publication_user2;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub2;

CREATE PUBLICATION testpub3 FOR TABLES IN SCHEMA pub_test;

CREATE PUBLICATION testpub3;

RESET client_min_messages;

ALTER PUBLICATION testpub2 ADD TABLE testpub_tbl1;

ALTER PUBLICATION testpub3 ADD TABLES IN SCHEMA pub_test;

SET ROLE regress_publication_user;

GRANT regress_publication_user TO regress_publication_user2;

SET ROLE regress_publication_user2;

ALTER PUBLICATION testpub2 ADD TABLE testpub_tbl1;

DROP PUBLICATION testpub2;

DROP PUBLICATION testpub3;

SET ROLE regress_publication_user;

CREATE ROLE regress_publication_user3;

GRANT regress_publication_user2 TO regress_publication_user3;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub4 FOR TABLES IN SCHEMA pub_test;

RESET client_min_messages;

ALTER PUBLICATION testpub4 OWNER TO regress_publication_user3;

SET ROLE regress_publication_user3;

ALTER PUBLICATION testpub4 owner to regress_publication_user2;

ALTER PUBLICATION testpub4 owner to regress_publication_user;

SET ROLE regress_publication_user;

DROP PUBLICATION testpub4;

DROP ROLE regress_publication_user3;

REVOKE CREATE ON DATABASE regression FROM regress_publication_user2;

DROP TABLE testpub_parted;

DROP TABLE testpub_tbl1;

SET ROLE regress_publication_user_dummy;

ALTER PUBLICATION testpub_default RENAME TO testpub_dummy;

RESET ROLE;

ALTER PUBLICATION testpub_default RENAME TO testpub_foo;

ALTER PUBLICATION testpub_foo RENAME TO testpub_default;

ALTER PUBLICATION testpub_default OWNER TO regress_publication_user2;

CREATE SCHEMA pub_test1;

CREATE SCHEMA pub_test2;

CREATE SCHEMA pub_test3;

CREATE SCHEMA "CURRENT_SCHEMA";

CREATE TABLE pub_test1.tbl (id int, data text);

CREATE TABLE pub_test1.tbl1 (id serial primary key, data text);

CREATE TABLE pub_test2.tbl1 (id serial primary key, data text);

CREATE TABLE "CURRENT_SCHEMA"."CURRENT_SCHEMA"(id int);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub1_forschema FOR TABLES IN SCHEMA pub_test1;

CREATE PUBLICATION testpub2_forschema FOR TABLES IN SCHEMA pub_test1, pub_test2, pub_test3;

CREATE PUBLICATION testpub3_forschema FOR TABLES IN SCHEMA CURRENT_SCHEMA;

CREATE PUBLICATION testpub4_forschema FOR TABLES IN SCHEMA "CURRENT_SCHEMA";

CREATE PUBLICATION testpub5_forschema FOR TABLES IN SCHEMA CURRENT_SCHEMA, "CURRENT_SCHEMA";

CREATE PUBLICATION testpub6_forschema FOR TABLES IN SCHEMA "CURRENT_SCHEMA", CURRENT_SCHEMA;

CREATE PUBLICATION testpub_fortable FOR TABLE "CURRENT_SCHEMA"."CURRENT_SCHEMA";

RESET client_min_messages;

SET SEARCH_PATH='';

CREATE PUBLICATION testpub_forschema FOR TABLES IN SCHEMA CURRENT_SCHEMA;

RESET SEARCH_PATH;

CREATE PUBLICATION testpub_forschema FOR TABLES IN SCHEMA non_existent_schema;

CREATE PUBLICATION testpub_forschema FOR TABLES IN SCHEMA pg_catalog;

CREATE PUBLICATION testpub1_forschema1 FOR TABLES IN SCHEMA testpub_view;

DROP SCHEMA pub_test3;

ALTER SCHEMA pub_test1 RENAME to pub_test1_renamed;

ALTER SCHEMA pub_test1_renamed RENAME to pub_test1;

ALTER PUBLICATION testpub1_forschema ADD TABLES IN SCHEMA pub_test2;

ALTER PUBLICATION testpub1_forschema ADD TABLES IN SCHEMA non_existent_schema;

ALTER PUBLICATION testpub1_forschema ADD TABLES IN SCHEMA pub_test1;

ALTER PUBLICATION testpub1_forschema DROP TABLES IN SCHEMA pub_test2;

ALTER PUBLICATION testpub1_forschema DROP TABLES IN SCHEMA pub_test2;

ALTER PUBLICATION testpub1_forschema DROP TABLES IN SCHEMA non_existent_schema;

ALTER PUBLICATION testpub1_forschema DROP TABLES IN SCHEMA pub_test1;

ALTER PUBLICATION testpub1_forschema SET TABLES IN SCHEMA pub_test1, pub_test2;

ALTER PUBLICATION testpub1_forschema SET TABLES IN SCHEMA non_existent_schema;

ALTER PUBLICATION testpub1_forschema SET TABLES IN SCHEMA pub_test1, pub_test1;

ALTER PUBLICATION testpub2_forschema DROP TABLES IN SCHEMA pub_test1;

DROP PUBLICATION testpub3_forschema, testpub4_forschema, testpub5_forschema, testpub6_forschema, testpub_fortable;

DROP SCHEMA "CURRENT_SCHEMA" CASCADE;

INSERT INTO pub_test1.tbl VALUES(1, 'test');

UPDATE pub_test1.tbl SET id = 2;

ALTER PUBLICATION testpub1_forschema DROP TABLES IN SCHEMA pub_test1;

UPDATE pub_test1.tbl SET id = 2;

ALTER PUBLICATION testpub1_forschema SET TABLES IN SCHEMA pub_test1;

UPDATE pub_test1.tbl SET id = 2;

CREATE SCHEMA pub_testpart1;

CREATE SCHEMA pub_testpart2;

CREATE TABLE pub_testpart1.parent1 (a int) partition by list (a);

CREATE TABLE pub_testpart2.child_parent1 partition of pub_testpart1.parent1 for values in (1);

INSERT INTO pub_testpart2.child_parent1 values(1);

UPDATE pub_testpart2.child_parent1 set a = 1;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpubpart_forschema FOR TABLES IN SCHEMA pub_testpart1;

RESET client_min_messages;

UPDATE pub_testpart1.parent1 set a = 1;

UPDATE pub_testpart2.child_parent1 set a = 1;

DROP PUBLICATION testpubpart_forschema;

CREATE TABLE pub_testpart2.parent2 (a int) partition by list (a);

CREATE TABLE pub_testpart1.child_parent2 partition of pub_testpart2.parent2 for values in (1);

INSERT INTO pub_testpart1.child_parent2 values(1);

UPDATE pub_testpart1.child_parent2 set a = 1;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpubpart_forschema FOR TABLES IN SCHEMA pub_testpart2;

RESET client_min_messages;

UPDATE pub_testpart2.child_parent1 set a = 1;

UPDATE pub_testpart2.parent2 set a = 1;

UPDATE pub_testpart1.child_parent2 set a = 1;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub3_forschema;

RESET client_min_messages;

ALTER PUBLICATION testpub3_forschema SET TABLES IN SCHEMA pub_test1;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION testpub_forschema_fortable FOR TABLES IN SCHEMA pub_test1, TABLE pub_test2.tbl1;

CREATE PUBLICATION testpub_fortable_forschema FOR TABLE pub_test2.tbl1, TABLES IN SCHEMA pub_test1;

RESET client_min_messages;

DROP VIEW testpub_view;

DROP PUBLICATION testpub_default;

DROP PUBLICATION testpub_ins_trunct;

DROP PUBLICATION testpub_fortbl;

DROP PUBLICATION testpub1_forschema;

DROP PUBLICATION testpub2_forschema;

DROP PUBLICATION testpub3_forschema;

DROP PUBLICATION testpub_forschema_fortable;

DROP PUBLICATION testpub_fortable_forschema;

DROP PUBLICATION testpubpart_forschema;

DROP SCHEMA pub_test CASCADE;

DROP SCHEMA pub_test1 CASCADE;

DROP SCHEMA pub_test2 CASCADE;

DROP SCHEMA pub_testpart1 CASCADE;

DROP SCHEMA pub_testpart2 CASCADE;

SET client_min_messages = 'ERROR';

CREATE SCHEMA sch1;

CREATE SCHEMA sch2;

CREATE TABLE sch1.tbl1 (a int) PARTITION BY RANGE(a);

CREATE TABLE sch2.tbl1_part1 PARTITION OF sch1.tbl1 FOR VALUES FROM (1) to (10);

CREATE PUBLICATION pub FOR TABLES IN SCHEMA sch2 WITH (PUBLISH_VIA_PARTITION_ROOT=1);

SELECT * FROM pg_publication_tables;

DROP PUBLICATION pub;

CREATE PUBLICATION pub FOR TABLE sch2.tbl1_part1 WITH (PUBLISH_VIA_PARTITION_ROOT=1);

SELECT * FROM pg_publication_tables;

ALTER PUBLICATION pub ADD TABLE sch1.tbl1;

SELECT * FROM pg_publication_tables;

DROP PUBLICATION pub;

CREATE PUBLICATION pub FOR TABLES IN SCHEMA sch2 WITH (PUBLISH_VIA_PARTITION_ROOT=0);

SELECT * FROM pg_publication_tables;

DROP PUBLICATION pub;

CREATE PUBLICATION pub FOR TABLE sch2.tbl1_part1 WITH (PUBLISH_VIA_PARTITION_ROOT=0);

SELECT * FROM pg_publication_tables;

ALTER PUBLICATION pub ADD TABLE sch1.tbl1;

SELECT * FROM pg_publication_tables;

DROP PUBLICATION pub;

DROP TABLE sch2.tbl1_part1;

DROP TABLE sch1.tbl1;

CREATE TABLE sch1.tbl1 (a int) PARTITION BY RANGE(a);

CREATE TABLE sch1.tbl1_part1 PARTITION OF sch1.tbl1 FOR VALUES FROM (1) to (10);

CREATE TABLE sch1.tbl1_part2 PARTITION OF sch1.tbl1 FOR VALUES FROM (10) to (20);

CREATE TABLE sch1.tbl1_part3 (a int) PARTITION BY RANGE(a);

ALTER TABLE sch1.tbl1 ATTACH PARTITION sch1.tbl1_part3 FOR VALUES FROM (20) to (30);

CREATE PUBLICATION pub FOR TABLES IN SCHEMA sch1 WITH (PUBLISH_VIA_PARTITION_ROOT=1);

SELECT * FROM pg_publication_tables;

RESET client_min_messages;

DROP PUBLICATION pub;

DROP TABLE sch1.tbl1;

DROP SCHEMA sch1 cascade;

DROP SCHEMA sch2 cascade;

SET client_min_messages = 'ERROR';

CREATE PUBLICATION pub1 FOR ALL TABLES WITH (publish_generated_columns = stored);

CREATE PUBLICATION pub2 FOR ALL TABLES WITH (publish_generated_columns = none);

DROP PUBLICATION pub1;

DROP PUBLICATION pub2;

CREATE TABLE gencols (a int, gen1 int GENERATED ALWAYS AS (a * 2) STORED);

CREATE PUBLICATION pub1 FOR table gencols(a, gen1) WITH (publish_generated_columns = none);

CREATE PUBLICATION pub2 FOR table gencols(a, gen1) WITH (publish_generated_columns = stored);

ALTER PUBLICATION pub2 SET (publish_generated_columns = none);

ALTER PUBLICATION pub2 SET TABLE gencols(a);

ALTER PUBLICATION pub2 SET TABLE gencols(a, gen1);

DROP PUBLICATION pub1;

DROP PUBLICATION pub2;

DROP TABLE gencols;

RESET client_min_messages;

CREATE TABLE testpub_insert_onconfl_no_ri (a int unique, b int);

CREATE TABLE testpub_insert_onconfl_parted (a int unique, b int) PARTITION by RANGE (a);

CREATE TABLE testpub_insert_onconfl_part_no_ri PARTITION OF testpub_insert_onconfl_parted FOR VALUES FROM (1) TO (10);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION pub1 FOR ALL TABLES;

RESET client_min_messages;

INSERT INTO testpub_insert_onconfl_no_ri VALUES (1, 1) ON CONFLICT (a) DO UPDATE SET b = 2;

INSERT INTO testpub_insert_onconfl_no_ri VALUES (1, 1) ON CONFLICT DO NOTHING;

INSERT INTO testpub_insert_onconfl_parted VALUES (1, 1) ON CONFLICT (a) DO UPDATE SET b = 2;

INSERT INTO testpub_insert_onconfl_parted VALUES (1, 1) ON CONFLICT DO NOTHING;

DROP PUBLICATION pub1;

DROP TABLE testpub_insert_onconfl_no_ri;

DROP TABLE testpub_insert_onconfl_parted;

CREATE TABLE testpub_merge_no_ri (a int, b int);

CREATE TABLE testpub_merge_pk (a int primary key, b int);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION pub1 FOR ALL TABLES;

RESET client_min_messages;

DROP PUBLICATION pub1;

DROP TABLE testpub_merge_no_ri;

DROP TABLE testpub_merge_pk;

RESET SESSION AUTHORIZATION;

DROP ROLE regress_publication_user, regress_publication_user2;

DROP ROLE regress_publication_user_dummy;

CREATE SCHEMA pubme

CREATE TABLE t0 (c int, d int)

CREATE TABLE t1 (c int);

CREATE SCHEMA pubme2

CREATE TABLE t0 (c int, d int);

SET client_min_messages = 'ERROR';

CREATE PUBLICATION dump_pub_qual_1ct FOR
  TABLE ONLY pubme.t0 (c, d) WHERE (c > 0);

CREATE PUBLICATION dump_pub_qual_2ct FOR
  TABLE ONLY pubme.t0 (c) WHERE (c > 0),
  TABLE ONLY pubme.t1 (c);

CREATE PUBLICATION dump_pub_nsp_1ct FOR
  TABLES IN SCHEMA pubme;

CREATE PUBLICATION dump_pub_nsp_2ct FOR
  TABLES IN SCHEMA pubme,
  TABLES IN SCHEMA pubme2;

CREATE PUBLICATION dump_pub_all FOR
  TABLE ONLY pubme.t0,
  TABLE ONLY pubme.t1 WHERE (c < 0),
  TABLES IN SCHEMA pubme,
  TABLES IN SCHEMA pubme2
  WITH (publish_via_partition_root = true);

RESET client_min_messages;
