create view view_of_joins_2d as select * from (tbl1 join tbl1a using (a) as x) as y;
