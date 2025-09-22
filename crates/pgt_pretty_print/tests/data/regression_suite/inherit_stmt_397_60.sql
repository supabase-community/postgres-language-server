select 1 - id as c from
(select id from matest3 t1 union all select id * 2 from matest3 t2) ss
order by c;
