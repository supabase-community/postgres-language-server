SET client_min_messages TO 'warning';

DROP ROLE IF EXISTS regress_alter_table_user1;

RESET client_min_messages;

CREATE USER regress_alter_table_user1;

CREATE TABLE attmp (initial int4);

COMMENT ON TABLE attmp_wrong IS 'table comment';

COMMENT ON TABLE attmp IS 'table comment';

COMMENT ON TABLE attmp IS NULL;

ALTER TABLE attmp ADD COLUMN xmin integer;

ALTER TABLE attmp ADD COLUMN a int4 default 3;

ALTER TABLE attmp ADD COLUMN b name;

ALTER TABLE attmp ADD COLUMN c text;

ALTER TABLE attmp ADD COLUMN d float8;

ALTER TABLE attmp ADD COLUMN e float4;

ALTER TABLE attmp ADD COLUMN f int2;

ALTER TABLE attmp ADD COLUMN g polygon;

ALTER TABLE attmp ADD COLUMN i char;

ALTER TABLE attmp ADD COLUMN k int4;

ALTER TABLE attmp ADD COLUMN l tid;

ALTER TABLE attmp ADD COLUMN m xid;

ALTER TABLE attmp ADD COLUMN n oidvector;

ALTER TABLE attmp ADD COLUMN p boolean;

ALTER TABLE attmp ADD COLUMN q point;

ALTER TABLE attmp ADD COLUMN r lseg;

ALTER TABLE attmp ADD COLUMN s path;

ALTER TABLE attmp ADD COLUMN t box;

ALTER TABLE attmp ADD COLUMN v timestamp;

ALTER TABLE attmp ADD COLUMN w interval;

ALTER TABLE attmp ADD COLUMN x float8[];

ALTER TABLE attmp ADD COLUMN y float4[];

ALTER TABLE attmp ADD COLUMN z int2[];

INSERT INTO attmp (a, b, c, d, e, f, g,    i,    k, l, m, n, p, q, r, s, t,
	v, w, x, y, z)
   VALUES (4, 'name', 'text', 4.1, 4.1, 2, '(4.1,4.1,3.1,3.1)',
	'c',
	314159, '(1,1)', '512',
	'1 2 3 4 5 6 7 8', true, '(1.1,1.1)', '(4.1,4.1,3.1,3.1)',
	'(0,2,4.1,4.1,3.1,3.1)', '(4.1,4.1,3.1,3.1)',
	'epoch', '01:00:10', '{1.0,2.0,3.0,4.0}', '{1.0,2.0,3.0,4.0}', '{1,2,3,4}');

SELECT * FROM attmp;

DROP TABLE attmp;

CREATE TABLE attmp (
	initial 	int4
);

ALTER TABLE attmp ADD COLUMN a int4;

ALTER TABLE attmp ADD COLUMN b name;

ALTER TABLE attmp ADD COLUMN c text;

ALTER TABLE attmp ADD COLUMN d float8;

ALTER TABLE attmp ADD COLUMN e float4;

ALTER TABLE attmp ADD COLUMN f int2;

ALTER TABLE attmp ADD COLUMN g polygon;

ALTER TABLE attmp ADD COLUMN i char;

ALTER TABLE attmp ADD COLUMN k int4;

ALTER TABLE attmp ADD COLUMN l tid;

ALTER TABLE attmp ADD COLUMN m xid;

ALTER TABLE attmp ADD COLUMN n oidvector;

ALTER TABLE attmp ADD COLUMN p boolean;

ALTER TABLE attmp ADD COLUMN q point;

ALTER TABLE attmp ADD COLUMN r lseg;

ALTER TABLE attmp ADD COLUMN s path;

ALTER TABLE attmp ADD COLUMN t box;

ALTER TABLE attmp ADD COLUMN v timestamp;

ALTER TABLE attmp ADD COLUMN w interval;

ALTER TABLE attmp ADD COLUMN x float8[];

ALTER TABLE attmp ADD COLUMN y float4[];

ALTER TABLE attmp ADD COLUMN z int2[];

INSERT INTO attmp (a, b, c, d, e, f, g,    i,   k, l, m, n, p, q, r, s, t,
	v, w, x, y, z)
   VALUES (4, 'name', 'text', 4.1, 4.1, 2, '(4.1,4.1,3.1,3.1)',
        'c',
	314159, '(1,1)', '512',
	'1 2 3 4 5 6 7 8', true, '(1.1,1.1)', '(4.1,4.1,3.1,3.1)',
	'(0,2,4.1,4.1,3.1,3.1)', '(4.1,4.1,3.1,3.1)',
	'epoch', '01:00:10', '{1.0,2.0,3.0,4.0}', '{1.0,2.0,3.0,4.0}', '{1,2,3,4}');

SELECT * FROM attmp;

CREATE INDEX attmp_idx ON attmp (a, (d + e), b);

ALTER INDEX attmp_idx ALTER COLUMN 1 SET STATISTICS 1000;

ALTER INDEX attmp_idx ALTER COLUMN 2 SET STATISTICS 1000;

ALTER INDEX attmp_idx ALTER COLUMN 3 SET STATISTICS 1000;

ALTER INDEX attmp_idx ALTER COLUMN 4 SET STATISTICS 1000;

ALTER INDEX attmp_idx ALTER COLUMN 2 SET STATISTICS -1;

DROP TABLE attmp;

CREATE TABLE attmp (regtable int);

CREATE TEMP TABLE attmp (attmptable int);

ALTER TABLE attmp RENAME TO attmp_new;

SELECT * FROM attmp;

SELECT * FROM attmp_new;

ALTER TABLE attmp RENAME TO attmp_new2;

SELECT * FROM attmp;

SELECT * FROM attmp_new;

SELECT * FROM attmp_new2;

DROP TABLE attmp_new;

DROP TABLE attmp_new2;

CREATE TABLE part_attmp (a int primary key) partition by range (a);

CREATE TABLE part_attmp1 PARTITION OF part_attmp FOR VALUES FROM (0) TO (100);

ALTER INDEX part_attmp_pkey RENAME TO part_attmp_index;

ALTER INDEX part_attmp1_pkey RENAME TO part_attmp1_index;

ALTER TABLE part_attmp RENAME TO part_at2tmp;

ALTER TABLE part_attmp1 RENAME TO part_at2tmp1;

SET ROLE regress_alter_table_user1;

ALTER INDEX part_attmp_index RENAME TO fail;

ALTER INDEX part_attmp1_index RENAME TO fail;

ALTER TABLE part_at2tmp RENAME TO fail;

ALTER TABLE part_at2tmp1 RENAME TO fail;

RESET ROLE;

DROP TABLE part_at2tmp;

CREATE TABLE attmp_array (id int);

CREATE TABLE attmp_array2 (id int);

SELECT typname FROM pg_type WHERE oid = 'attmp_array[]'::regtype;

SELECT typname FROM pg_type WHERE oid = 'attmp_array2[]'::regtype;

ALTER TABLE attmp_array2 RENAME TO _attmp_array;

SELECT typname FROM pg_type WHERE oid = 'attmp_array[]'::regtype;

SELECT typname FROM pg_type WHERE oid = '_attmp_array[]'::regtype;

DROP TABLE _attmp_array;

DROP TABLE attmp_array;

CREATE TABLE attmp_array (id int);

SELECT typname FROM pg_type WHERE oid = 'attmp_array[]'::regtype;

ALTER TABLE attmp_array RENAME TO _attmp_array;

SELECT typname FROM pg_type WHERE oid = '_attmp_array[]'::regtype;

DROP TABLE _attmp_array;

ALTER INDEX IF EXISTS __onek_unique1 RENAME TO attmp_onek_unique1;

ALTER INDEX IF EXISTS __attmp_onek_unique1 RENAME TO onek_unique1;

ALTER INDEX onek_unique1 RENAME TO attmp_onek_unique1;

ALTER INDEX attmp_onek_unique1 RENAME TO onek_unique1;

SET ROLE regress_alter_table_user1;

ALTER INDEX onek_unique1 RENAME TO fail;

RESET ROLE;

CREATE TABLE alter_idx_rename_test (a INT);

CREATE INDEX alter_idx_rename_test_idx ON alter_idx_rename_test (a);

CREATE TABLE alter_idx_rename_test_parted (a INT) PARTITION BY LIST (a);

CREATE INDEX alter_idx_rename_test_parted_idx ON alter_idx_rename_test_parted (a);

BEGIN;

ALTER INDEX alter_idx_rename_test RENAME TO alter_idx_rename_test_2;

ALTER INDEX alter_idx_rename_test_parted RENAME TO alter_idx_rename_test_parted_2;

SELECT relation::regclass, mode FROM pg_locks
WHERE pid = pg_backend_pid() AND locktype = 'relation'
  AND relation::regclass::text LIKE 'alter\_idx%'
ORDER BY relation::regclass::text COLLATE "C";

COMMIT;

BEGIN;

ALTER INDEX alter_idx_rename_test_idx RENAME TO alter_idx_rename_test_idx_2;

ALTER INDEX alter_idx_rename_test_parted_idx RENAME TO alter_idx_rename_test_parted_idx_2;

SELECT relation::regclass, mode FROM pg_locks
WHERE pid = pg_backend_pid() AND locktype = 'relation'
  AND relation::regclass::text LIKE 'alter\_idx%'
ORDER BY relation::regclass::text COLLATE "C";

COMMIT;

BEGIN;

ALTER TABLE alter_idx_rename_test_idx_2 RENAME TO alter_idx_rename_test_idx_3;

ALTER TABLE alter_idx_rename_test_parted_idx_2 RENAME TO alter_idx_rename_test_parted_idx_3;

SELECT relation::regclass, mode FROM pg_locks
WHERE pid = pg_backend_pid() AND locktype = 'relation'
  AND relation::regclass::text LIKE 'alter\_idx%'
ORDER BY relation::regclass::text COLLATE "C";

COMMIT;

DROP TABLE alter_idx_rename_test_2;

CREATE VIEW attmp_view (unique1) AS SELECT unique1 FROM tenk1;

ALTER TABLE attmp_view RENAME TO attmp_view_new;

SET ROLE regress_alter_table_user1;

ALTER VIEW attmp_view_new RENAME TO fail;

RESET ROLE;

set enable_seqscan to off;

set enable_bitmapscan to off;

SELECT unique1 FROM tenk1 WHERE unique1 < 5;

reset enable_seqscan;

reset enable_bitmapscan;

DROP VIEW attmp_view_new;

alter table stud_emp rename to pg_toast_stud_emp;

alter table pg_toast_stud_emp rename to stud_emp;

ALTER TABLE onek ADD CONSTRAINT onek_unique1_constraint UNIQUE (unique1);

ALTER INDEX onek_unique1_constraint RENAME TO onek_unique1_constraint_foo;

ALTER TABLE onek DROP CONSTRAINT onek_unique1_constraint_foo;

ALTER TABLE onek ADD CONSTRAINT onek_check_constraint CHECK (unique1 >= 0);

ALTER TABLE onek RENAME CONSTRAINT onek_check_constraint TO onek_check_constraint_foo;

ALTER TABLE onek DROP CONSTRAINT onek_check_constraint_foo;

ALTER TABLE onek ADD CONSTRAINT onek_unique1_constraint UNIQUE (unique1);

DROP INDEX onek_unique1_constraint;

ALTER TABLE onek RENAME CONSTRAINT onek_unique1_constraint TO onek_unique1_constraint_foo;

DROP INDEX onek_unique1_constraint_foo;

ALTER TABLE onek DROP CONSTRAINT onek_unique1_constraint_foo;

CREATE TABLE constraint_rename_test (a int CONSTRAINT con1 CHECK (a > 0), b int, c int);

CREATE TABLE constraint_rename_test2 (a int CONSTRAINT con1 CHECK (a > 0), d int) INHERITS (constraint_rename_test);

ALTER TABLE constraint_rename_test2 RENAME CONSTRAINT con1 TO con1foo;

ALTER TABLE ONLY constraint_rename_test RENAME CONSTRAINT con1 TO con1foo;

ALTER TABLE constraint_rename_test RENAME CONSTRAINT con1 TO con1foo;

ALTER TABLE constraint_rename_test ADD CONSTRAINT con2 CHECK (b > 0) NO INHERIT;

ALTER TABLE ONLY constraint_rename_test RENAME CONSTRAINT con2 TO con2foo;

ALTER TABLE constraint_rename_test RENAME CONSTRAINT con2foo TO con2bar;

ALTER TABLE constraint_rename_test ADD CONSTRAINT con3 PRIMARY KEY (a);

ALTER TABLE constraint_rename_test RENAME CONSTRAINT con3 TO con3foo;

DROP TABLE constraint_rename_test2;

DROP TABLE constraint_rename_test;

ALTER TABLE IF EXISTS constraint_not_exist RENAME CONSTRAINT con3 TO con3foo;

ALTER TABLE IF EXISTS constraint_rename_test ADD CONSTRAINT con4 UNIQUE (a);

CREATE TABLE constraint_rename_cache (a int,
  CONSTRAINT chk_a CHECK (a > 0),
  PRIMARY KEY (a));

ALTER TABLE constraint_rename_cache
  RENAME CONSTRAINT chk_a TO chk_a_new;

ALTER TABLE constraint_rename_cache
  RENAME CONSTRAINT constraint_rename_cache_pkey TO constraint_rename_pkey_new;

CREATE TABLE like_constraint_rename_cache
  (LIKE constraint_rename_cache INCLUDING ALL);

DROP TABLE constraint_rename_cache;

DROP TABLE like_constraint_rename_cache;

CREATE TABLE attmp2 (a int primary key);

CREATE TABLE attmp3 (a int, b int);

CREATE TABLE attmp4 (a int, b int, unique(a,b));

CREATE TABLE attmp5 (a int, b int);

INSERT INTO attmp2 values (1);

INSERT INTO attmp2 values (2);

INSERT INTO attmp2 values (3);

INSERT INTO attmp2 values (4);

INSERT INTO attmp3 values (1,10);

INSERT INTO attmp3 values (1,20);

INSERT INTO attmp3 values (5,50);

ALTER TABLE attmp3 add constraint attmpconstr foreign key(c) references attmp2 match full;

ALTER TABLE attmp3 add constraint attmpconstr foreign key(a) references attmp2(b) match full;

ALTER TABLE attmp3 add constraint attmpconstr foreign key (a) references attmp2 match full;

DELETE FROM attmp3 where a=5;

ALTER TABLE attmp3 add constraint attmpconstr foreign key (a) references attmp2 match full;

ALTER TABLE attmp3 drop constraint attmpconstr;

INSERT INTO attmp3 values (5,50);

ALTER TABLE attmp3 add constraint attmpconstr foreign key (a) references attmp2 match full NOT VALID;

ALTER TABLE attmp3 validate constraint attmpconstr;

DELETE FROM attmp3 where a=5;

ALTER TABLE attmp3 validate constraint attmpconstr;

ALTER TABLE attmp3 validate constraint attmpconstr;

ALTER TABLE attmp3 ADD CONSTRAINT b_greater_than_ten CHECK (b > 10);

ALTER TABLE attmp3 ADD CONSTRAINT b_greater_than_ten CHECK (b > 10) NOT VALID;

ALTER TABLE attmp3 VALIDATE CONSTRAINT b_greater_than_ten;

DELETE FROM attmp3 WHERE NOT b > 10;

ALTER TABLE attmp3 VALIDATE CONSTRAINT b_greater_than_ten;

ALTER TABLE attmp3 VALIDATE CONSTRAINT b_greater_than_ten;

ALTER TABLE attmp3 VALIDATE CONSTRAINT b_greater_than_ten_not_enforced;

select * from attmp3;

CREATE TABLE attmp6 () INHERITS (attmp3);

CREATE TABLE attmp7 () INHERITS (attmp3);

INSERT INTO attmp6 VALUES (6, 30), (7, 16);

ALTER TABLE attmp3 ADD CONSTRAINT b_le_20 CHECK (b <= 20) NOT VALID;

ALTER TABLE attmp3 VALIDATE CONSTRAINT b_le_20;

DELETE FROM attmp6 WHERE b > 20;

ALTER TABLE attmp3 VALIDATE CONSTRAINT b_le_20;

CREATE FUNCTION boo(int) RETURNS int IMMUTABLE STRICT LANGUAGE plpgsql AS $$ BEGIN RAISE NOTICE 'boo: %', $1; RETURN $1; END; $$;

INSERT INTO attmp7 VALUES (8, 18);

ALTER TABLE attmp7 ADD CONSTRAINT identity CHECK (b = boo(b));

ALTER TABLE attmp3 ADD CONSTRAINT IDENTITY check (b = boo(b)) NOT VALID;

ALTER TABLE attmp3 VALIDATE CONSTRAINT identity;

create table parent_noinh_convalid (a int);

create table child_noinh_convalid () inherits (parent_noinh_convalid);

insert into parent_noinh_convalid values (1);

insert into child_noinh_convalid values (1);

alter table parent_noinh_convalid add constraint check_a_is_2 check (a = 2) no inherit not valid;

alter table parent_noinh_convalid validate constraint check_a_is_2;

delete from only parent_noinh_convalid;

alter table parent_noinh_convalid validate constraint check_a_is_2;

select convalidated from pg_constraint where conrelid = 'parent_noinh_convalid'::regclass and conname = 'check_a_is_2';

drop table parent_noinh_convalid, child_noinh_convalid;

ALTER TABLE attmp5 add constraint attmpconstr foreign key(a) references attmp4(a) match full;

DROP TABLE attmp7;

DROP TABLE attmp6;

DROP TABLE attmp5;

DROP TABLE attmp4;

DROP TABLE attmp3;

DROP TABLE attmp2;

set constraint_exclusion TO 'partition';

create table nv_parent (d date, check (false) no inherit not valid);

create table nv_child_2010 () inherits (nv_parent);

create table nv_child_2011 () inherits (nv_parent);

alter table nv_child_2010 add check (d between '2010-01-01'::date and '2010-12-31'::date) not valid;

alter table nv_child_2011 add check (d between '2011-01-01'::date and '2011-12-31'::date) not valid;

select * from nv_parent where d between '2011-08-01' and '2011-08-31';

create table nv_child_2009 (check (d between '2009-01-01'::date and '2009-12-31'::date)) inherits (nv_parent);

select * from nv_parent where d between '2011-08-01'::date and '2011-08-31'::date;

select * from nv_parent where d between '2009-08-01'::date and '2009-08-31'::date;

alter table nv_child_2011 VALIDATE CONSTRAINT nv_child_2011_d_check;

select * from nv_parent where d between '2009-08-01'::date and '2009-08-31'::date;

alter table nv_parent add check (d between '2001-01-01'::date and '2099-12-31'::date) not valid;

CREATE TEMP TABLE PKTABLE (ptest1 int PRIMARY KEY);

INSERT INTO PKTABLE VALUES(42);

CREATE TEMP TABLE FKTABLE (ftest1 inet);

ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest1) references pktable;

ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest1) references pktable(ptest1);

DROP TABLE FKTABLE;

CREATE TEMP TABLE FKTABLE (ftest1 int8);

ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest1) references pktable;

INSERT INTO FKTABLE VALUES(42);

INSERT INTO FKTABLE VALUES(43);

DROP TABLE FKTABLE;

CREATE TEMP TABLE FKTABLE (ftest1 numeric);

ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest1) references pktable;

DROP TABLE FKTABLE;

DROP TABLE PKTABLE;

CREATE TEMP TABLE PKTABLE (ptest1 numeric PRIMARY KEY);

INSERT INTO PKTABLE VALUES(42);

CREATE TEMP TABLE FKTABLE (ftest1 int);

ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest1) references pktable;

INSERT INTO FKTABLE VALUES(42);

INSERT INTO FKTABLE VALUES(43);

DROP TABLE FKTABLE;

DROP TABLE PKTABLE;

CREATE TEMP TABLE PKTABLE (ptest1 int, ptest2 inet,
                           PRIMARY KEY(ptest1, ptest2));

CREATE TEMP TABLE FKTABLE (ftest1 cidr, ftest2 timestamp);

ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest1, ftest2) references pktable;

DROP TABLE FKTABLE;

CREATE TEMP TABLE FKTABLE (ftest1 cidr, ftest2 timestamp);

ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest1, ftest2)
     references pktable(ptest1, ptest2);

DROP TABLE FKTABLE;

CREATE TEMP TABLE FKTABLE (ftest1 int, ftest2 inet);

ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest1, ftest2)
     references pktable(ptest2, ptest1);

ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest2, ftest1)
     references pktable(ptest1, ptest2);

DROP TABLE FKTABLE;

DROP TABLE PKTABLE;

CREATE TEMP TABLE PKTABLE (ptest1 int primary key);

CREATE TEMP TABLE FKTABLE (ftest1 int);

ALTER TABLE FKTABLE ADD CONSTRAINT fknd FOREIGN KEY(ftest1) REFERENCES pktable
  ON DELETE CASCADE ON UPDATE NO ACTION NOT DEFERRABLE;

ALTER TABLE FKTABLE ADD CONSTRAINT fkdd FOREIGN KEY(ftest1) REFERENCES pktable
  ON DELETE CASCADE ON UPDATE NO ACTION DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE FKTABLE ADD CONSTRAINT fkdi FOREIGN KEY(ftest1) REFERENCES pktable
  ON DELETE CASCADE ON UPDATE NO ACTION DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE FKTABLE ADD CONSTRAINT fknd2 FOREIGN KEY(ftest1) REFERENCES pktable
  ON DELETE CASCADE ON UPDATE NO ACTION DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE FKTABLE ALTER CONSTRAINT fknd2 NOT DEFERRABLE;

ALTER TABLE FKTABLE ADD CONSTRAINT fkdd2 FOREIGN KEY(ftest1) REFERENCES pktable
  ON DELETE CASCADE ON UPDATE NO ACTION NOT DEFERRABLE;

ALTER TABLE FKTABLE ALTER CONSTRAINT fkdd2 DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE FKTABLE ADD CONSTRAINT fkdi2 FOREIGN KEY(ftest1) REFERENCES pktable
  ON DELETE CASCADE ON UPDATE NO ACTION NOT DEFERRABLE;

ALTER TABLE FKTABLE ALTER CONSTRAINT fkdi2 DEFERRABLE INITIALLY IMMEDIATE;

SELECT conname, tgfoid::regproc, tgtype, tgdeferrable, tginitdeferred
FROM pg_trigger JOIN pg_constraint con ON con.oid = tgconstraint
WHERE tgrelid = 'pktable'::regclass
ORDER BY 1,2,3;

SELECT conname, tgfoid::regproc, tgtype, tgdeferrable, tginitdeferred
FROM pg_trigger JOIN pg_constraint con ON con.oid = tgconstraint
WHERE tgrelid = 'fktable'::regclass
ORDER BY 1,2,3;

create table atacc1 ( test int );

alter table atacc1 add constraint atacc_test1 check (test>3);

insert into atacc1 (test) values (2);

insert into atacc1 (test) values (4);

drop table atacc1;

create table atacc1 ( test int );

insert into atacc1 (test) values (2);

alter table atacc1 add constraint atacc_test1 check (test>3);

insert into atacc1 (test) values (4);

drop table atacc1;

create table atacc1 ( test int );

alter table atacc1 add constraint atacc_test1 check (test1>3);

drop table atacc1;

create table atacc1 ( test int, test2 int, test3 int);

alter table atacc1 add constraint atacc_test1 check (test+test2<test3*4);

insert into atacc1 (test,test2,test3) values (4,4,2);

insert into atacc1 (test,test2,test3) values (4,4,5);

drop table atacc1;

create table atacc1 (test int check (test>3), test2 int);

alter table atacc1 add check (test2>test);

insert into atacc1 (test2, test) values (3, 4);

drop table atacc1;

create table atacc1 (test int);

create table atacc2 (test2 int);

create table atacc3 (test3 int) inherits (atacc1, atacc2);

alter table atacc2 add constraint foo check (test2>0);

insert into atacc2 (test2) values (-3);

insert into atacc2 (test2) values (3);

insert into atacc3 (test2) values (-3);

insert into atacc3 (test2) values (3);

drop table atacc3;

drop table atacc2;

drop table atacc1;

create table atacc1 (test int);

create table atacc2 (test2 int);

create table atacc3 (test3 int) inherits (atacc1, atacc2);

alter table atacc3 no inherit atacc2;

alter table atacc3 no inherit atacc2;

insert into atacc3 (test2) values (3);

select test2 from atacc2;

alter table atacc2 add constraint foo check (test2>0);

alter table atacc3 inherit atacc2;

alter table atacc3 rename test2 to testx;

alter table atacc3 inherit atacc2;

alter table atacc3 add test2 bool;

alter table atacc3 inherit atacc2;

alter table atacc3 drop test2;

alter table atacc3 add test2 int;

update atacc3 set test2 = 4 where test2 is null;

alter table atacc3 add constraint foo check (test2>0);

alter table atacc3 inherit atacc2;

alter table atacc3 inherit atacc2;

alter table atacc2 inherit atacc3;

alter table atacc2 inherit atacc2;

select test2 from atacc2;

drop table atacc2 cascade;

drop table atacc1;

create table atacc1 (test int);

create table atacc2 (test2 int) inherits (atacc1);

alter table atacc1 add constraint foo check (test>0) no inherit;

insert into atacc2 (test) values (-3);

insert into atacc1 (test) values (-3);

insert into atacc1 (test) values (3);

alter table atacc2 add constraint foo check (test>0) no inherit;

drop table atacc2;

drop table atacc1;

create table atacc1 ( test int ) ;

alter table atacc1 add constraint atacc_test1 unique (test);

insert into atacc1 (test) values (2);

insert into atacc1 (test) values (2);

insert into atacc1 (test) values (4);

alter table atacc1 alter column test type integer using 0;

drop table atacc1;

create table atacc1 ( test int );

insert into atacc1 (test) values (2);

insert into atacc1 (test) values (2);

alter table atacc1 add constraint atacc_test1 unique (test);

insert into atacc1 (test) values (3);

drop table atacc1;

create table atacc1 ( test int );

alter table atacc1 add constraint atacc_test1 unique (test1);

drop table atacc1;

create table atacc1 ( test int, test2 int);

alter table atacc1 add constraint atacc_test1 unique (test, test2);

insert into atacc1 (test,test2) values (4,4);

insert into atacc1 (test,test2) values (4,4);

insert into atacc1 (test,test2) values (4,5);

insert into atacc1 (test,test2) values (5,4);

insert into atacc1 (test,test2) values (5,5);

drop table atacc1;

create table atacc1 (test int, test2 int, unique(test));

alter table atacc1 add unique (test2);

insert into atacc1 (test2, test) values (3, 3);

insert into atacc1 (test2, test) values (2, 3);

drop table atacc1;

create table atacc1 ( id serial, test int) ;

alter table atacc1 add constraint atacc_test1 primary key (test);

insert into atacc1 (test) values (2);

insert into atacc1 (test) values (2);

insert into atacc1 (test) values (4);

insert into atacc1 (test) values(NULL);

alter table atacc1 add constraint atacc_oid1 primary key(id);

alter table atacc1 drop constraint atacc_test1 restrict;

alter table atacc1 add constraint atacc_oid1 primary key(id);

drop table atacc1;

create table atacc1 ( test int );

insert into atacc1 (test) values (2);

insert into atacc1 (test) values (2);

alter table atacc1 add constraint atacc_test1 primary key (test);

insert into atacc1 (test) values (3);

drop table atacc1;

create table atacc1 ( test int );

insert into atacc1 (test) values (NULL);

alter table atacc1 add constraint atacc_test1 primary key (test);

insert into atacc1 (test) values (3);

drop table atacc1;

create table atacc1 ( test int );

alter table atacc1 add constraint atacc_test1 primary key (test1);

drop table atacc1;

create table atacc1 ( test int );

insert into atacc1 (test) values (0);

alter table atacc1 add column test2 int primary key;

alter table atacc1 add column test2 int default 0 primary key;

drop table atacc1;

create table atacc1 (a int);

insert into atacc1 values(1);

alter table atacc1
  add column b float8 not null default random(),
  add primary key(a);

drop table atacc1;

create table atacc1 (a int primary key);

alter table atacc1 add constraint atacc1_fkey foreign key (a) references atacc1 (a) not valid;

alter table atacc1 validate constraint atacc1_fkey, alter a type bigint;

drop table atacc1;

create table atacc1 (a bigint, b int);

insert into atacc1 values(1,1);

alter table atacc1 add constraint atacc1_chk check(b = 1) not valid;

alter table atacc1 validate constraint atacc1_chk, alter a type int;

drop table atacc1;

create table atacc1 (a bigint, b int);

insert into atacc1 values(1,2);

alter table atacc1 add constraint atacc1_chk check(b = 1) not valid;

alter table atacc1 validate constraint atacc1_chk, alter a type int;

drop table atacc1;

create table atacc1 ( test int, test2 int);

alter table atacc1 add constraint atacc_test1 primary key (test, test2);

alter table atacc1 add constraint atacc_test2 primary key (test);

insert into atacc1 (test,test2) values (4,4);

insert into atacc1 (test,test2) values (4,4);

insert into atacc1 (test,test2) values (NULL,3);

insert into atacc1 (test,test2) values (3, NULL);

insert into atacc1 (test,test2) values (NULL,NULL);

insert into atacc1 (test,test2) values (4,5);

insert into atacc1 (test,test2) values (5,4);

insert into atacc1 (test,test2) values (5,5);

drop table atacc1;

create table atacc1 (test int, test2 int, primary key(test));

insert into atacc1 (test2, test) values (3, 3);

insert into atacc1 (test2, test) values (2, 3);

insert into atacc1 (test2, test) values (1, NULL);

drop table atacc1;

alter table pg_class alter column relname drop not null;

alter table pg_class alter relname set not null;

alter table non_existent alter column bar set not null;

alter table non_existent alter column bar drop not null;

create table atacc1 (test int not null);

alter table atacc1 add constraint "atacc1_pkey" primary key (test);

alter table atacc1 alter column test drop not null;

alter table atacc1 drop constraint "atacc1_pkey";

alter table atacc1 alter column test drop not null;

insert into atacc1 values (null);

alter table atacc1 alter test set not null;

delete from atacc1;

alter table atacc1 alter test set not null;

alter table atacc1 alter bar set not null;

alter table atacc1 alter bar drop not null;

create view myview as select * from atacc1;

alter table myview alter column test drop not null;

alter table myview alter column test set not null;

drop view myview;

drop table atacc1;

create table atacc1 (test_a int, test_b int);

insert into atacc1 values (null, 1);

alter table atacc1 add constraint atacc1_constr_or check(test_a is not null or test_b < 10);

alter table atacc1 alter test_a set not null;

alter table atacc1 drop constraint atacc1_constr_or;

alter table atacc1 add constraint atacc1_constr_invalid check(test_a is not null) not valid;

alter table atacc1 alter test_a set not null;

alter table atacc1 drop constraint atacc1_constr_invalid;

update atacc1 set test_a = 1;

alter table atacc1 add constraint atacc1_constr_a_valid check(test_a is not null);

alter table atacc1 alter test_a set not null;

delete from atacc1;

insert into atacc1 values (2, null);

alter table atacc1 alter test_a drop not null;

alter table atacc1 alter test_a set not null, alter test_b set not null;

alter table atacc1 alter test_b set not null, alter test_a set not null;

update atacc1 set test_b = 1;

alter table atacc1 alter test_b set not null, alter test_a set not null;

alter table atacc1 alter test_a drop not null, alter test_b drop not null;

alter table atacc1 add constraint atacc1_constr_b_valid check(test_b is not null);

alter table atacc1 alter test_b set not null, alter test_a set not null;

drop table atacc1;

CREATE TABLE atnnparted (id int, col1 int) PARTITION BY LIST (id);

CREATE TABLE atnnpart1 (col1 int, id int);

ALTER TABLE atnnpart1 ADD PRIMARY KEY (id);

ALTER TABLE atnnparted ATTACH PARTITION atnnpart1 FOR VALUES IN ('1');

BEGIN;

ALTER TABLE atnnparted VALIDATE CONSTRAINT dummy_constr;

ROLLBACK;

create table parent (a int);

create table child (b varchar(255)) inherits (parent);

alter table parent alter a set not null;

insert into parent values (NULL);

insert into child (a, b) values (NULL, 'foo');

alter table parent alter a drop not null;

insert into parent values (NULL);

insert into child (a, b) values (NULL, 'foo');

alter table only parent alter a set not null;

alter table child alter a set not null;

drop table child;

drop table parent;

create table def_test (
	c1	int4 default 5,
	c2	text default 'initial_default'
);

insert into def_test default values;

alter table def_test alter column c1 drop default;

insert into def_test default values;

alter table def_test alter column c2 drop default;

insert into def_test default values;

alter table def_test alter column c1 set default 10;

alter table def_test alter column c2 set default 'new_default';

insert into def_test default values;

select * from def_test;

alter table def_test alter column c1 set default 'wrong_datatype';

alter table def_test alter column c2 set default 20;

alter table def_test alter column c3 set default 30;

create view def_view_test as select * from def_test;

select new.*;

insert into def_view_test default values;

alter table def_view_test alter column c1 set default 45;

insert into def_view_test default values;

alter table def_view_test alter column c2 set default 'view_default';

insert into def_view_test default values;

select * from def_view_test;

drop rule def_view_test_ins on def_view_test;

drop view def_view_test;

drop table def_test;

alter table pg_class drop column relname;

alter table nosuchtable drop column bar;

create table atacc1 (a int4 not null, b int4, c int4 not null, d int4);

insert into atacc1 values (1, 2, 3, 4);

alter table atacc1 drop a;

alter table atacc1 drop a;

select * from atacc1;

select * from atacc1 order by a;

select * from atacc1 order by "........pg.dropped.1........";

select * from atacc1 group by a;

select * from atacc1 group by "........pg.dropped.1........";

select atacc1.* from atacc1;

select a from atacc1;

select atacc1.a from atacc1;

select b,c,d from atacc1;

select a,b,c,d from atacc1;

select * from atacc1 where a = 1;

select "........pg.dropped.1........" from atacc1;

select atacc1."........pg.dropped.1........" from atacc1;

select "........pg.dropped.1........",b,c,d from atacc1;

select * from atacc1 where "........pg.dropped.1........" = 1;

update atacc1 set a = 3;

update atacc1 set b = 2 where a = 3;

update atacc1 set "........pg.dropped.1........" = 3;

update atacc1 set b = 2 where "........pg.dropped.1........" = 3;

insert into atacc1 values (10, 11, 12, 13);

insert into atacc1 values (default, 11, 12, 13);

insert into atacc1 values (11, 12, 13);

insert into atacc1 (a) values (10);

insert into atacc1 (a) values (default);

insert into atacc1 (a,b,c,d) values (10,11,12,13);

insert into atacc1 (a,b,c,d) values (default,11,12,13);

insert into atacc1 (b,c,d) values (11,12,13);

insert into atacc1 ("........pg.dropped.1........") values (10);

insert into atacc1 ("........pg.dropped.1........") values (default);

insert into atacc1 ("........pg.dropped.1........",b,c,d) values (10,11,12,13);

insert into atacc1 ("........pg.dropped.1........",b,c,d) values (default,11,12,13);

delete from atacc1 where a = 3;

delete from atacc1 where "........pg.dropped.1........" = 3;

delete from atacc1;

alter table atacc1 drop bar;

alter table atacc1 SET WITHOUT OIDS;
