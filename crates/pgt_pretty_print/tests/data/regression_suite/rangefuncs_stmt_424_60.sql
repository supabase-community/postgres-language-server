with a(b) as (values (row(1,2,3)))
select * from a, coalesce(b) as c(d int, e int);
