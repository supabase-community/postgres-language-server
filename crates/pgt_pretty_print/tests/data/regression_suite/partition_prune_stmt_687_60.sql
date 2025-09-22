select
  'select tableoid::regclass,* from hp_prefix_test where ' ||
  string_agg(c.colname || case when g.s & (1 << c.colpos) = 0 then ' is null' else ' = ' || (colpos+1)::text end, ' and ' order by c.colpos)
from (values('a',0),('b',1),('c',2),('d',3)) c(colname, colpos), generate_Series(0,15) g(s)
group by g.s
order by g.s;
