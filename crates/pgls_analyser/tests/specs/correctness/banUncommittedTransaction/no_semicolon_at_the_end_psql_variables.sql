-- The file ends without a semicolon and uses psql variables. The fix adds the missing
-- semicolon before the COMMIT, and it is checked on the text of the file with the variables
-- read as parameters.
-- expect_lint/banUncommittedTransaction
begin;
create table :"schema".users (id bigint default :'id')