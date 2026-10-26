-- Postgres names an unaliased scalar subquery after the column of its select list, so all of
-- these are accepted.
-- expect_no_diagnostics
create view max_and_min as select (select max(a) from t2), (select min(a) from t2);
create view first_values as select (select a from t2 limit 1), (select x from t2 limit 1);
create table max_and_min_table as select (select max(a) from t2), (select min(a) from t2);
select (select max(a) from t2), (select min(a) from t2) into max_and_min_into;
create view nested as select (select (select max(a) from t2)), (select min(a) from t2);
create view set_operation as select (select a from t2 union select x from t2 limit 1), x from t2;
create view named as select (select 1 as q), (select 1 as r);
create view row_value as select (values (1)), 1 as column2;
-- `?column?` and `text`, and `max` and `min`.
create view casted as select (select 1)::text, 'x'::text;
create view casted_aggregates as select (select max(a) from t2)::text, (select min(a) from t2)::text;
-- `(t2.c).arr[1]` is named after the field `arr`, not after `c`.
create view field_subscript as select (t2.c).arr[1], c from t2;
create view subscript_of_field as select ((t2.c).arr)[1], (t2.c).f from t2;
-- A CASE takes the name of its ELSE result if that has a good name: `a`, `x` and `case`.
create view cases as select case when true then 1 else a end, case when true then 1 else x end, case when true then 1 end from t2;
