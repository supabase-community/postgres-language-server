select * from
  (select * from tenk1 order by four) t1 join tenk1 t2 on t1.four = t2.four and t1.two = t2.two
order by t1.four, t1.two limit 1;
