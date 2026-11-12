-- Postgres doesn't nest transaction blocks: one COMMIT ends the block, also after a second
-- BEGIN or a savepoint. PREPARE TRANSACTION ends it too.
-- section: up transaction:false queries:separate
begin;
begin;
commit;
vacuum users;
begin;
savepoint a;
commit;
vacuum users;
begin;
prepare transaction 'tx';
vacuum users;
-- COMMIT AND CHAIN opens the next transaction block right away.
begin;
commit and chain;
-- expect_lint/banNonTransactionalStatementInTransaction
vacuum users;
commit;
