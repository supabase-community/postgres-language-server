select a, d, grouping(a,b,c)
    from gstest3
   group by grouping sets ((a,b), (a,c));
