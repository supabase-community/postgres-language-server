select sum(ten) from onek group by rollup(four::text), two order by 1;
