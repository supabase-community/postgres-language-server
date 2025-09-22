select * from (select (pg_timezone_names()).name) ptn where name='UTC' limit 1;
