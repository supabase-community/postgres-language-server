select * from (select * from ab where a = 1 union all select * from ab) ab where b = (select 1);
