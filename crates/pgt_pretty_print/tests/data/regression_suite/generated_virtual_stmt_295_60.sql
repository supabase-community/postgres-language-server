select t1.a from gtest32 t1 left join gtest32 t2 on t1.a = t2.a
where coalesce(t2.b, 1) = 2 or t1.a is null;
