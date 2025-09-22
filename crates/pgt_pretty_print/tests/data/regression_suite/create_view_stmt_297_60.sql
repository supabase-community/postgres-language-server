select (r).column2 from (select r from (values(1,2),(3,4)) r limit 1) ss;
