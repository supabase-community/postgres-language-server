select array_agg(a order by a)
  from (values (1,4),(2,3),(3,1),(4,2)) v(a,b);
