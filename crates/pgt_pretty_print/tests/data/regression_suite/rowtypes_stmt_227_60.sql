with cte(c) as materialized (select row(1, 2)),
     cte2(c) as (select * from cte)
select (c).f1 from cte2 as t
where false;
