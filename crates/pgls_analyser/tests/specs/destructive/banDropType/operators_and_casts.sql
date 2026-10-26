-- expect_lint/banDropType
drop cast (text as point);
-- expect_lint/banDropType
drop operator === (int, int);
-- expect_lint/banDropType
drop operator class my_ops using btree;
-- expect_lint/banDropType
drop operator family my_ops using btree;
