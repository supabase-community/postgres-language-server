select f1, count(*) from
t1 x(x0,x1) left join (t1 left join t2 using(f1)) on (x0 = 0)
group by f1;
