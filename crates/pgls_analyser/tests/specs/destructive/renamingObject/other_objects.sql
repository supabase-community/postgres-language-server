-- expect_lint/renamingObject
alter type order_status rename value 'shipped' to 'dispatched';
-- expect_lint/renamingObject
alter table users rename constraint users_email_key to users_email_unique;
-- expect_lint/renamingObject
alter function calculate_total(int) rename to compute_total;
-- expect_lint/renamingObject
alter schema billing rename to invoicing;
-- expect_lint/renamingObject
alter index users_email_idx rename to users_email_index;
-- expect_lint/renamingObject
alter materialized view user_stats rename to account_stats;
