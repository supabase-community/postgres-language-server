begin;
-- expect_lint/banNonTransactionalStatementInTransaction
create subscription sub connection 'dbname=app' publication pub with (connect = on);
-- `off` is not a reserved keyword, so it is parsed as a type name.
create subscription sub connection 'dbname=app' publication pub with (connect = off);
-- Postgres rejects these values.
create subscription sub connection 'dbname=app' publication pub with (create_slot = 2);
create subscription sub connection 'dbname=app' publication pub with (create_slot = 'yes');
create subscription sub connection 'dbname=app' publication pub with (connect = '0');
commit;
