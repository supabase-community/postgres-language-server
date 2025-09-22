select i8.q2, ss.* from
int8_tbl i8,
lateral (select t1.fivethous, i4.f1 from tenk1 t1 join int4_tbl i4
         on t1.fivethous = i4.f1+i8.q2 order by 1,2) ss;
