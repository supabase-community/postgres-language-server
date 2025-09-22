SELECT q1, case when q1 > 0 then generate_series(1,3) else 0 end FROM int8_tbl;
