-- A serial2 column is a smallint, and there is no smallint + timestamp operator.
create table counters (a serial2);
-- expect_lint/operatorTypeMismatch
select a + timestamp '2020-01-01' from counters;
