select t1.ten, sum(x) from
  tenk1 t1 left join lateral (
    select t1.ten + t2.ten as x, t2.fivethous from tenk1 t2
  ) ss on t1.unique1 = ss.fivethous
group by t1.ten
order by t1.ten;
