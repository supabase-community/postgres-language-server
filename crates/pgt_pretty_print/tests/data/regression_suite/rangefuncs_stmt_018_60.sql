create temporary view vw_ord as select * from (values (1)) v(n) join rows from(rngfunct(1),rngfunct(2)) with ordinality as z(a,b,c,d,ord) on (n=ord);
