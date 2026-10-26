-- The fix commits at the end of the section, not of the file.
-- section: up transaction:false
-- expect_lint/banUncommittedTransaction
begin;
create table users (id bigint);

-- section: down transaction:false
drop table users;
