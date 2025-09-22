create view view_of_joins_2c as select * from (tbl1 join tbl1a using (a)) as y;
