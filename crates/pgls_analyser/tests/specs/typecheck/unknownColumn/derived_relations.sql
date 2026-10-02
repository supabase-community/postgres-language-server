-- Columns of CREATE TABLE AS and views are derived from their query (#786).
create table source (id int8, name text);
create table copied as select id, name as label from source;
create view everything as select * from source;
select label from copied;
select id, name from everything;
-- expect_lint/unknownColumn
select name from copied;
