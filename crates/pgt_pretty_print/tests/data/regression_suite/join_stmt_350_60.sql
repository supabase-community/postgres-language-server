select * from t t1
    left join (select 1 as x, * from t t2(i2)) t2ss on t1.i = t2ss.i2
    left join t t3(i3) on false
    left join t t4(i4) on t4.i4 > t2ss.x;
