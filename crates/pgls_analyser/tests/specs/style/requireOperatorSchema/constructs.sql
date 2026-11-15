-- expect_lint/requireOperatorSchema
select * from pg_catalog.pg_class where relkind in ('r', 'p');
-- expect_lint/requireOperatorSchema
select * from pg_catalog.pg_class where relname like 'pg%';
-- expect_lint/requireOperatorSchema
select * from pg_catalog.pg_class where relname is distinct from 'x';
-- expect_lint/requireOperatorSchema
select * from pg_catalog.pg_class where relpages between 1 and 10;
-- expect_lint/requireOperatorSchema
select nullif(1, 2);
-- expect_lint/requireOperatorSchema
select * from pg_catalog.pg_class where oid in (select indrelid from pg_catalog.pg_index);
-- expect_lint/requireOperatorSchema
select * from pg_catalog.pg_class where relkind = any (array['r', 'p']);
-- expect_lint/requireOperatorSchema
select * from pg_catalog.pg_class order by relname using <;
-- expect_lint/requireOperatorSchema
select -relpages from pg_catalog.pg_class;
