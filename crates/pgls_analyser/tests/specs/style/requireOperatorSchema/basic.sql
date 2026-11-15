-- expect_lint/requireOperatorSchema
select * from pg_catalog.pg_stat_io where backend_type = 'client backend';
