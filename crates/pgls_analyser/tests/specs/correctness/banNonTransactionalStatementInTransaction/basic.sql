begin;
-- expect_lint/banNonTransactionalStatementInTransaction
vacuum users;
-- expect_lint/banNonTransactionalStatementInTransaction
vacuum (analyze) users;
analyze users;
-- expect_lint/banNonTransactionalStatementInTransaction
create database app;
-- expect_lint/banNonTransactionalStatementInTransaction
drop database if exists app;
-- expect_lint/banNonTransactionalStatementInTransaction
alter database app set tablespace fast;
alter database app set work_mem = '4MB';
-- expect_lint/banNonTransactionalStatementInTransaction
create tablespace fast location '/mnt/fast';
-- expect_lint/banNonTransactionalStatementInTransaction
drop tablespace fast;
-- expect_lint/banNonTransactionalStatementInTransaction
alter system set work_mem = '4MB';
-- expect_lint/banNonTransactionalStatementInTransaction
cluster;
cluster users using users_pkey;
-- expect_lint/banNonTransactionalStatementInTransaction
reindex schema public;
-- expect_lint/banNonTransactionalStatementInTransaction
reindex system app;
-- expect_lint/banNonTransactionalStatementInTransaction
reindex database app;
reindex table users;
-- expect_lint/banNonTransactionalStatementInTransaction
discard all;
discard plans;
-- expect_lint/banNonTransactionalStatementInTransaction
commit prepared 'tx';
-- expect_lint/banNonTransactionalStatementInTransaction
rollback prepared 'tx';
-- expect_lint/banNonTransactionalStatementInTransaction
create subscription sub connection 'dbname=app' publication pub;
-- expect_lint/banNonTransactionalStatementInTransaction
create subscription sub connection 'dbname=app' publication pub with (connect = false, create_slot = true);
create subscription sub connection 'dbname=app' publication pub with (connect = false);
create subscription sub connection 'dbname=app' publication pub with (create_slot = false, slot_name = none);
-- expect_lint/banNonTransactionalStatementInTransaction
alter subscription sub refresh publication;
alter subscription sub disable;
-- CONCURRENTLY is reported by banConcurrentIndexCreationInTransaction.
reindex schema concurrently public;
commit;
