select * from gtest32 t group by grouping sets (a, b, c, d, e) having c = 20;
