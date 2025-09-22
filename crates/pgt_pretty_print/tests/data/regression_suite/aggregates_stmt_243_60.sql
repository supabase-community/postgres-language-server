select array_agg(distinct a)
  from (values (1),(2),(1),(3),(null),(2)) v(a);
