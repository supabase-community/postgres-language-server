-- Postgres rejects both INHERIT statements: `missing_column` doesn't have the column `x` of
-- `parent`, and `other_type` has it with another type. So neither gets the column added to
-- `parent` afterwards.
create table parent (x int);
create table missing_column (y int);
alter table missing_column inherit parent;
create table other_type (x text);
alter table other_type inherit parent;
create table accepted (x int);
alter table accepted inherit parent;
alter table parent add column w int;
-- expect_lint/unknownColumn
select w from missing_column;
-- expect_lint/unknownColumn
select w from other_type;
select w from accepted;
