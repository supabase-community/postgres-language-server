select (select grouping(v1)) from (values ((select 1))) v(v1) group by v1;
