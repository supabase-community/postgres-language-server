-- expect_lint/banDropFunction
drop procedure archive_orders();
-- expect_lint/banDropFunction
drop routine archive_orders();
-- expect_lint/banDropFunction
drop aggregate my_sum(int);
