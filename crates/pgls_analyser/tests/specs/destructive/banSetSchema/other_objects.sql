-- expect_lint/banSetSchema
alter view active_users set schema reporting;
-- expect_lint/banSetSchema
alter function calculate_total(int) set schema billing;
-- expect_lint/banSetSchema
alter type order_status set schema billing;
-- expect_lint/banSetSchema
alter extension hstore set schema extensions;
-- expect_lint/banSetSchema
alter sequence order_number_seq set schema billing;
