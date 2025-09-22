select 1 from gtest32 t1 where exists
  (select 1 from gtest32 t2 where t1.a > t2.a and t2.b = 2);
