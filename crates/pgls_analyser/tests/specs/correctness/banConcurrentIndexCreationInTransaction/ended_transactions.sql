-- Postgres doesn't nest transaction blocks: one COMMIT ends the block, also after a second
-- BEGIN or a savepoint. PREPARE TRANSACTION ends it too.
-- section: up transaction:false queries:separate
begin;
begin;
commit;
create index concurrently users_email_idx on users (email);
begin;
savepoint a;
commit;
drop index concurrently users_email_idx;
begin;
prepare transaction 'tx';
create index concurrently users_name_idx on users (name);
-- COMMIT AND CHAIN opens the next transaction block right away.
begin;
commit and chain;
-- expect_lint/banConcurrentIndexCreationInTransaction
drop index concurrently users_name_idx;
commit;
