select * from rangep where b IN((select 1),(select 2)) order by a;
