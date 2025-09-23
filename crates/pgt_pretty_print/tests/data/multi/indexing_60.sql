create table idxpart (a int, b int, c text) partition by range (a);

create index idxpart_idx on idxpart (a);

select relhassubclass from pg_class where relname = 'idxpart_idx';

select indexdef from pg_indexes where indexname like 'idxpart_idx%';

drop index idxpart_idx;

create table idxpart1 partition of idxpart for values from (0) to (10);

create table idxpart2 partition of idxpart for values from (10) to (100)
	partition by range (b);

create table idxpart21 partition of idxpart2 for values from (0) to (100);

create index idxpart_idx on only idxpart(a);

select relhassubclass from pg_class where relname = 'idxpart_idx';

drop index idxpart_idx;

create index on idxpart (a);

select relname, relkind, relhassubclass, inhparent::regclass
    from pg_class left join pg_index ix on (indexrelid = oid)
	left join pg_inherits on (ix.indexrelid = inhrelid)
	where relname like 'idxpart%' order by relname;

drop table idxpart;

create table idxpart (a int, b int, c text) partition by range (a);

create table idxpart1 partition of idxpart for values from (0) to (10);

create index concurrently on idxpart (a);

drop table idxpart;

CREATE TABLE idxpart (col1 INT) PARTITION BY RANGE (col1);

CREATE INDEX ON idxpart (col1);

CREATE TABLE idxpart_two (col2 INT);

SELECT col2 FROM idxpart_two fk LEFT OUTER JOIN idxpart pk ON (col1 = col2);

DROP table idxpart, idxpart_two;

CREATE TABLE idxpart (a INT, b TEXT, c INT) PARTITION BY RANGE(a);

CREATE TABLE idxpart1 PARTITION OF idxpart FOR VALUES FROM (MINVALUE) TO (MAXVALUE);

CREATE INDEX partidx_abc_idx ON idxpart (a, b, c);

INSERT INTO idxpart (a, b, c) SELECT i, i, i FROM generate_series(1, 50) i;

ALTER TABLE idxpart ALTER COLUMN c TYPE numeric;

DROP TABLE idxpart;

create table idxpart (a int, b int, c text) partition by range (a);

create index idxparti on idxpart (a);

create index idxparti2 on idxpart (b, c);

create table idxpart1 (like idxpart);

alter table idxpart attach partition idxpart1 for values from (0) to (10);

create index idxpart_c on only idxpart (c);

create index idxpart1_c on idxpart1 (c);

alter table idxpart_c attach partition idxpart1_c for values from (10) to (20);

alter index idxpart_c attach partition idxpart1_c;

select relname, relpartbound from pg_class
  where relname in ('idxpart_c', 'idxpart1_c')
  order by relname;

alter table idxpart_c detach partition idxpart1_c;

drop table idxpart;

create table idxpart (a int, b int) partition by range (a, b);

create table idxpart1 partition of idxpart for values from (0, 0) to (10, 10);

create index on idxpart1 (a, b);

create index on idxpart (a, b);

select relname, relkind, relhassubclass, inhparent::regclass
    from pg_class left join pg_index ix on (indexrelid = oid)
	left join pg_inherits on (ix.indexrelid = inhrelid)
	where relname like 'idxpart%' order by relname;

drop table idxpart;

create table idxpart (a int) partition by range (a);

create index on idxpart (a);

create table idxpart1 partition of idxpart for values from (0) to (10);

drop index idxpart1_a_idx;

drop index concurrently idxpart_a_idx;

drop index idxpart_a_idx;

select relname, relkind from pg_class
  where relname like 'idxpart%' order by relname;

create index on idxpart (a);

drop table idxpart1;

select relname, relkind from pg_class
  where relname like 'idxpart%' order by relname;

drop table idxpart;

create temp table idxpart_temp (a int) partition by range (a);

create index on idxpart_temp(a);

create temp table idxpart1_temp partition of idxpart_temp
  for values from (0) to (10);

drop index idxpart1_temp_a_idx;

drop index concurrently idxpart_temp_a_idx;

select relname, relkind from pg_class
  where relname like 'idxpart_temp%' order by relname;

drop table idxpart_temp;

create table idxpart (a int, b int) partition by range (a, b);

create table idxpart1 partition of idxpart for values from (0, 0) to (10, 10);

create index idxpart_a_b_idx on only idxpart (a, b);

create index idxpart1_a_b_idx on idxpart1 (a, b);

create index idxpart1_tst1 on idxpart1 (b, a);

create index idxpart1_tst2 on idxpart1 using hash (a);

create index idxpart1_tst3 on idxpart1 (a, b) where a > 10;

alter index idxpart attach partition idxpart1;

alter index idxpart_a_b_idx attach partition idxpart1;

alter index idxpart_a_b_idx attach partition idxpart_a_b_idx;

alter index idxpart_a_b_idx attach partition idxpart1_b_idx;

alter index idxpart_a_b_idx attach partition idxpart1_tst1;

alter index idxpart_a_b_idx attach partition idxpart1_tst2;

alter index idxpart_a_b_idx attach partition idxpart1_tst3;

alter index idxpart_a_b_idx attach partition idxpart1_a_b_idx;

alter index idxpart_a_b_idx attach partition idxpart1_a_b_idx;

create index idxpart1_2_a_b on idxpart1 (a, b);

alter index idxpart_a_b_idx attach partition idxpart1_2_a_b;

drop table idxpart;

select indexrelid::regclass, indrelid::regclass
  from pg_index where indexrelid::regclass::text like 'idxpart%';

create table idxpart (a int, b int) partition by range (a);

create table idxpart1 (a int, b int);

create index on idxpart1 using hash (a);

create index on idxpart1 (a) where b > 1;

create index on idxpart1 ((a + 0));

create index on idxpart1 (a, a);

create index on idxpart (a);

alter table idxpart attach partition idxpart1 for values from (0) to (1000);

drop table idxpart;

create table idxpart (a int) partition by range (a);

create table idxpart1 partition of idxpart for values from (0) to (100);

create table idxpart2 partition of idxpart for values from (100) to (1000)
  partition by range (a);

create table idxpart21 partition of idxpart2 for values from (100) to (200);

create table idxpart22 partition of idxpart2 for values from (200) to (300);

create index on idxpart22 (a);

create index on only idxpart2 (a);

create index on idxpart (a);

select indexrelid::regclass, indrelid::regclass, inhparent::regclass
  from pg_index idx left join pg_inherits inh on (idx.indexrelid = inh.inhrelid)
where indexrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

alter index idxpart2_a_idx attach partition idxpart22_a_idx;

select indexrelid::regclass, indrelid::regclass, inhparent::regclass
  from pg_index idx left join pg_inherits inh on (idx.indexrelid = inh.inhrelid)
where indexrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

alter index idxpart2_a_idx attach partition idxpart22_a_idx;

create index on idxpart21 (a);

alter index idxpart2_a_idx attach partition idxpart21_a_idx;

drop table idxpart;

create table idxpart (a int, b int, c text, d bool) partition by range (a);

create index idxparti on idxpart (a);

create index idxparti2 on idxpart (b, c);

create table idxpart1 (like idxpart including indexes);

select relname, relkind, inhparent::regclass
    from pg_class left join pg_index ix on (indexrelid = oid)
	left join pg_inherits on (ix.indexrelid = inhrelid)
	where relname like 'idxpart%' order by relname;

alter table idxpart attach partition idxpart1 for values from (0) to (10);

select relname, relkind, inhparent::regclass
    from pg_class left join pg_index ix on (indexrelid = oid)
	left join pg_inherits on (ix.indexrelid = inhrelid)
	where relname like 'idxpart%' order by relname;

create index on idxpart1 ((a+b)) where d = true;

select relname, relkind, inhparent::regclass
    from pg_class left join pg_index ix on (indexrelid = oid)
	left join pg_inherits on (ix.indexrelid = inhrelid)
	where relname like 'idxpart%' order by relname;

create index idxparti3 on idxpart ((a+b)) where d = true;

select relname, relkind, inhparent::regclass
    from pg_class left join pg_index ix on (indexrelid = oid)
	left join pg_inherits on (ix.indexrelid = inhrelid)
	where relname like 'idxpart%' order by relname;

drop table idxpart;

create table idxpart (a int, b int) partition by range (a);

create table idxpart1 partition of idxpart for values from (1) to (1000) partition by range (a);

create table idxpart11 partition of idxpart1 for values from (1) to (100);

create index on only idxpart1 (a);

create index on only idxpart (a);

select relname, indisvalid from pg_class join pg_index on indexrelid = oid
   where relname like 'idxpart%' order by relname;

alter index idxpart_a_idx attach partition idxpart1_a_idx;

select relname, indisvalid from pg_class join pg_index on indexrelid = oid
   where relname like 'idxpart%' order by relname;

create index on idxpart11 (a);

alter index idxpart1_a_idx attach partition idxpart11_a_idx;

select relname, indisvalid from pg_class join pg_index on indexrelid = oid
   where relname like 'idxpart%' order by relname;

drop table idxpart;

create table idxpart (a int) partition by range (a);

create table idxpart1 (like idxpart);

create index on idxpart1 (a);

create index on idxpart (a);

create table idxpart2 (like idxpart);

alter table idxpart attach partition idxpart1 for values from (0000) to (1000);

alter table idxpart attach partition idxpart2 for values from (1000) to (2000);

create table idxpart3 partition of idxpart for values from (2000) to (3000);

select relname, relkind from pg_class where relname like 'idxpart%' order by relname;

alter table idxpart detach partition idxpart1;

alter table idxpart detach partition idxpart2;

alter table idxpart detach partition idxpart3;

drop index idxpart1_a_idx;

drop index idxpart2_a_idx;

drop index idxpart3_a_idx;

select relname, relkind from pg_class where relname like 'idxpart%' order by relname;

drop table idxpart, idxpart1, idxpart2, idxpart3;

select relname, relkind from pg_class where relname like 'idxpart%' order by relname;

create table idxpart (a int) partition by range (a);

create table idxpart1 (like idxpart);

create index on idxpart1 (a);

create index on idxpart (a);

create table idxpart2 (like idxpart);

alter table idxpart attach partition idxpart1 for values from (0000) to (1000);

alter table idxpart attach partition idxpart2 for values from (1000) to (2000);

create table idxpart3 partition of idxpart for values from (2000) to (3000);

select relname, relkind from pg_class where relname like 'idxpart%' order by relname;

alter table idxpart detach partition idxpart1;

alter table idxpart detach partition idxpart2;

alter table idxpart detach partition idxpart3;

drop index idxpart_a_idx;

select relname, relkind from pg_class where relname like 'idxpart%' order by relname;

drop table idxpart, idxpart1, idxpart2, idxpart3;

select relname, relkind from pg_class where relname like 'idxpart%' order by relname;

create table idxpart (a int, b int, c int) partition by range(a);

create index on idxpart(c);

create table idxpart1 partition of idxpart for values from (0) to (250);

create table idxpart2 partition of idxpart for values from (250) to (500);

alter table idxpart detach partition idxpart2;

alter table idxpart2 drop column c;

drop table idxpart, idxpart2;

create table idxpart (a int, b int) partition by range (a);

create table idxpart1 (like idxpart);

create index on idxpart1 ((a + b));

create index on idxpart ((a + b));

create table idxpart2 (like idxpart);

alter table idxpart attach partition idxpart1 for values from (0000) to (1000);

alter table idxpart attach partition idxpart2 for values from (1000) to (2000);

create table idxpart3 partition of idxpart for values from (2000) to (3000);

select relname as child, inhparent::regclass as parent, pg_get_indexdef as childdef
  from pg_class join pg_inherits on inhrelid = oid,
  lateral pg_get_indexdef(pg_class.oid)
  where relkind in ('i', 'I') and relname like 'idxpart%' order by relname;

drop table idxpart;

create table idxpart (a text) partition by range (a);

create table idxpart1 (like idxpart);

create table idxpart2 (like idxpart);

create index on idxpart2 (a collate "POSIX");

create index on idxpart2 (a);

create index on idxpart2 (a collate "C");

alter table idxpart attach partition idxpart1 for values from ('aaa') to ('bbb');

alter table idxpart attach partition idxpart2 for values from ('bbb') to ('ccc');

create table idxpart3 partition of idxpart for values from ('ccc') to ('ddd');

create index on idxpart (a collate "C");

create table idxpart4 partition of idxpart for values from ('ddd') to ('eee');

select relname as child, inhparent::regclass as parent, pg_get_indexdef as childdef
  from pg_class left join pg_inherits on inhrelid = oid,
  lateral pg_get_indexdef(pg_class.oid)
  where relkind in ('i', 'I') and relname like 'idxpart%' order by relname;

drop table idxpart;

create table idxpart (a text) partition by range (a);

create table idxpart1 (like idxpart);

create table idxpart2 (like idxpart);

create index on idxpart2 (a);

alter table idxpart attach partition idxpart1 for values from ('aaa') to ('bbb');

alter table idxpart attach partition idxpart2 for values from ('bbb') to ('ccc');

create table idxpart3 partition of idxpart for values from ('ccc') to ('ddd');

create index on idxpart (a text_pattern_ops);

create table idxpart4 partition of idxpart for values from ('ddd') to ('eee');

select relname as child, inhparent::regclass as parent, pg_get_indexdef as childdef
  from pg_class left join pg_inherits on inhrelid = oid,
  lateral pg_get_indexdef(pg_class.oid)
  where relkind in ('i', 'I') and relname like 'idxpart%' order by relname;

drop index idxpart_a_idx;

create index on only idxpart (a text_pattern_ops);

alter index idxpart_a_idx attach partition idxpart2_a_idx;

drop table idxpart;

create table idxpart (col1 int, a int, col2 int, b int) partition by range (a);

create table idxpart1 (b int, col1 int, col2 int, col3 int, a int);

alter table idxpart drop column col1, drop column col2;

alter table idxpart1 drop column col1, drop column col2, drop column col3;

alter table idxpart attach partition idxpart1 for values from (0) to (1000);

create index idxpart_1_idx on only idxpart (b, a);

create index idxpart1_1_idx on idxpart1 (b, a);

create index idxpart1_1b_idx on idxpart1 (b);

create index idxpart_2_idx on only idxpart ((b + a)) where a > 1;

create index idxpart1_2_idx on idxpart1 ((b + a)) where a > 1;

create index idxpart1_2b_idx on idxpart1 ((a + b)) where a > 1;

create index idxpart1_2c_idx on idxpart1 ((b + a)) where b > 1;

alter index idxpart_1_idx attach partition idxpart1_1b_idx;

alter index idxpart_1_idx attach partition idxpart1_1_idx;

alter index idxpart_2_idx attach partition idxpart1_2b_idx;

alter index idxpart_2_idx attach partition idxpart1_2c_idx;

alter index idxpart_2_idx attach partition idxpart1_2_idx;

select relname as child, inhparent::regclass as parent, pg_get_indexdef as childdef
  from pg_class left join pg_inherits on inhrelid = oid,
  lateral pg_get_indexdef(pg_class.oid)
  where relkind in ('i', 'I') and relname like 'idxpart%' order by relname;

drop table idxpart;

create table idxpart (a int, b int, c text) partition by range (a);

create index idxparti on idxpart (a);

create index idxparti2 on idxpart (c, b);

create table idxpart1 (c text, a int, b int);

alter table idxpart attach partition idxpart1 for values from (0) to (10);

create table idxpart2 (c text, a int, b int);

create index on idxpart2 (a);

create index on idxpart2 (c, b);

alter table idxpart attach partition idxpart2 for values from (10) to (20);

select c.relname, pg_get_indexdef(indexrelid)
  from pg_class c join pg_index i on c.oid = i.indexrelid
  where indrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

drop table idxpart;

create table idxpart (col1 int, col2 int, a int, b int) partition by range (a);

create table idxpart1 (col2 int, b int, col1 int, a int);

create table idxpart2 (col1 int, col2 int, b int, a int);

alter table idxpart drop column col1, drop column col2;

alter table idxpart1 drop column col1, drop column col2;

alter table idxpart2 drop column col1, drop column col2;

create index on idxpart2 (abs(b));

alter table idxpart attach partition idxpart2 for values from (0) to (1);

create index on idxpart (abs(b));

create index on idxpart ((b + 1));

alter table idxpart attach partition idxpart1 for values from (1) to (2);

select c.relname, pg_get_indexdef(indexrelid)
  from pg_class c join pg_index i on c.oid = i.indexrelid
  where indrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

drop table idxpart;

create table idxpart (col1 int, a int, col3 int, b int) partition by range (a);

alter table idxpart drop column col1, drop column col3;

create table idxpart1 (col1 int, col2 int, col3 int, col4 int, b int, a int);

alter table idxpart1 drop column col1, drop column col2, drop column col3, drop column col4;

alter table idxpart attach partition idxpart1 for values from (0) to (1000);

create table idxpart2 (col1 int, col2 int, b int, a int);

create index on idxpart2 (a) where b > 1000;

alter table idxpart2 drop column col1, drop column col2;

alter table idxpart attach partition idxpart2 for values from (1000) to (2000);

create index on idxpart (a) where b > 1000;

select c.relname, pg_get_indexdef(indexrelid)
  from pg_class c join pg_index i on c.oid = i.indexrelid
  where indrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

drop table idxpart;

create table idxpart1 (drop_1 int, drop_2 int, col_keep int, drop_3 int);

alter table idxpart1 drop column drop_1;

alter table idxpart1 drop column drop_2;

alter table idxpart1 drop column drop_3;

create index on idxpart1 (col_keep);

create table idxpart (col_keep int) partition by range (col_keep);

create index on idxpart (col_keep);

alter table idxpart attach partition idxpart1 for values from (0) to (1000);

select attrelid::regclass, attname, attnum from pg_attribute
  where attrelid::regclass::text like 'idxpart%' and attnum > 0
  order by attrelid::regclass, attnum;

drop table idxpart;

create table idxpart(drop_1 int, drop_2 int, col_keep int, drop_3 int) partition by range (col_keep);

alter table idxpart drop column drop_1;

alter table idxpart drop column drop_2;

alter table idxpart drop column drop_3;

create table idxpart1 (col_keep int);

create index on idxpart1 (col_keep);

create index on idxpart (col_keep);

alter table idxpart attach partition idxpart1 for values from (0) to (1000);

select attrelid::regclass, attname, attnum from pg_attribute
  where attrelid::regclass::text like 'idxpart%' and attnum > 0
  order by attrelid::regclass, attnum;

drop table idxpart;

create table idxpart (a int primary key, b int) partition by range (a);

create table failpart partition of idxpart (b primary key) for values from (0) to (100);

drop table idxpart;

create table idxpart (a int) partition by range (a);

create table idxpart1pk partition of idxpart (a primary key) for values from (0) to (100);

drop table idxpart;

create table idxpart (a int unique, b int) partition by range (a, b);

create table idxpart (a int, b int unique) partition by range (a, b);

create table idxpart (a int primary key, b int) partition by range (b, a);

create table idxpart (a int, b int primary key) partition by range (b, a);

create table idxpart (a int, b int, c text, primary key  (a, b, c)) partition by range (b, c, a);

drop table idxpart;

create table idxpart (a int4range, exclude USING GIST (a with = )) partition by range (a);

drop table idxpart;

create table idxpart (a int4range, b int4range, exclude USING GIST (a with =, b with =)) partition by range (a, b);

drop table idxpart;

create table idxpart (a int4range, b int4range, exclude USING GIST (a with =, b with =)) partition by range (a);

drop table idxpart;

create table idxpart (a int4range, b int4range, exclude USING GIST (a with = )) partition by range (a, b);

create table idxpart (a int4range, exclude USING GIST (a with -|- )) partition by range (a);

create table idxpart (a int4range, b int4range, exclude USING GIST (a with =, b with &&)) partition by range (a);

drop table idxpart;

create table idxpart (a int4range, b int4range, c int4range, exclude USING GIST (b with =, c with &&)) partition by range (a);

create table idxpart (a int4range, b int4range, c int4range, exclude USING GIST (a with =, b with =, c with &&)) partition by range (a, b);

drop table idxpart;

create table idxpart (a int primary key, b int) partition by range ((b + a));

create table idxpart (a int unique, b int) partition by range ((b + a));

create table idxpart (a int, b int, c text) partition by range (a, b);

alter table idxpart add primary key (a);

alter table idxpart add primary key (a, b);

create table idxpart1 partition of idxpart for values from (0, 0) to (1000, 1000);

drop table idxpart;

create table idxpart (a int, b int) partition by range (a, b);

alter table idxpart add unique (a);

alter table idxpart add unique (b, a);

drop table idxpart;

create table idxpart (a int4range, b int4range) partition by range (a);

alter table idxpart add exclude USING GIST (a with =);

drop table idxpart;

create table idxpart (a int4range, b int4range) partition by range (a, b);

alter table idxpart add exclude USING GIST (a with =, b with =);

drop table idxpart;

create table idxpart (a int4range, b int4range) partition by range (a);

alter table idxpart add exclude USING GIST (a with =, b with =);

drop table idxpart;

create table idxpart (a int4range, b int4range) partition by range (a, b);

alter table idxpart add exclude USING GIST (a with =);

drop table idxpart;

create table idxpart (a int4range, b int4range) partition by range (a, b);

alter table idxpart add exclude USING GIST (a with -|-);

drop table idxpart;

create table idxpart (a int4range, b int4range) partition by range (a);

alter table idxpart add exclude USING GIST (a with =, b with &&);

drop table idxpart;

create table idxpart (a int4range, b int4range, c int4range) partition by range (a);

alter table idxpart add exclude USING GIST (b with =, c with &&);

drop table idxpart;

create table idxpart (a int4range, b int4range, c int4range) partition by range (a, b);

alter table idxpart add exclude USING GIST (a with =, b with =, c with &&);

drop table idxpart;

create table idxpart (a int, b int, primary key (a, b)) partition by range (a, b);

create table idxpart1 partition of idxpart for values from (1, 1) to (10, 10);

create table idxpart2 partition of idxpart for values from (10, 10) to (20, 20)
  partition by range (b);

create table idxpart21 partition of idxpart2 for values from (10) to (15);

create table idxpart22 partition of idxpart2 for values from (15) to (20);

create table idxpart3 (b int not null, a int not null);

alter table idxpart attach partition idxpart3 for values from (20, 20) to (30, 30);

select conname, contype, conrelid::regclass, conindid::regclass, conkey
  from pg_constraint where conrelid::regclass::text like 'idxpart%'
  order by conrelid::regclass::text, conname;

drop table idxpart;

create table idxpart (a int, b int, primary key (a)) partition by range (a);

create table idxpart2 partition of idxpart
for values from (0) to (1000) partition by range (b);

drop table idxpart;

create table idxpart (a int unique, b int) partition by range (a);

create table idxpart1 (a int not null, b int, unique (a, b))
  partition by range (a, b);

alter table idxpart attach partition idxpart1 for values from (1) to (1000);

DROP TABLE idxpart, idxpart1;

create table idxpart (a int, b int, primary key (a, b)) partition by range (a);

create table idxpart2 partition of idxpart for values from (0) to (1000) partition by range (b);

create table idxpart21 partition of idxpart2 for values from (0) to (1000);

select conname, contype, conrelid::regclass, conindid::regclass, conkey
  from pg_constraint where conrelid::regclass::text like 'idxpart%'
  order by conrelid::regclass::text, conname;

drop table idxpart;

create table idxpart (i int) partition by hash (i);

create table idxpart0 partition of idxpart (i) for values with (modulus 2, remainder 0);

create table idxpart1 partition of idxpart (i) for values with (modulus 2, remainder 1);

alter table idxpart0 add primary key(i);

alter table idxpart add primary key(i);

select indrelid::regclass, indexrelid::regclass, inhparent::regclass, indisvalid,
  conname, conislocal, coninhcount, connoinherit, convalidated
  from pg_index idx left join pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  left join pg_constraint con on (idx.indexrelid = con.conindid)
  where indrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

drop index idxpart0_pkey;

drop index idxpart1_pkey;

alter table idxpart0 drop constraint idxpart0_pkey;

alter table idxpart1 drop constraint idxpart1_pkey;

alter table idxpart drop constraint idxpart_pkey;

select indrelid::regclass, indexrelid::regclass, inhparent::regclass, indisvalid,
  conname, conislocal, coninhcount, connoinherit, convalidated
  from pg_index idx left join pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  left join pg_constraint con on (idx.indexrelid = con.conindid)
  where indrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

drop table idxpart;

CREATE TABLE idxpart (c1 INT PRIMARY KEY, c2 INT, c3 VARCHAR(10)) PARTITION BY RANGE(c1);

CREATE TABLE idxpart1 (LIKE idxpart);

ALTER TABLE idxpart1 ADD PRIMARY KEY (c1, c2);

ALTER TABLE idxpart ATTACH PARTITION idxpart1 FOR VALUES FROM (100) TO (200);

DROP TABLE idxpart, idxpart1;

create table idxpart (a int, b int, primary key (a)) partition by range (a);

create table idxpart1 (a int not null, b int) partition by range (a);

create table idxpart11 (a int not null, b int primary key);

alter table idxpart1 attach partition idxpart11 for values from (0) to (1000);

alter table idxpart attach partition idxpart1 for values from (0) to (10000);

drop table idxpart, idxpart1, idxpart11;

create table idxpart (a int) partition by range (a);

create table idxpart0 (like idxpart);

alter table idxpart0 add primary key (a);

alter table idxpart attach partition idxpart0 for values from (0) to (1000);

alter table only idxpart add primary key (a);

select indrelid::regclass, indexrelid::regclass, inhparent::regclass, indisvalid,
  conname, conislocal, coninhcount, connoinherit, convalidated
  from pg_index idx left join pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  left join pg_constraint con on (idx.indexrelid = con.conindid)
  where indrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

alter index idxpart_pkey attach partition idxpart0_pkey;

select indrelid::regclass, indexrelid::regclass, inhparent::regclass, indisvalid,
  conname, conislocal, coninhcount, connoinherit, convalidated
  from pg_index idx left join pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  left join pg_constraint con on (idx.indexrelid = con.conindid)
  where indrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

drop table idxpart;

create table idxpart (a int) partition by range (a);

create table idxpart0 (like idxpart);

alter table idxpart0 add unique (a);

alter table idxpart attach partition idxpart0 default;

alter table only idxpart add primary key (a);

alter table idxpart0 alter column a set not null;

alter table only idxpart add primary key (a);

alter index idxpart_pkey attach partition idxpart0_a_key;

drop table idxpart;

create table idxpart (a int, b int) partition by range (a);

create table idxpart1 (a int not null, b int);

create unique index on idxpart1 (a);

alter table idxpart add primary key (a);

alter table idxpart attach partition idxpart1 for values from (1) to (1000);

select indrelid::regclass, indexrelid::regclass, inhparent::regclass, indisvalid,
  conname, conislocal, coninhcount, connoinherit, convalidated
  from pg_index idx left join pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  left join pg_constraint con on (idx.indexrelid = con.conindid)
  where indrelid::regclass::text like 'idxpart%'
  order by indexrelid::regclass::text collate "C";

drop table idxpart;

create table idxpart (a int, b int) partition by range (a);

create table idxpart1 (a int not null, b int);

create unique index on idxpart1 (a);

alter table idxpart attach partition idxpart1 for values from (1) to (1000);

alter table only idxpart add primary key (a);

alter index idxpart_pkey attach partition idxpart1_a_idx;

drop table idxpart;

create table idxpart (a int, b text, primary key (a, b)) partition by range (a);

create table idxpart1 partition of idxpart for values from (0) to (100000);

create table idxpart2 (c int, like idxpart);

insert into idxpart2 (c, a, b) values (42, 572814, 'inserted first');

alter table idxpart2 drop column c;

create unique index on idxpart (a);

alter table idxpart attach partition idxpart2 for values from (100000) to (1000000);

insert into idxpart values (0, 'zero'), (42, 'life'), (2^16, 'sixteen');

insert into idxpart select 2^g, format('two to power of %s', g) from generate_series(15, 17) g;

insert into idxpart values (16, 'sixteen');

insert into idxpart (b, a) values ('one', 142857), ('two', 285714);

insert into idxpart select a * 2, b || b from idxpart where a between 2^16 and 2^19;

insert into idxpart values (572814, 'five');

insert into idxpart values (857142, 'six');

select tableoid::regclass, * from idxpart order by a;

drop table idxpart;

create table idxpart (a int, b text, c int[]) partition by range (a);

create table idxpart1 partition of idxpart for values from (0) to (100000);

set enable_seqscan to off;

create index idxpart_brin on idxpart using brin(b);

select * from idxpart where b = 'abcd';

drop index idxpart_brin;

create index idxpart_spgist on idxpart using spgist(b);

select * from idxpart where b = 'abcd';

drop index idxpart_spgist;

create index idxpart_gin on idxpart using gin(c);

select * from idxpart where c @> array[42];

drop index idxpart_gin;

reset enable_seqscan;

drop table idxpart;

create table idxpart (a int) partition by range (a);

create table idxpart1 partition of idxpart for values from (0) to (100);

create table idxpart2 partition of idxpart for values from (100) to (1000)
  partition by range (a);

create table idxpart21 partition of idxpart2 for values from (100) to (200);

create table idxpart22 partition of idxpart2 for values from (200) to (300);

create index on idxpart22 (a);

create index on only idxpart2 (a);

alter index idxpart2_a_idx attach partition idxpart22_a_idx;

create index on idxpart (a);

create table idxpart_another (a int, b int, primary key (a, b)) partition by range (a);

create table idxpart_another_1 partition of idxpart_another for values from (0) to (100);

create table idxpart3 (c int, b int, a int) partition by range (a);

alter table idxpart3 drop column b, drop column c;

create table idxpart31 partition of idxpart3 for values from (1000) to (1200);

create table idxpart32 partition of idxpart3 for values from (1200) to (1400);

alter table idxpart attach partition idxpart3 for values from (1000) to (2000);

create schema regress_indexing;

set search_path to regress_indexing;

create table pk (a int primary key) partition by range (a);

create table pk1 partition of pk for values from (0) to (1000);

create table pk2 (b int, a int);

alter table pk2 drop column b;

alter table pk2 alter a set not null;

alter table pk attach partition pk2 for values from (1000) to (2000);

create table pk3 partition of pk for values from (2000) to (3000);

create table pk4 (like pk);

alter table pk attach partition pk4 for values from (3000) to (4000);

create table pk5 (like pk) partition by range (a);

create table pk51 partition of pk5 for values from (4000) to (4500);

create table pk52 partition of pk5 for values from (4500) to (5000);

alter table pk attach partition pk5 for values from (4000) to (5000);

reset search_path;

create table covidxpart (a int, b int) partition by list (a);

create unique index on covidxpart (a) include (b);

create table covidxpart1 partition of covidxpart for values in (1);

create table covidxpart2 partition of covidxpart for values in (2);

insert into covidxpart values (1, 1);

insert into covidxpart values (1, 1);

create table covidxpart3 (b int, c int, a int);

alter table covidxpart3 drop c;

alter table covidxpart attach partition covidxpart3 for values in (3);

insert into covidxpart values (3, 1);

insert into covidxpart values (3, 1);

create table covidxpart4 (b int, a int);

create unique index on covidxpart4 (a) include (b);

create unique index on covidxpart4 (a);

alter table covidxpart attach partition covidxpart4 for values in (4);

insert into covidxpart values (4, 1);

insert into covidxpart values (4, 1);

create unique index on covidxpart (b) include (a);

create table parted_pk_detach_test (a int primary key) partition by list (a);

create table parted_pk_detach_test1 partition of parted_pk_detach_test for values in (1);

alter table parted_pk_detach_test1 drop constraint parted_pk_detach_test1_pkey;

alter table parted_pk_detach_test detach partition parted_pk_detach_test1;

alter table parted_pk_detach_test1 drop constraint parted_pk_detach_test1_pkey;

drop table parted_pk_detach_test, parted_pk_detach_test1;

create table parted_uniq_detach_test (a int unique) partition by list (a);

create table parted_uniq_detach_test1 partition of parted_uniq_detach_test for values in (1);

alter table parted_uniq_detach_test1 drop constraint parted_uniq_detach_test1_a_key;

alter table parted_uniq_detach_test detach partition parted_uniq_detach_test1;

alter table parted_uniq_detach_test1 drop constraint parted_uniq_detach_test1_a_key;

drop table parted_uniq_detach_test, parted_uniq_detach_test1;

create table parted_index_col_drop(a int, b int, c int)
  partition by list (a);

create table parted_index_col_drop1 partition of parted_index_col_drop
  for values in (1) partition by list (a);

create table parted_index_col_drop2 partition of parted_index_col_drop
  for values in (2) partition by list (a);

create table parted_index_col_drop11 partition of parted_index_col_drop1
  for values in (1);

create index on parted_index_col_drop (b);

create index on parted_index_col_drop (c);

create index on parted_index_col_drop (b, c);

alter table parted_index_col_drop drop column c;

drop table parted_index_col_drop;

create table parted_inval_tab (a int) partition by range (a);

create index parted_inval_idx on parted_inval_tab (a);

create table parted_inval_tab_1 (a int) partition by range (a);

create table parted_inval_tab_1_1 partition of parted_inval_tab_1
  for values from (0) to (10);

create table parted_inval_tab_1_2 partition of parted_inval_tab_1
  for values from (10) to (20);

create index parted_inval_ixd_1 on only parted_inval_tab_1 (a);

alter table parted_inval_tab attach partition parted_inval_tab_1
  for values from (1) to (100);

select indexrelid::regclass, indisvalid,
       indrelid::regclass, inhparent::regclass
  from pg_index idx left join
       pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  where indexrelid::regclass::text like 'parted_inval%'
  order by indexrelid::regclass::text collate "C";

drop table parted_inval_tab;

create table parted_isvalid_tab (a int, b int) partition by range (a);

create table parted_isvalid_tab_1 partition of parted_isvalid_tab
  for values from (1) to (10) partition by range (a);

create table parted_isvalid_tab_2 partition of parted_isvalid_tab
  for values from (10) to (20) partition by range (a);

create table parted_isvalid_tab_11 partition of parted_isvalid_tab_1
  for values from (1) to (5);

create table parted_isvalid_tab_12 partition of parted_isvalid_tab_1
  for values from (5) to (10);

insert into parted_isvalid_tab_11 values (1, 0);

create index concurrently parted_isvalid_idx_11 on parted_isvalid_tab_11 ((a/b));

create index parted_isvalid_idx on parted_isvalid_tab ((a/b));

select indexrelid::regclass, indisvalid,
       indrelid::regclass, inhparent::regclass
  from pg_index idx left join
       pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  where indexrelid::regclass::text like 'parted_isvalid%'
  order by indexrelid::regclass::text collate "C";

drop table parted_isvalid_tab;

begin;

create table parted_replica_tab (id int not null) partition by range (id);

create table parted_replica_tab_1 partition of parted_replica_tab
  for values from (1) to (10) partition by range (id);

create table parted_replica_tab_11 partition of parted_replica_tab_1
  for values from (1) to (5);

create unique index parted_replica_idx
  on only parted_replica_tab using btree (id);

create unique index parted_replica_idx_1
  on only parted_replica_tab_1 using btree (id);

alter table only parted_replica_tab_1 replica identity
  using index parted_replica_idx_1;

create unique index parted_replica_idx_11 on parted_replica_tab_11 USING btree (id);

select indexrelid::regclass, indisvalid, indisreplident,
       indrelid::regclass, inhparent::regclass
  from pg_index idx left join
       pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  where indexrelid::regclass::text like 'parted_replica%'
  order by indexrelid::regclass::text collate "C";

alter index parted_replica_idx ATTACH PARTITION parted_replica_idx_1;

select indexrelid::regclass, indisvalid, indisreplident,
       indrelid::regclass, inhparent::regclass
  from pg_index idx left join
       pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  where indexrelid::regclass::text like 'parted_replica%'
  order by indexrelid::regclass::text collate "C";

alter index parted_replica_idx_1 ATTACH PARTITION parted_replica_idx_11;

alter table only parted_replica_tab_1 replica identity
  using index parted_replica_idx_1;

commit;

select indexrelid::regclass, indisvalid, indisreplident,
       indrelid::regclass, inhparent::regclass
  from pg_index idx left join
       pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  where indexrelid::regclass::text like 'parted_replica%'
  order by indexrelid::regclass::text collate "C";

drop table parted_replica_tab;

create table test_pg_index_toast_table (a int);

create or replace function test_pg_index_toast_func (a int, b int[])
  returns bool as $$ select true $$ language sql immutable;

select array_agg(n) b from generate_series(1, 10000) n ;

create index concurrently test_pg_index_toast_index
  on test_pg_index_toast_table (test_pg_index_toast_func(a, 'b'));

reindex index concurrently test_pg_index_toast_index;

drop index concurrently test_pg_index_toast_index;

create index test_pg_index_toast_index
  on test_pg_index_toast_table (test_pg_index_toast_func(a, 'b'));

reindex index test_pg_index_toast_index;

drop index test_pg_index_toast_index;

drop function test_pg_index_toast_func;

drop table test_pg_index_toast_table;
