select count(*)
from tenk1 t1
join tenk1 t2 on t1.unique1 = t2.unique2
join tenk1 t3 on t2.unique1 = t3.unique1
order by count(*);
