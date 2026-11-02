-- The file ends without a semicolon and uses psql variables. The fix is checked on the text of
-- the file with the variables read as parameters, so the fix that would make it
-- `create table ... commit;` is left out.
-- expect_lint/banUncommittedTransaction
begin;
create table :"schema".users (id bigint default :'id')