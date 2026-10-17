-- expect_lint/preferJsonb
alter table users add column settings JSON default '{}';
