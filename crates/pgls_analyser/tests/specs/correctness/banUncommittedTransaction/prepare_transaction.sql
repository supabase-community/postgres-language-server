-- expect_no_diagnostics
-- PREPARE TRANSACTION ends the transaction block; COMMIT PREPARED commits it later.
begin;
create table users (id bigint);
prepare transaction 'create_users';
