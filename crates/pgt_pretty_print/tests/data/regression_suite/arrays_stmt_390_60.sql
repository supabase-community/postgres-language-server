select array_agg(unique1) from (select unique1 from tenk1 where unique1 < 15 order by unique1) ss;
