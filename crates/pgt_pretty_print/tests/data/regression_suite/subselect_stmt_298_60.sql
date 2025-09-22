with x as (select * from (select f1 from subselect_tbl) ss)
select * from x where f1 = 1;
