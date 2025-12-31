create table spgist_point_tbl(id int4, p point);

create index spgist_point_idx on spgist_point_tbl using spgist(p) with (fillfactor = 75);

insert into spgist_point_tbl (id, p)
select g, point(g*10, g*10) from generate_series(1, 10) g;

delete from spgist_point_tbl where id < 5;

vacuum spgist_point_tbl;

insert into spgist_point_tbl (id, p)
select g,      point(g*10, g*10) from generate_series(1, 10000) g;

insert into spgist_point_tbl (id, p)
select g+100000, point(g*10+1, g*10+1) from generate_series(1, 10000) g;

delete from spgist_point_tbl where id % 2 = 1;

delete from spgist_point_tbl where id < 10000;

vacuum spgist_point_tbl;

create table spgist_box_tbl(id serial, b box);

insert into spgist_box_tbl(b)
select box(point(i,j),point(i+s,j+s))
  from generate_series(1,100,5) i,
       generate_series(1,100,5) j,
       generate_series(1,10) s;

create index spgist_box_idx on spgist_box_tbl using spgist (b);

select count(*)
  from (values (point(5,5)),(point(8,8)),(point(12,12))) v(p)
 where exists(select * from spgist_box_tbl b where b.b && box(v.p,v.p));

create table spgist_text_tbl(id int4, t text);

create index spgist_text_idx on spgist_text_tbl using spgist(t);

insert into spgist_text_tbl (id, t)
select g, 'f' || repeat('o', 100) || g from generate_series(1, 10000) g
union all
select g, 'baaaaaaaaaaaaaar' || g from generate_series(1, 1000) g;

insert into spgist_text_tbl (id, t)
select -g, 'f' || repeat('o', 100-g) || 'surprise' from generate_series(1, 100) g;

create index spgist_point_idx2 on spgist_point_tbl using spgist(p) with (fillfactor = 9);

create index spgist_point_idx2 on spgist_point_tbl using spgist(p) with (fillfactor = 101);

alter index spgist_point_idx set (fillfactor = 90);

reindex index spgist_point_idx;

create domain spgist_text as varchar;

create table spgist_domain_tbl (f1 spgist_text);

create index spgist_domain_idx on spgist_domain_tbl using spgist(f1);

insert into spgist_domain_tbl values('fee'), ('fi'), ('fo'), ('fum');

select * from spgist_domain_tbl where f1 = 'fo';

select * from spgist_domain_tbl where f1 = 'fo';

create unlogged table spgist_unlogged_tbl(id serial, b box);

create index spgist_unlogged_idx on spgist_unlogged_tbl using spgist (b);

insert into spgist_unlogged_tbl(b)
select box(point(i,j))
  from generate_series(1,100,5) i,
       generate_series(1,10,5) j;
