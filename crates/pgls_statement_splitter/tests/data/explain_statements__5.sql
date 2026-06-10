explain select 1 from contact;

explain analyze select * from contact where id = 1;

explain (analyze, buffers, format json) update contact set name = 'x' where id = 1;

explain with x as (select 1) select * from x;

select 1;
