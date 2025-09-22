create view v2a as select * from (tt2 join tt3 using (b,c) join tt4 using (b)) j;
