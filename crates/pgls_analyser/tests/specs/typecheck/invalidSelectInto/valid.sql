-- expect_no_diagnostics
select 1 as id into ids union select 2;
select 1 as id into more_ids union select 2 union select 3;
(select 1 as id into even_more_ids) union (select 2 union select 3);
