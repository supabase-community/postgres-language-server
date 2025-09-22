select * from rows from(getrngfunc9(1),getrngfunc8(1),
                    getrngfunc7(1) AS (rngfuncid int, rngfuncsubid int, rngfuncname text),
                    getrngfunc6(1) AS (rngfuncid int, rngfuncsubid int, rngfuncname text),
                    getrngfunc5(1),getrngfunc4(1),getrngfunc3(1),getrngfunc2(1),getrngfunc1(1))
              with ordinality as t1(a,b,c,d,e,f,g,h,i,j,k,l,m,o,p,q,r,s,t,u);
