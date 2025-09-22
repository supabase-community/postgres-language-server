select * from tenk1 a, tenk1 b
where exists(select * from tenk1 c
             where b.twothousand = c.twothousand and b.fivethous <> c.fivethous)
      and a.tenthous = b.tenthous and a.tenthous < 5000;
