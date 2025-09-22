select sum(ten) from onek group by two, rollup(four::text) order by 1;
