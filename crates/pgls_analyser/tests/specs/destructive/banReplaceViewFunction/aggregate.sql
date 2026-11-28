-- expect_lint/banReplaceViewFunction
create or replace aggregate a (int) (sfunc = int4pl, stype = int);
