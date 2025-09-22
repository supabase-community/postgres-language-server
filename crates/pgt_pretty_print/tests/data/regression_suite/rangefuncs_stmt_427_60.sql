select * from int8_tbl, coalesce(row(1)) as (a int, b int);
