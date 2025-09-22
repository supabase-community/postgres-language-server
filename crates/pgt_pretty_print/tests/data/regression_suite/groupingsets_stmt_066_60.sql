select * from gstest1 group by grouping sets((a,b,v),(v)) order by v,b,a;
