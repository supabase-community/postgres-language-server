select a, b, grouping(a,b), array_agg(v order by v)
    from gstest1 group by cube(a,b);
