copy (select t from test1 where id = 1 UNION select * from v_test1 ORDER BY 1) to stdout;
