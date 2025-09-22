create temporary view vw_rngfunc as
  select * from rows from(getrngfunc9(1),
                      getrngfunc7(1) AS (rngfuncid int, rngfuncsubid int, rngfuncname text),
                      getrngfunc1(1))
                with ordinality as t1(a,b,c,d,e,f,g,n);
