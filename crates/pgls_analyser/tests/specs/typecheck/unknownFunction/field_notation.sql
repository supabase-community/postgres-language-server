-- `a(p)` selects field `a` of the composite column `p`. A scalar column can't be a row, so
-- `nope(id)` is a call of a function that doesn't exist.
-- expect_lint/unknownFunction
create type pair as (a int, b text);
create table pairs (id int, p pair);
select a(p), b(pairs.p) from pairs;
select nope(id) from pairs;
