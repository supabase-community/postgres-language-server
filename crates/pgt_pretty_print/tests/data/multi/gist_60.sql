create table gist_point_tbl(id int4, p point);

create index gist_pointidx on gist_point_tbl using gist(p);

create index gist_pointidx2 on gist_point_tbl using gist(p) with (buffering = on, fillfactor=50);

create index gist_pointidx3 on gist_point_tbl using gist(p) with (buffering = off);

create index gist_pointidx4 on gist_point_tbl using gist(p) with (buffering = auto);

drop index gist_pointidx2, gist_pointidx3, gist_pointidx4;

create index gist_pointidx5 on gist_point_tbl using gist(p) with (buffering = invalid_value);

create index gist_pointidx5 on gist_point_tbl using gist(p) with (fillfactor=9);

create index gist_pointidx5 on gist_point_tbl using gist(p) with (fillfactor=101);

insert into gist_point_tbl (id, p)
select g,        point(g*10, g*10) from generate_series(1, 10000) g;

insert into gist_point_tbl (id, p)
select g+100000, point(g*10+1, g*10+1) from generate_series(1, 10000) g;

delete from gist_point_tbl where id % 2 = 1;

delete from gist_point_tbl where id > 5000;

vacuum analyze gist_point_tbl;

alter index gist_pointidx SET (fillfactor = 40);

reindex index gist_pointidx;

create table gist_tbl (b box, p point, c circle);

insert into gist_tbl
select box(point(0.05*i, 0.05*i), point(0.05*i, 0.05*i)),
       point(0.05*i, 0.05*i),
       circle(point(0.05*i, 0.05*i), 1.0)
from generate_series(0,10000) as i;

vacuum analyze gist_tbl;

set enable_seqscan=off;

set enable_bitmapscan=off;

set enable_indexonlyscan=on;

create index gist_tbl_point_index on gist_tbl using gist (p);

select p from gist_tbl where p <@ box(point(0,0), point(0.5, 0.5));

select p from gist_tbl where p <@ box(point(0,0), point(0.5, 0.5));

select p from gist_tbl where p <@ box(point(0,0), point(0.5, 0.5))
order by p <-> point(0.201, 0.201);

select p from gist_tbl where p <@ box(point(0,0), point(0.5, 0.5))
order by p <-> point(0.201, 0.201);

select p from gist_tbl where p <@ box(point(0,0), point(0.5, 0.5))
order by point(0.101, 0.101) <-> p;

select p from gist_tbl where p <@ box(point(0,0), point(0.5, 0.5))
order by point(0.101, 0.101) <-> p;

select p from
  (values (box(point(0,0), point(0.5,0.5))),
          (box(point(0.5,0.5), point(0.75,0.75))),
          (box(point(0.8,0.8), point(1.0,1.0)))) as v(bb)
cross join lateral
  (select p from gist_tbl where p <@ bb order by p <-> bb[0] limit 2) ss;

select p from
  (values (box(point(0,0), point(0.5,0.5))),
          (box(point(0.5,0.5), point(0.75,0.75))),
          (box(point(0.8,0.8), point(1.0,1.0)))) as v(bb)
cross join lateral
  (select p from gist_tbl where p <@ bb order by p <-> bb[0] limit 2) ss;

drop index gist_tbl_point_index;

create index gist_tbl_box_index on gist_tbl using gist (b);

select b from gist_tbl where b <@ box(point(5,5), point(6,6));

select b from gist_tbl where b <@ box(point(5,5), point(6,6));

select b from gist_tbl where b <@ box(point(5,5), point(6,6))
order by b <-> point(5.2, 5.91);

select b from gist_tbl where b <@ box(point(5,5), point(6,6))
order by b <-> point(5.2, 5.91);

select b from gist_tbl where b <@ box(point(5,5), point(6,6))
order by point(5.2, 5.91) <-> b;

select b from gist_tbl where b <@ box(point(5,5), point(6,6))
order by point(5.2, 5.91) <-> b;

drop index gist_tbl_box_index;

create index gist_tbl_multi_index on gist_tbl using gist (p, c);

select p, c from gist_tbl
where p <@ box(point(5,5), point(6, 6));

select b, p from gist_tbl
where b <@ box(point(4.5, 4.5), point(5.5, 5.5))
and p <@ box(point(5,5), point(6, 6));

drop index gist_tbl_multi_index;

create index gist_tbl_multi_index on gist_tbl using gist (circle(p,1), p);

select circle(p,1) from gist_tbl
where p <@ box(point(5, 5), point(5.3, 5.3));

select circle(p,1) from gist_tbl
where p <@ box(point(5, 5), point(5.3, 5.3));

select p from gist_tbl where circle(p,1) @> circle(point(0,0),0.95);

select p from gist_tbl where circle(p,1) @> circle(point(0,0),0.95);

select count(*) from gist_tbl;

select count(*) from gist_tbl;

select p from gist_tbl order by circle(p,1) <-> point(0,0) limit 1;

select p from gist_tbl order by circle(p,1) <-> point(0,0) limit 1;

create index gist_tbl_box_index_forcing_buffering on gist_tbl using gist (p)
  with (buffering=on, fillfactor=50);

reset enable_seqscan;

reset enable_bitmapscan;

reset enable_indexonlyscan;

drop table gist_tbl;

create unlogged table gist_tbl (b box);

create index gist_tbl_box_index on gist_tbl using gist (b);

insert into gist_tbl
  select box(point(0.05*i, 0.05*i)) from generate_series(0,10) as i;

drop table gist_tbl;
