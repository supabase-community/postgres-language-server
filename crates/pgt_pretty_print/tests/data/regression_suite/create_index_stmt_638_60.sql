SELECT * FROM bitmap_split_or t1, bitmap_split_or t2
WHERE t1.a = t2.b OR t1.a = 2;
