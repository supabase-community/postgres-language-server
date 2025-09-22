select t1.unique1,t2.unique1 from tenk1 t1
inner join tenk1 t2 on t1.two = t2.two
  and t1.unique1 = (select min(unique1) from tenk1
                    where t2.unique1=unique1)
where t1.unique1 < 10 and t2.unique1 < 10
order by t1.unique1;
