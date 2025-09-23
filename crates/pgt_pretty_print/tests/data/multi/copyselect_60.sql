create table test1 (id serial, t text);

insert into test1 (t) values ('a');

insert into test1 (t) values ('b');

insert into test1 (t) values ('c');

insert into test1 (t) values ('d');

insert into test1 (t) values ('e');

create table test2 (id serial, t text);

insert into test2 (t) values ('A');

insert into test2 (t) values ('B');

insert into test2 (t) values ('C');

insert into test2 (t) values ('D');

insert into test2 (t) values ('E');

create view v_test1
as select 'v_'||t from test1;

copy test1 to stdout;

copy v_test1 to stdout;

copy (select t from test1 where id=1) to stdout;

copy (select t from test1 where id=3 for update) to stdout;

copy (select t into temp test3 from test1 where id=3) to stdout;

copy (select * from test1 join test2 using (id)) to stdout;

copy (select t from test1 where id = 1 UNION select * from v_test1 ORDER BY 1) to stdout;

copy (select * from (select t from test1 where id = 1 UNION select * from v_test1 ORDER BY 1) t1) to stdout;

copy (select t from test1 where id = 1) to stdout csv header force quote t;

drop table test2;

drop view v_test1;

drop table test1;

select 1/0;

copy (select 1) to stdout;

select 4;

create table test3 (c int);

select 1;

select * from test3;

drop table test3;
