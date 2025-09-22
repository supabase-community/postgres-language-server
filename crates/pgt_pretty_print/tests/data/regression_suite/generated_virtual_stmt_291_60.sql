select sum(t2.b) over (partition by t2.a),
       sum(t2.c) over (partition by t2.a),
       sum(t2.d) over (partition by t2.a)
from gtest32 as t1 left join gtest32 as t2 on (t1.a = t2.a)
order by t1.a;
