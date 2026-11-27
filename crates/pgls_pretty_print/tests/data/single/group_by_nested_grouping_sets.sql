select sum(c) from t group by grouping sets (f(c), grouping sets ((cube (c)))) order by 1 desc;
