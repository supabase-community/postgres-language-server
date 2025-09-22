select grouping((select t1.v from gstest5 t2 where id = t1.id)),
       (select t1.v from gstest5 t2 where id = t1.id) as s,
       case when grouping((select t1.v from gstest5 t2 where id = t1.id)) = 0
            then (select t1.v from gstest5 t2 where id = t1.id)
            else null end as o
from gstest5 t1
group by grouping sets(v, s)
order by o nulls first;
